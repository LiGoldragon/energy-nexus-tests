//! The wire of this contract, held against fixtures made by the 0.4.0
//! revision (9f37cb7, on signal 10.3.0, signal-energy-nexus 97741d2 and
//! signal-flow 57 at 3d0a4cd).
//!
//! Each fixture line is `Name value`. `binary.txt` holds the rkyv archive of
//! one value of every wire type, in hex; `datom.txt` holds its datom text
//! (read only with the `datom` feature); `contract.txt` holds the contract
//! identity the crate exposes: the digest of its authored Ethos source and the
//! greeting and receipt that carry it.
//!
//! The fixtures were produced by running this file at 9f37cb7 with
//! `WIRE_FIXTURES_WRITE=<directory>`; the one line that differs there is
//! `topic`, which at 9f37cb7 builds the alias `Topic = String` and here builds
//! the `Topic` newtype. A change to a byte, a text or a digest fails the test
//! naming the line.

use signal::{ByteViewable, Contracted, Restorable, Signal, Signalizable};
use meta_signal_energy_nexus::*;
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

fn address(aspect: signal_flow::Aspect, model: signal_flow::Model) -> signal_energy_nexus::Address {
    signal_energy_nexus::Address {
        aspect,
        topic: topic("energyNexus"),
        model,
    }
}

fn reminder(provider: Provider, level: signal_energy_nexus::Level) -> Reminder {
    Reminder {
        provider,
        level,
        recipient: signal_energy_nexus::Recipient(address(
            signal_flow::Aspect::Mind,
            signal_flow::Model::Sol,
        )),
    }
}

fn caller() -> Caller {
    Caller {
        process_id: 4242,
        user_id: 1001,
    }
}

fn threshold() -> Threshold {
    Threshold {
        level: signal_energy_nexus::Level::Fair,
        left: Percent(40),
    }
}

fn target() -> Target {
    Target::SpendBy(signal_energy_nexus::Instant(1_789_200_000))
}

fn path() -> Path {
    Path("/home/li/.codex/auth.json".to_string())
}

fn sets() -> Vec<(&'static str, Set)> {
    use signal_energy_nexus::Level;
    vec![
        ("Threshold", Set::Threshold(threshold())),
        ("Reminder", Set::Reminder(reminder(Provider::Codex, Level::Low))),
        ("RemoveReminder", Set::RemoveReminder(reminder(Provider::Claude, Level::Out))),
        ("RedeemExpiring", Set::RedeemExpiring(Minutes(720))),
        ("RedeemUsedAtLeast", Set::RedeemUsedAtLeast(Percent(90))),
        (
            "Target",
            Set::Target(ProviderTarget {
                provider: Provider::Claude,
                target: Target::Even,
            }),
        ),
        (
            "WindowTarget",
            Set::WindowTarget(WindowTarget {
                provider: Provider::Codex,
                name: Name::FableWeek,
                target: target(),
            }),
        ),
        (
            "Program",
            Set::Program(ProgramPath {
                program: Program::Flow,
                path: path(),
            }),
        ),
        ("Credentials", Set::Credentials(path())),
    ]
}

fn produce(wire: &mut Wire) {
    use signal_energy_nexus::Level;
    for provider in [Provider::Claude, Provider::Codex] {
        case!(wire, format!("Provider.{provider:?}"), Provider, provider);
    }
    case!(wire, "Percent", Percent, Percent(63));
    case!(wire, "Minutes", Minutes, Minutes(720));
    for name in [Name::Session, Name::Week, Name::FableWeek] {
        case!(wire, format!("Name.{name:?}"), Name, name);
    }
    case!(wire, "ResetId", ResetId, ResetId("reset-a1".to_string()));
    case!(wire, "Target.Even", Target, Target::Even);
    case!(wire, "Target.SpendBy", Target, target());
    case!(wire, "Threshold", Threshold, threshold());
    case!(wire, "Reminder", Reminder, reminder(Provider::Codex, Level::Low));
    case!(wire, "Caller", Caller, caller());
    case!(
        wire,
        "ProviderTarget",
        ProviderTarget,
        ProviderTarget {
            provider: Provider::Claude,
            target: target()
        }
    );
    case!(
        wire,
        "WindowTarget",
        WindowTarget,
        WindowTarget {
            provider: Provider::Codex,
            name: Name::Week,
            target: Target::Even
        }
    );
    for program in [Program::Codex, Program::Flow] {
        case!(wire, format!("Program.{program:?}"), Program, program);
    }
    case!(wire, "Path", Path, path());
    case!(
        wire,
        "ProgramPath",
        ProgramPath,
        ProgramPath {
            program: Program::Codex,
            path: path()
        }
    );
    for (variant, set) in sets() {
        case!(wire, format!("Set.{variant}"), Set, set.clone());
        case!(
            wire,
            format!("Query.Configure.{variant}"),
            Query,
            Query::Configure(set.clone())
        );
        case!(
            wire,
            format!("Call.{variant}"),
            Call,
            Call {
                caller: caller(),
                input: Query::Configure(set.clone())
            }
        );
        case!(
            wire,
            format!("Response.Configured.{variant}"),
            Response,
            Response::Configured(set)
        );
    }
    case!(
        wire,
        "Response.ConfigureRefused",
        Response,
        Response::ConfigureRefused("the program is not installed".to_string())
    );
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
