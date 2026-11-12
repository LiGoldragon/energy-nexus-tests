use energy_nexus_wire::{
    AdmissionRefusal, AdmissionStage, Admittable, BinaryConnection, ExchangeContract, Framing,
    PayloadRefusal, PeerIdentifiable, RequestReceivable, CAPACITY,
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
    C: ExchangeContract + Signalizable + PartialEq + Debug + Send + 'static,
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
                    let _ = admitted.receive_request();
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
        admitted.receive_request().unwrap()
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
        assert!(matches!(
            admitted.receive_request(),
            Err(PayloadRefusal::Malformed)
        ));
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
        admitted.receive_request().unwrap()
    });
    let mut client = BinaryConnection::<signal_energy_nexus::Call>::from(client)
        .greet()
        .unwrap_or_else(|e| panic!("{e:?}"));
    use energy_nexus_wire::RequestSendable;
    client.send_request(&ordinary()).unwrap();
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

fn exchange<C>(request: C, response: C::Response)
where
    C: ExchangeContract + Signalizable + PartialEq + Debug + Send + 'static,
    C::Response: Signalizable + PartialEq + Debug + Send + 'static,
    Signal<C>: Restorable<C>,
    Signal<C::Response>: Restorable<C::Response>,
{
    use energy_nexus_wire::{RequestSendable, ResponseReceivable, ResponseSendable};
    let (server, client) = sockets();
    let thread = std::thread::spawn(move || {
        let mut server = BinaryConnection::<C>::from(server)
            .admit()
            .unwrap_or_else(|e| panic!("{e:?}"));
        let received = server.receive_request().unwrap();
        server.send_response(&response).unwrap();
        (received, response)
    });
    let mut client = BinaryConnection::<C>::from(client)
        .greet()
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert!(client.peer().pid() > 0);
    client.send_request(&request).unwrap();
    let received = client.receive_response().unwrap();
    let (server_request, server_response) = thread.join().unwrap();
    assert_eq!(server_request, request);
    assert_eq!(received, server_response);
}
#[test]
fn ordinary_two_way_contract_exchange() {
    exchange(
        ordinary(),
        signal_energy_nexus::Response::Subscribed(Vec::new()),
    );
}
#[test]
fn public_meta_two_way_contract_exchange_not_a_public_binary_listener() {
    // Contract/role proof on an isolated pair; the actual public frontend is text.
    exchange(
        public(),
        meta_signal_energy_nexus::Response::Refused(
            signal_energy_nexus::library::Failure::ResponseMalformed,
        ),
    );
}
#[test]
fn internal_policy_two_way_contract_exchange() {
    exchange(
        internal(),
        meta_signal_energy_nexus::internal::Response::Refused(
            signal_energy_nexus::library::Failure::ResponseMalformed,
        ),
    );
}
#[test]
fn emission_two_way_contract_exchange() {
    exchange(
        emission(),
        energy_nexus_emission::Response::Emitted(
            energy_nexus_emission::EmissionOutcome::TurnEndsObserved(Vec::new()),
        ),
    );
}

#[test]
fn compiler_enforces_root_direction_and_admission_using_the_built_libraries() {
    // Reuse this gate's exact dependency artifacts; compile only tiny metadata probes.
    let executable = std::env::current_exe().unwrap();
    let dependencies = executable.parent().unwrap().to_owned();
    let (unit, unit_hash) = executable
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .rsplit_once('-')
        .unwrap();
    let fingerprints = dependencies.parent().unwrap().join(".fingerprint");
    let unit_fingerprint = fingerprints
        .join(format!("{}-{unit_hash}", env!("CARGO_PKG_NAME")))
        .join(format!("test-integration-test-{unit}.json"));
    let unit_metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&unit_fingerprint).unwrap()).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let libraries = [
        "energy_nexus_wire",
        "signal_energy_nexus",
        "meta_signal_energy_nexus",
        "energy_nexus_emission",
    ];
    let mut external = Vec::new();
    for library in libraries {
        let identities: Vec<_> = unit_metadata["deps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry[1].as_str() == Some(library))
            .collect();
        assert_eq!(
            identities.len(),
            1,
            "unit must name exactly one {library} dependency"
        );
        let expected = identities[0][3].as_u64().unwrap();
        let package = library.replace('_', "-");
        let mut candidates = Vec::new();
        for entry in std::fs::read_dir(&fingerprints).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let Some(hash) = name.strip_prefix(&format!("{package}-")) else {
                continue;
            };
            let marker = path.join(format!("lib-{library}"));
            if !marker.is_file() {
                continue;
            }
            let hex = std::fs::read_to_string(marker).unwrap();
            let bytes: Vec<_> = (0..hex.len())
                .step_by(2)
                .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
                .collect();
            let observed = u64::from_le_bytes(bytes.try_into().unwrap());
            if observed == expected {
                let artifact = dependencies.join(format!("lib{library}-{hash}.rlib"));
                assert!(
                    artifact.is_file(),
                    "matched fingerprint has no rlib: {artifact:?}"
                );
                candidates.push(artifact);
            }
        }
        assert_eq!(
            candidates.len(),
            1,
            "ambiguous or absent exact gate artifact for {library}: {candidates:?}"
        );
        external.push(format!("{library}={}", candidates[0].display()));
    }
    let prelude = r#"
#![allow(dead_code, unused_imports)]
use energy_nexus_wire::{AdmittedServerConnection as Server, AdmittedClientConnection as Client, BinaryConnection, RequestSendable, RequestReceivable, ResponseSendable, ResponseReceivable};
use signal_energy_nexus::{Call as Ordinary, Response as OrdinaryReply};
use meta_signal_energy_nexus::{Call as Public, Response as PublicReply};
use meta_signal_energy_nexus::internal::{Call as Policy, Response as PolicyReply};
use energy_nexus_emission::{Query as Emission, Response as EmissionReply};
"#;
    let positive = r#"
fn ordinary(mut s: Server<Ordinary>, mut c: Client<Ordinary>, q: Ordinary, r: OrdinaryReply) { c.send_request(&q).unwrap(); let _: Ordinary = s.receive_request().unwrap(); s.send_response(&r).unwrap(); let _: OrdinaryReply = c.receive_response().unwrap(); }
fn public(mut s: Server<Public>, mut c: Client<Public>, q: Public, r: PublicReply) { c.send_request(&q).unwrap(); let _: Public = s.receive_request().unwrap(); s.send_response(&r).unwrap(); let _: PublicReply = c.receive_response().unwrap(); }
fn policy(mut s: Server<Policy>, mut c: Client<Policy>, q: Policy, r: PolicyReply) { c.send_request(&q).unwrap(); let _: Policy = s.receive_request().unwrap(); s.send_response(&r).unwrap(); let _: PolicyReply = c.receive_response().unwrap(); }
fn emission(mut s: Server<Emission>, mut c: Client<Emission>, q: Emission, r: EmissionReply) { c.send_request(&q).unwrap(); let _: Emission = s.receive_request().unwrap(); s.send_response(&r).unwrap(); let _: EmissionReply = c.receive_response().unwrap(); }
"#;
    let cases = [
        ("positive", positive, None),
        ("wrong_request_root", "fn check(mut c: Client<Ordinary>, q: Public) { c.send_request(&q).unwrap(); }", Some("E0308")),
        ("wrong_reply_root", "fn check(mut s: Server<Ordinary>, r: PublicReply) { s.send_response(&r).unwrap(); }", Some("E0308")),
        ("wrong_public_policy_request", "fn check(mut c: Client<Public>, q: Policy) { c.send_request(&q).unwrap(); }", Some("E0308")),
        ("wrong_public_policy_reply", "fn check(mut s: Server<Public>, r: PolicyReply) { s.send_response(&r).unwrap(); }", Some("E0308")),
        ("wrong_emission_request", "fn check(mut c: Client<Emission>, q: Ordinary) { c.send_request(&q).unwrap(); }", Some("E0308")),
        ("wrong_emission_reply", "fn check(mut s: Server<Emission>, r: OrdinaryReply) { s.send_response(&r).unwrap(); }", Some("E0308")),
        ("server_sends_request", "fn check(mut s: Server<Ordinary>, q: Ordinary) { s.send_request(&q).unwrap(); }", Some("E0599")),
        ("client_sends_reply", "fn check(mut c: Client<Ordinary>, r: OrdinaryReply) { c.send_response(&r).unwrap(); }", Some("E0599")),
        ("server_receives_reply", "fn check(mut s: Server<Ordinary>) { let _: OrdinaryReply = s.receive_response().unwrap(); }", Some("E0599")),
        ("client_receives_request", "fn check(mut c: Client<Ordinary>) { let _: Ordinary = c.receive_request().unwrap(); }", Some("E0599")),
        ("unadmitted_send", "fn check(mut c: BinaryConnection<Ordinary>, q: Ordinary) { c.send_request(&q).unwrap(); }", Some("E0599")),
        ("unadmitted_receive", "fn check(mut s: BinaryConnection<Ordinary>) { let _: Ordinary = s.receive_request().unwrap(); }", Some("E0599")),
        ("private_owner", "use energy_nexus_wire::AdmittedConnection;", Some("E0432")),
        ("old_unrestricted_send", "use energy_nexus_wire::PayloadSendable;", Some("E0432")),
        ("old_unrestricted_receive", "use energy_nexus_wire::PayloadReceivable;", Some("E0432")),
        ("private_codec", "fn check(mut c: Client<Ordinary>, q: Public) { c.send_message(&q).unwrap(); }", Some("E0599")),
    ];
    for (name, body, expected) in cases {
        let source = scratch.path().join(format!("{name}.rs"));
        std::fs::write(&source, format!("{prelude}\n{body}")).unwrap();
        let mut compiler =
            std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
        compiler
            .args([
                "--edition=2021",
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=json",
            ])
            .arg("--out-dir")
            .arg(scratch.path())
            .arg("-L")
            .arg(format!("dependency={}", dependencies.display()));
        for library in &external {
            compiler.arg("--extern").arg(library);
        }
        let output = compiler.arg(&source).output().unwrap();
        let diagnostics = String::from_utf8_lossy(&output.stderr);
        match expected {
            None => assert!(
                output.status.success(),
                "positive harness failed: {diagnostics}"
            ),
            Some(code) => {
                assert!(!output.status.success(), "{name} unexpectedly compiled");
                assert!(
                    diagnostics.contains(&format!("\"code\":\"{code}\"")),
                    "{name} failed for the wrong reason: {diagnostics}"
                );
                assert!(
                    !diagnostics.contains("\"code\":\"E0463\"")
                        && !diagnostics.contains("\"code\":\"E0514\""),
                    "artifact/toolchain mismatch: {diagnostics}"
                );
            }
        }
        eprintln!(
            "exchange compiler probe {name}: {}",
            expected.unwrap_or("accepted")
        );
    }
}
