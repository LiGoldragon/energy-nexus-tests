use energy_nexus_wire::{
    AdmissionRefusal, AdmissionStage, Admittable, BinaryConnection, Framing, PayloadReceivable,
    PayloadRefusal, PeerIdentifiable, CAPACITY,
};
use signal::{
    ByteViewable, ContractDigest, Contracted, Handshake, HandshakeReceipt, HandshakeRejection,
    Restorable, Signal, Signalizable,
};
use std::{
    fmt::Debug,
    io::{Read, Write},
    os::unix::net::UnixStream,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

fn sockets() -> (UnixStream, UnixStream) {
    let (server, client) = UnixStream::pair().unwrap();
    for stream in [&server, &client] {
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .unwrap();
    }
    (server, client)
}
fn ordinary() -> signal_energy_nexus::Call {
    use signal_energy_nexus::{library::*, Call, Query};
    Call {
        caller: Caller {
            process_id: ProcessId(4242),
            user_id: UserId(1001),
        },
        query: Query::Subscribe,
    }
}
fn public() -> meta_signal_energy_nexus::Call {
    meta_signal_energy_nexus::Call {
        caller: ordinary().caller,
        query: meta_signal_energy_nexus::Query::Configuration,
    }
}
fn internal() -> meta_signal_energy_nexus::internal::Call {
    meta_signal_energy_nexus::internal::Call {
        caller: ordinary().caller,
        query: meta_signal_energy_nexus::internal::Query::Configuration,
    }
}
fn emission() -> energy_nexus_emission::Query {
    energy_nexus_emission::Query::Emit(energy_nexus_emission::Emission::TurnEnds)
}
fn wrong_greetings<C: Contracted>() -> Vec<Handshake> {
    let mut greetings = vec![
        signal_energy_nexus::Call::greeting(),
        meta_signal_energy_nexus::Call::greeting(),
        meta_signal_energy_nexus::internal::Call::greeting(),
        energy_nexus_emission::Query::greeting(),
    ];
    greetings.retain(|greeting| greeting.contract_digest != C::contract_digest());
    for text in [
        include_str!("legacy-contracts/signal-energy-nexus/tests/fixtures/contract.txt"),
        include_str!("legacy-contracts/meta-signal-energy-nexus/tests/fixtures/contract.txt"),
    ] {
        let digest = text
            .lines()
            .find_map(|line| line.strip_prefix("ContractDigest "))
            .unwrap()
            .parse::<i64>()
            .unwrap();
        greetings.push(Handshake {
            contract_digest: ContractDigest(digest),
        });
    }
    greetings.push(Handshake::of_source(include_str!("legacy-emission.ethos")));
    greetings.push(Handshake {
        contract_digest: ContractDigest(C::contract_digest().0 ^ 1),
    });
    greetings
}
fn roundtrip<C>(value: C)
where
    C: Contracted + Signalizable + PartialEq + Debug + Send + 'static,
    Signal<C>: Restorable<C>,
{
    for greeting in wrong_greetings::<C>() {
        let (server, mut client) = sockets();
        let restores = Arc::new(AtomicUsize::new(0));
        let count = restores.clone();
        let thread = std::thread::spawn(move || {
            let admitted = BinaryConnection::<C>::from(server).admit();
            match admitted {
                Err(AdmissionRefusal::Mismatch {
                    expected,
                    observed,
                    receipt_failure,
                }) => {
                    assert_eq!(expected, C::contract_digest());
                    assert!(receipt_failure.is_none());
                    observed
                }
                Ok(mut admitted) => {
                    count.fetch_add(1, Ordering::SeqCst);
                    let _ = admitted.receive();
                    panic!("wrong root reached payload restoration");
                }
                Err(other) => panic!("wrong mismatch refusal: {other:?}"),
            }
        });
        // Valid current payload is pipelined behind the wrong/old greeting.
        let greeting_bytes = greeting.signalize().unwrap();
        let payload_bytes = value.signalize().unwrap();
        let mut pipelined = Vec::new();
        for frame in [greeting_bytes.bytes(), payload_bytes.bytes()] {
            pipelined.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            pipelined.extend_from_slice(frame);
        }
        client.write_all(&pipelined).unwrap();
        let receipt = Signal::<HandshakeReceipt>::from(client.read_frame().unwrap())
            .restore()
            .unwrap();
        assert_eq!(
            receipt,
            HandshakeReceipt::GreetingRefused(HandshakeRejection::ContractMismatch(
                C::contract_digest()
            ))
        );
        assert_eq!(thread.join().unwrap(), greeting.contract_digest);
        assert_eq!(restores.load(Ordering::SeqCst), 0);
        assert!(
            client.read_frame().is_err(),
            "refused connection must close"
        );
    }
    let (server, mut client) = sockets();
    let thread = std::thread::spawn(move || {
        let mut admitted = BinaryConnection::<C>::from(server)
            .admit()
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert!(admitted.peer().pid() > 0);
        admitted.receive().unwrap()
    });
    client
        .write_frame(C::greeting().signalize().unwrap().bytes())
        .unwrap();
    let receipt = Signal::<HandshakeReceipt>::from(client.read_frame().unwrap())
        .restore()
        .unwrap();
    assert_eq!(receipt, HandshakeReceipt::Greeted(C::contract_digest()));
    client
        .write_frame(value.signalize().unwrap().bytes())
        .unwrap();
    assert_eq!(thread.join().unwrap(), value);
}
#[test]
fn ordinary_rejects_other_old_and_wrong_roots_before_payload_and_accepts_current() {
    roundtrip(ordinary());
}
#[test]
fn public_meta_contract_rejects_other_old_and_wrong_roots_before_payload_and_accepts_current() {
    roundtrip(public());
}
#[test]
fn internal_policy_rejects_other_old_and_wrong_roots_before_payload_and_accepts_current() {
    roundtrip(internal());
}
#[test]
fn emission_rejects_other_old_and_wrong_roots_before_payload_and_accepts_current() {
    roundtrip(emission());
}

#[test]
fn malformed_archive_and_truncated_io_have_distinct_causes() {
    let (server, mut client) = sockets();
    client.write_frame(&[]).unwrap();
    assert!(matches!(
        BinaryConnection::<signal_energy_nexus::Call>::from(server).admit(),
        Err(AdmissionRefusal::MalformedHandshake)
    ));
    let (server, mut client) = sockets();
    client.write_all(&[1, 0]).unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();
    assert!(
        matches!(BinaryConnection::<signal_energy_nexus::Call>::from(server).admit(), Err(AdmissionRefusal::Io { stage: AdmissionStage::GreetingRead, source }) if source.kind() == std::io::ErrorKind::UnexpectedEof)
    );
}
#[test]
fn configured_read_timeout_is_retained() {
    let (server, _client) = sockets();
    server
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    assert!(
        matches!(BinaryConnection::<signal_energy_nexus::Call>::from(server).admit(), Err(AdmissionRefusal::Io { stage: AdmissionStage::GreetingRead, source }) if matches!(source.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut))
    );
}
#[test]
fn framing_capacity_is_checked_before_handshake_archive_decoding() {
    let (server, mut client) = sockets();
    client
        .write_all(&((CAPACITY + 1) as u32).to_le_bytes())
        .unwrap();
    assert!(
        matches!(BinaryConnection::<signal_energy_nexus::Call>::from(server).admit(), Err(AdmissionRefusal::Io { stage: AdmissionStage::GreetingRead, source }) if source.kind() == std::io::ErrorKind::InvalidData)
    );
}
#[test]
fn matched_handshake_does_not_bypass_payload_validation() {
    let (server, mut client) = sockets();
    let thread = std::thread::spawn(move || {
        let mut admitted = BinaryConnection::<signal_energy_nexus::Call>::from(server)
            .admit()
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert!(matches!(admitted.receive(), Err(PayloadRefusal::Malformed)));
    });
    client
        .write_frame(
            signal_energy_nexus::Call::greeting()
                .signalize()
                .unwrap()
                .bytes(),
        )
        .unwrap();
    client.read_frame().unwrap();
    client.write_frame(&[]).unwrap();
    thread.join().unwrap();
}
#[test]
fn outgoing_connection_requires_matching_receipt_and_then_sends_actual_payload() {
    let (server, client) = sockets();
    let thread = std::thread::spawn(move || {
        let mut admitted = BinaryConnection::<signal_energy_nexus::Call>::from(server)
            .admit()
            .unwrap_or_else(|e| panic!("{e:?}"));
        admitted.receive().unwrap()
    });
    let mut client = BinaryConnection::<signal_energy_nexus::Call>::from(client)
        .greet()
        .unwrap_or_else(|e| panic!("{e:?}"));
    use energy_nexus_wire::PayloadSendable;
    client.send(&ordinary()).unwrap();
    assert_eq!(thread.join().unwrap(), ordinary());
}
#[test]
fn outgoing_wrong_or_malformed_receipt_is_not_admission() {
    for receipt in [
        Some(HandshakeReceipt::Greeted(ContractDigest(17))),
        Some(HandshakeReceipt::GreetingRefused(
            HandshakeRejection::ContractMismatch(ContractDigest(17)),
        )),
        None,
    ] {
        let (mut server, client) = sockets();
        let thread = std::thread::spawn(move || {
            server.read_frame().unwrap();
            match receipt {
                Some(receipt) => server
                    .write_frame(receipt.signalize().unwrap().bytes())
                    .unwrap(),
                None => server.write_frame(&[]).unwrap(),
            }
            let mut byte = [0];
            assert!(
                matches!(server.read(&mut byte), Ok(0)),
                "no payload after a bad receipt"
            );
        });
        assert!(matches!(
            BinaryConnection::<signal_energy_nexus::Call>::from(client).greet(),
            Err(AdmissionRefusal::UnexpectedReceipt { .. }
                | AdmissionRefusal::Refused { .. }
                | AdmissionRefusal::MalformedReceipt)
        ));
        thread.join().unwrap();
    }
}
