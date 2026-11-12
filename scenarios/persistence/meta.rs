use meta_signal_energy_nexus::{internal, library::*, Query, Response};
use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_energy_nexus::library::*;

macro_rules! roundtrip {
    ($kind:ty, $value:expr) => {{
        let value: $kind = $value;
        let bytes = value.signalize().unwrap();
        assert_eq!(Signal::<$kind>::from(bytes.bytes().to_vec()).restore().unwrap(), value);
        #[cfg(feature = "datom")]
        {
            use datom_codec::{Actualizing, Budget, Datomizable, Potential};
            use protos::{Compactable, Protosizable, ReaderBudget};
            let text = value.datomize(Vec::new()).protosize().compact();
            let mut budget = Budget { remaining: 8192, reader: ReaderBudget::try_from(8192).unwrap(), depth: 0, maximum_depth: 64 };
            assert_eq!(Potential::<$kind>::from(text.as_str()).actualize(&mut budget).unwrap(), value);
        }
    }};
}

#[test]
fn configuration_readback_keeps_values_revisions_and_durability_per_owner() {
    let readback = PersistenceReadback { persistence_status: PersistenceStatus::Unproved(StoredStateId("actual-store".into())), last_save: LastSave::Never };
    let snapshot = ConfigurationSnapshot {
        nexus_part: NexusPart::NexusConfiguration(NexusConfiguration { configuration_revision: ConfigurationRevision(31),
            policy_configuration: PolicyConfiguration { threshold_vector: vec![], reminder_vector: vec![], redeem_expiring: RedeemExpiring(90),
                redeem_used_at_least: RedeemUsedAtLeast(95), provider_target_vector: vec![], window_target_vector: vec![] },
            persistence_readback: readback.clone(),
        }),
        translator_part: TranslatorPart::TranslatorConfiguration(TranslatorConfiguration { configuration_revision: ConfigurationRevision(52),
            programs: Programs { codex_program: CodexProgram::Path(Path("/declared/codex".into())), flow_program: FlowProgram::Unconfigured,
                credential_reference: CredentialReference::Unconfigured }, persistence_readback: readback,
        }),
    };
    roundtrip!(Response, Response::Configuration(snapshot.clone()));
    roundtrip!(internal::Response, internal::Response::Configuration(match snapshot.nexus_part { NexusPart::NexusConfiguration(value) => value, _ => unreachable!() }));
    roundtrip!(Response, Response::Configuration(ConfigurationSnapshot { nexus_part: NexusPart::Failure(Failure::Transport(Transport_Data::Timeout)), ..snapshot }));
}

#[test]
fn initiating_save_id_and_every_configure_receipt_shape_survive_both_edges() {
    let policy = PolicySet::RedeemExpiring(RedeemExpiring(89));
    roundtrip!(internal::Query, internal::Query::Configure(PolicyUpdate { save_id: SaveId("same-initiator".into()), policy_set: policy.clone() }));
    roundtrip!(Query, Query::Configure(ConfigurationChange::PolicySet(policy.clone())));
    for ack in [
        SaveAcknowledgement::SaveReceipt(SaveReceipt { save_id: SaveId("same-initiator".into()), save_outcome: SaveOutcome::NotReplaced(BeforeReplacement::Validation) }),
        SaveAcknowledgement::SaveReceipt(SaveReceipt { save_id: SaveId("same-initiator".into()), save_outcome: SaveOutcome::ReplacedDurabilityUncertain(ReplacedDurabilityUncertain_Data {
            stored_state_id: StoredStateId("current-not-durable".into()), after_replacement: AfterReplacement::ParentSync,
        }) }),
        SaveAcknowledgement::SaveReceipt(SaveReceipt { save_id: SaveId("same-initiator".into()), save_outcome: SaveOutcome::Durable(StoredStateId("durable".into())) }),
        SaveAcknowledgement::Unconfirmed(Unconfirmed_Data { save_id: SaveId("same-initiator".into()), failure: Failure::Transport(Transport_Data::EndOfStream) }),
    ] {
        let update = ConfigurationUpdate { configuration_change: ConfigurationChange::PolicySet(policy.clone()), save_acknowledgement: ack.clone() };
        roundtrip!(Response, Response::ConfigureResult(update.clone()));
        roundtrip!(internal::Response, internal::Response::ConfigureResult(update));
        roundtrip!(Response, Response::ConfigureResult(ConfigurationUpdate {
            configuration_change: ConfigurationChange::ProgramChange(ProgramChange::ProgramPath(ProgramPath { program: Program::Codex, path: Path("/configured/codex".into()) })),
            save_acknowledgement: ack,
        }));
    }
}
