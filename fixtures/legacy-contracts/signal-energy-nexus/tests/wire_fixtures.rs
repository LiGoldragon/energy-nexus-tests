//! The wire of this contract, held against fixtures made by the 0.4.0
//! revision (97741d2, on signal 10.3.0 and signal-flow 57 at 3d0a4cd).
//!
//! Each fixture line is `Name value`. `binary.txt` holds the rkyv archive of
//! one value of every wire type, in hex; `datom.txt` holds its datom text
//! (read only with the `datom` feature); `contract.txt` holds the contract
//! identity the crate exposes: the digest of its authored Ethos source and the
//! greeting and receipt that carry it.
//!
//! The fixtures were produced by running this file at 97741d2 with
//! `WIRE_FIXTURES_WRITE=<directory>`; the one line that differs there is
//! `topic`, which at 97741d2 builds the alias `Topic = String` and here builds
//! the `Topic` newtype. A change to a byte, a text or a digest fails the test
//! naming the line.

use signal::{ByteViewable, Contracted, Restorable, Signal, Signalizable};
use signal_energy_nexus::*;
use std::collections::BTreeMap;

fn topic(text: &str) -> signal_flow::Topic {
    signal_flow::Topic(text.to_string())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).unwrap())
        .collect()
}

fn fixture(name: &str) -> BTreeMap<String, String> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{path}: {error}"))
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(' ').unwrap();
            (key.to_string(), value.to_string())
        })
        .collect()
}

struct Wire {
    held_binary: BTreeMap<String, String>,
    held_datom: BTreeMap<String, String>,
    binary: Vec<(String, String)>,
    datom: Vec<(String, String)>,
    writing: bool,
}

#[cfg(feature = "datom")]
fn datom_text<T>(value: &T) -> String
where
    T: datom_codec::Datomizable,
{
    use protos::{Compactable, Protosizable};
    value.datomize(Vec::new()).protosize().compact()
}

#[cfg(feature = "datom")]
fn datom_value<T>(text: &str) -> T
where
    datom_codec::Potential<T>: datom_codec::Actualizing<T>,
{
    use datom_codec::{Actualizing, Budget, Potential};
    use protos::ReaderBudget;
    let mut budget = Budget {
        remaining: 4096,
        reader: ReaderBudget::try_from(4096).unwrap(),
        depth: 0,
        maximum_depth: 32,
    };
    Potential::<T>::from(text).actualize(&mut budget).unwrap()
}

macro_rules! case {
    ($wire:expr, $name:expr, $type:ty, $value:expr) => {{
        let name = $name.to_string();
        let value: $type = $value;
        let bytes = value.signalize().unwrap();
        $wire.binary.push((name.clone(), hex(bytes.bytes())));
        if let Some(held) = $wire.held_binary.get(&name) {
            let restored = Signal::<$type>::from(unhex(held)).restore().unwrap();
            assert_eq!(restored, value, "the 0.4.0 bytes of {name} restore to the value");
        }
        #[cfg(feature = "datom")]
        {
            let text = datom_text(&value);
            $wire.datom.push((name.clone(), text));
            if let Some(held) = $wire.held_datom.get(&name) {
                let restored: $type = datom_value(held);
                assert_eq!(restored, value, "the 0.4.0 text of {name} reads as the value");
            }
        }
    }};
}

fn instant(at: i64) -> Instant {
    Instant(at)
}

fn window(name: Name) -> Window {
    Window {
        name,
        used: Percent(37),
        resets: instant(1_789_000_000),
        duration: Minutes(10_080),
    }
}

fn reset(status: ResetStatus) -> Reset {
    Reset {
        reset_id: ResetId("reset-a1".to_string()),
        reset_status: status,
        expires: instant(1_789_100_000),
    }
}

fn reading(provider: Provider) -> Reading {
    Reading {
        provider,
        allowed: true,
        window_vector: vec![window(Name::Session), window(Name::Week), window(Name::FableWeek)],
        reset_vector: vec![reset(ResetStatus::Available), reset(ResetStatus::Redeemed)],
        at: instant(1_788_999_000),
    }
}

fn address(aspect: signal_flow::Aspect, model: signal_flow::Model) -> Address {
    Address {
        aspect,
        topic: topic("energyNexus"),
        model,
    }
}

fn pace(standing: Standing, target: Target) -> Pace {
    Pace {
        provider: Provider::Claude,
        name: Name::Week,
        target,
        expected: Percent(50),
        used: Percent(61),
        standing,
        per_day: Percent(9),
    }
}

fn caller() -> Caller {
    Caller {
        process_id: 4242,
        user_id: 1001,
    }
}

fn produce(wire: &mut Wire) {
    use signal_flow::{Aspect, Model};
    for provider in [Provider::Claude, Provider::Codex] {
        case!(wire, format!("Provider.{provider:?}"), Provider, provider);
    }
    case!(wire, "Percent", Percent, Percent(63));
    case!(wire, "Instant", Instant, instant(1_789_000_000));
    case!(wire, "Minutes", Minutes, Minutes(720));
    for name in [Name::Session, Name::Week, Name::FableWeek] {
        case!(wire, format!("Name.{name:?}"), Name, name);
    }
    case!(wire, "Window", Window, window(Name::Week));
    case!(wire, "ResetId", ResetId, ResetId("reset-a1".to_string()));
    for status in [ResetStatus::Available, ResetStatus::Redeeming, ResetStatus::Redeemed] {
        case!(wire, format!("ResetStatus.{status:?}"), ResetStatus, status);
    }
    case!(wire, "Reset", Reset, reset(ResetStatus::Redeeming));
    case!(wire, "Reading", Reading, reading(Provider::Codex));
    for level in [Level::Plenty, Level::Fair, Level::Low, Level::Out] {
        case!(wire, format!("Level.{level:?}"), Level, level);
    }
    case!(
        wire,
        "Threshold",
        Threshold,
        Threshold {
            level: Level::Fair,
            left: Percent(40)
        }
    );
    for aspect in [Aspect::Psyche, Aspect::Mind, Aspect::Field] {
        case!(
            wire,
            format!("Address.{aspect:?}"),
            Address,
            address(aspect, Model::Sol)
        );
    }
    for model in [
        Model::Fable,
        Model::Opus,
        Model::Sonnet,
        Model::Haiku,
        Model::Astra,
        Model::Sol,
        Model::Luna,
        Model::Terra,
    ] {
        case!(
            wire,
            format!("Recipient.{model:?}"),
            Recipient,
            Recipient(address(Aspect::Mind, model))
        );
    }
    case!(
        wire,
        "Reminder",
        Reminder,
        Reminder {
            provider: Provider::Codex,
            level: Level::Low,
            recipient: Recipient(address(Aspect::Mind, Model::Sol)),
        }
    );
    case!(wire, "Target.Even", Target, Target::Even);
    case!(wire, "Target.SpendBy", Target, Target::SpendBy(instant(1_789_200_000)));
    for standing in [Standing::Over, Standing::Under, Standing::On] {
        case!(wire, format!("Standing.{standing:?}"), Standing, standing);
    }
    case!(wire, "Pace", Pace, pace(Standing::Over, Target::Even));
    case!(wire, "Caller", Caller, caller());
    case!(wire, "Query.Read", Query, Query::Read(Provider::Claude));
    case!(wire, "Query.Subscribe", Query, Query::Subscribe);
    case!(wire, "Query.Redeem", Query, Query::Redeem(Provider::Codex));
    case!(
        wire,
        "Call",
        Call,
        Call {
            caller: caller(),
            input: Query::Read(Provider::Codex)
        }
    );
    case!(
        wire,
        "Read_Data",
        Read_Data,
        Read_Data {
            reading: reading(Provider::Claude),
            pace_vector: vec![
                pace(Standing::Under, Target::SpendBy(instant(1_789_200_000))),
                pace(Standing::On, Target::Even)
            ],
        }
    );
    case!(
        wire,
        "Changed_Data",
        Changed_Data,
        Changed_Data {
            provider: Provider::Codex,
            level: Level::Out,
            reading: reading(Provider::Codex),
        }
    );
    case!(wire, "Refused.ProviderDown", Refused, Refused::ProviderDown(Provider::Claude));
    case!(wire, "Refused.NoReset", Refused, Refused::NoReset);
    case!(
        wire,
        "Refused.RuleRefused",
        Refused,
        Refused::RuleRefused("no reset is available".to_string())
    );
    case!(
        wire,
        "Response.Read",
        Response,
        Response::Read(Read_Data {
            reading: reading(Provider::Claude),
            pace_vector: vec![pace(Standing::Over, Target::Even)],
        })
    );
    case!(wire, "Response.Subscribed", Response, Response::Subscribed);
    case!(
        wire,
        "Response.Changed",
        Response,
        Response::Changed(Changed_Data {
            provider: Provider::Claude,
            level: Level::Plenty,
            reading: reading(Provider::Claude),
        })
    );
    case!(wire, "Response.Redeemed", Response, Response::Redeemed(reset(ResetStatus::Redeemed)));
    case!(wire, "Response.Refused", Response, Response::Refused(Refused::NoReset));
}

fn contract() -> Vec<(String, String)> {
    let greeting = Call::greeting();
    let own = Call::contract_digest();
    let other = signal::ContractDigest(own.0 ^ 1);
    let mut lines = vec![
        ("ETHOS_BYTES".to_string(), ETHOS.len().to_string()),
        ("ContractDigest".to_string(), own.0.to_string()),
        (
            "Handshake.binary".to_string(),
            hex(greeting.signalize().unwrap().bytes()),
        ),
        (
            "Receipt.Greeted.binary".to_string(),
            hex(Call::receipt(&greeting).signalize().unwrap().bytes()),
        ),
        (
            "Receipt.GreetingRefused.binary".to_string(),
            hex(Call::receipt(&signal::Handshake { contract_digest: other })
                .signalize()
                .unwrap()
                .bytes()),
        ),
    ];
    #[cfg(feature = "datom")]
    {
        lines.push(("Handshake.datom".to_string(), datom_text(&greeting)));
        lines.push((
            "Receipt.Greeted.datom".to_string(),
            datom_text(&Call::receipt(&greeting)),
        ));
        lines.push((
            "Receipt.GreetingRefused.datom".to_string(),
            datom_text(&Call::receipt(&signal::Handshake { contract_digest: other })),
        ));
    }
    lines
}

fn render(lines: &[(String, String)]) -> String {
    lines
        .iter()
        .map(|(key, value)| format!("{key} {value}\n"))
        .collect()
}

fn differences(name: &str, produced: &[(String, String)]) -> Vec<String> {
    let held = fixture(name);
    let mut found = Vec::new();
    for (key, value) in produced {
        match held.get(key) {
            Some(expected) if expected == value => {}
            Some(expected) => found.push(format!("{name} {key}: 0.4.0 {expected} now {value}")),
            None => found.push(format!("{name} {key}: not in the 0.4.0 fixture, now {value}")),
        }
    }
    for key in held.keys() {
        if !produced.iter().any(|(produced_key, _)| produced_key == key) {
            found.push(format!("{name} {key}: in the 0.4.0 fixture, not produced now"));
        }
    }
    found
}

#[test]
fn the_wire_is_the_0_4_0_wire() {
    let writing = std::env::var("WIRE_FIXTURES_WRITE").ok();
    let mut wire = Wire {
        held_binary: BTreeMap::new(),
        held_datom: BTreeMap::new(),
        binary: Vec::new(),
        datom: Vec::new(),
        writing: writing.is_some(),
    };
    if !wire.writing {
        wire.held_binary = fixture("binary.txt");
        wire.held_datom = fixture("datom.txt");
    }
    produce(&mut wire);
    let contract = contract();
    if let Some(directory) = writing {
        std::fs::write(format!("{directory}/binary.txt"), render(&wire.binary)).unwrap();
        std::fs::write(format!("{directory}/datom.txt"), render(&wire.datom)).unwrap();
        std::fs::write(format!("{directory}/contract.txt"), render(&contract)).unwrap();
        return;
    }
    let mut found = differences("binary.txt", &wire.binary);
    if cfg!(feature = "datom") {
        found.extend(differences("datom.txt", &wire.datom));
    }
    let held_contract = fixture("contract.txt");
    let produced_contract = contract
        .into_iter()
        .filter(|(key, _)| cfg!(feature = "datom") || !key.ends_with(".datom"))
        .collect::<Vec<_>>();
    for (key, value) in &produced_contract {
        if held_contract.get(key) != Some(value) {
            found.push(format!("contract.txt {key}: 0.4.0 {:?} now {value}", held_contract.get(key)));
        }
    }
    assert!(found.is_empty(), "the wire moved:\n{}", found.join("\n"));
}
