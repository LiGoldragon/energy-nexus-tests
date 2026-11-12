use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_energy_nexus::{library::*, Refused_Data, Response};

#[test]
fn imported_failure_payload_survives_binary_roundtrip() {
    let value = Response::Refused(Refused_Data::Failure(Failure::ResponseMalformed));
    let bytes = value.signalize().unwrap();
    let restored = Signal::<Response>::from(bytes.bytes().to_vec())
        .restore()
        .unwrap();
    assert_eq!(restored, value);
    assert!(matches!(
        restored,
        Response::Refused(Refused_Data::Failure(Failure::ResponseMalformed))
    ));
}

#[cfg(feature = "datom")]
#[test]
fn imported_failure_payload_survives_datom_roundtrip() {
    use datom_codec::{Actualizing, Budget, Datomizable, Potential};
    use protos::{Compactable, Protosizable, ReaderBudget};
    let value = Response::Refused(Refused_Data::Failure(Failure::ResponseMalformed));
    let text = value.datomize(Vec::new()).protosize().compact();
    assert_eq!(text, "Refused.Failure.ResponseMalformed");
    let mut budget = Budget {
        remaining: 256,
        reader: ReaderBudget::try_from(256).unwrap(),
        depth: 0,
        maximum_depth: 32,
    };
    let restored = Potential::<Response>::from(text.as_str())
        .actualize(&mut budget)
        .unwrap();
    assert_eq!(restored, value);
}

#[test]
fn semantic_scalars_are_direct_and_rate_exceeds_one_hundred() {
    let value = Pace::Calculated(Calculated_Data {
        provider: Provider::Codex,
        name: Name::Week,
        target: Target::Even,
        expected: Expected(50),
        used: Used(75),
        standing: Standing::Over,
        per_day: PerDay(600),
    });
    let bytes = value.signalize().unwrap();
    let restored = Signal::<Pace>::from(bytes.bytes().to_vec())
        .restore()
        .unwrap();
    assert_eq!(restored, value);
    let timestamp = At(123);
    assert_eq!(timestamp.0, 123);
}

#[test]
fn generated_ordinary_payload_and_scalar_shapes_are_explicit() {
    let contract = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/signal.rs"));
    assert!(contract.contains("Failure(crate::library::Failure),"));
    let library = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/library.rs"));
    for scalar in ["At", "Expected", "Used", "PerDay", "Left"] {
        assert!(library.contains(&format!("pub struct {scalar}(pub i64);")));
    }
}
