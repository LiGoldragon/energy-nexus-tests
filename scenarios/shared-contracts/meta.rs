use meta_signal_energy_nexus::{internal, library::*, Response};
use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_energy_nexus::library as domain;

macro_rules! roundtrip {
    ($name:ident, $kind:ty, $value:expr) => {
        #[test]
        fn $name() {
            let value: $kind = $value;
            let bytes = value.signalize().unwrap();
            let restored = Signal::<$kind>::from(bytes.bytes().to_vec())
                .restore()
                .unwrap();
            assert_eq!(restored, value);
            #[cfg(feature = "datom")]
            {
                use datom_codec::{Actualizing, Budget, Datomizable, Potential};
                use protos::{Compactable, Protosizable, ReaderBudget};
                let text = value.datomize(Vec::new()).protosize().compact();
                let mut budget = Budget {
                    remaining: 4096,
                    reader: ReaderBudget::try_from(4096).unwrap(),
                    depth: 0,
                    maximum_depth: 64,
                };
                let restored = Potential::<$kind>::from(text.as_str())
                    .actualize(&mut budget)
                    .unwrap();
                assert_eq!(restored, value);
            }
        }
    };
}
roundtrip!(
    policy_threshold,
    PolicySet,
    PolicySet::Threshold(domain::Threshold {
        level: domain::Level::Fair,
        left: domain::Left(27)
    })
);
roundtrip!(
    nexus_part_failure,
    NexusPart,
    NexusPart::Failure(domain::Failure::ResponseMalformed)
);
roundtrip!(
    translator_part_failure,
    TranslatorPart,
    TranslatorPart::Failure(domain::Failure::ResponseMalformed)
);
roundtrip!(
    public_failure,
    Response,
    Response::Refused(domain::Failure::ResponseMalformed)
);
roundtrip!(
    internal_failure,
    internal::Response,
    internal::Response::Refused(domain::Failure::ResponseMalformed)
);
roundtrip!(
    public_configuration,
    meta_signal_energy_nexus::Query,
    meta_signal_energy_nexus::Query::Configuration
);
roundtrip!(
    internal_configuration,
    internal::Query,
    internal::Query::Configuration
);

#[test]
fn generated_imported_variants_have_payload_shapes() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/library.rs"));
    assert!(source.contains("Threshold(signal_energy_nexus::library::Threshold),"));
    assert_eq!(
        source
            .matches("Failure(signal_energy_nexus::library::Failure),")
            .count(),
        2
    );
    assert!(source.contains("pub struct RedeemExpiring(pub i64);"));
    assert!(source.contains("pub struct RedeemUsedAtLeast(pub i64);"));
}

#[test]
fn identities_cover_manifests_and_libraries_and_distinguish_roots() {
    use signal::Contracted;
    let public = <meta_signal_energy_nexus::Call as Contracted>::CONTRACT_SOURCE;
    let internal = <internal::Call as Contracted>::CONTRACT_SOURCE;
    let ordinary = <signal_energy_nexus::Call as Contracted>::CONTRACT_SOURCE;
    assert_ne!(public, internal);
    assert_ne!(public, ordinary);
    assert_ne!(internal, ordinary);
    for source in [public, internal, ordinary] {
        assert!(source.contains("version = \"0.7.0\""));
        assert!(source.contains("Library\n"));
        assert!(source.contains("Signal\n"));
    }
}
