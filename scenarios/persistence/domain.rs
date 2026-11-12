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
fn every_save_stage_identity_and_remote_ack_payload_roundtrips() {
    for stage in [BeforeReplacement::Validation, BeforeReplacement::Serialization,
        BeforeReplacement::TemporaryCreate, BeforeReplacement::TemporaryWrite,
        BeforeReplacement::TemporarySync, BeforeReplacement::Replacement] {
        let receipt = SaveReceipt { save_id: SaveId("initiator-kept".into()), save_outcome: SaveOutcome::NotReplaced(stage) };
        roundtrip!(SaveReceipt, receipt.clone());
        roundtrip!(SaveAcknowledgement, SaveAcknowledgement::SaveReceipt(receipt));
    }
    for stage in [AfterReplacement::ParentOpen, AfterReplacement::ParentSync] {
        let receipt = SaveReceipt { save_id: SaveId("initiator-kept".into()), save_outcome: SaveOutcome::ReplacedDurabilityUncertain(ReplacedDurabilityUncertain_Data {
            stored_state_id: StoredStateId("candidate-bytes".into()), after_replacement: stage,
        }) };
        roundtrip!(SaveReceipt, receipt.clone());
        roundtrip!(LastSave, LastSave::SaveReceipt(receipt));
    }
    roundtrip!(SaveAcknowledgement, SaveAcknowledgement::Unconfirmed(Unconfirmed_Data {
        save_id: SaveId("initiator-kept".into()), failure: Failure::Transport(Transport_Data::EndOfStream),
    }));
    roundtrip!(SaveOutcome, SaveOutcome::Durable(StoredStateId("synced-bytes".into())));
}

#[test]
fn every_current_state_status_and_last_attempt_shape_roundtrips() {
    for status in [PersistenceStatus::Durable(StoredStateId("durable".into())),
        PersistenceStatus::Unproved(StoredStateId("loaded-or-replaced".into())), PersistenceStatus::Volatile] {
        roundtrip!(PersistenceReadback, PersistenceReadback { persistence_status: status, last_save: LastSave::Never });
    }
    roundtrip!(PersistenceReadback, PersistenceReadback {
        persistence_status: PersistenceStatus::Durable(StoredStateId("prior".into())),
        last_save: LastSave::SaveReceipt(SaveReceipt { save_id: SaveId("failed-new".into()), save_outcome: SaveOutcome::NotReplaced(BeforeReplacement::TemporarySync) }),
    });
}

#[test]
fn every_redemption_projection_preserves_record_candidate_and_save_evidence() {
    let reset = Reset { reset_id: ResetId("selected".into()), reset_status: ResetStatus::Available,
        reset_kind: ResetKind::RateLimits, expiry: Expiry::NoExpiry, granted: Granted::Unknown };
    for state in [RedemptionState::Prepared, RedemptionState::Submitted,
        RedemptionState::Uncertain(Uncertain { failure: Failure::ResponseMalformed, at: At(17) }),
        RedemptionState::RedemptionOutcome(RedemptionOutcome::ResetApplied),
        RedemptionState::RedemptionOutcome(RedemptionOutcome::NothingToReset),
        RedemptionState::RedemptionOutcome(RedemptionOutcome::NoCredit),
        RedemptionState::RedemptionOutcome(RedemptionOutcome::AlreadyRedeemed)] {
        let candidate = Redemption { redemption_id: RedemptionId("one-key".into()), reset: reset.clone(), at: At(18), redemption_state: state };
        roundtrip!(RedemptionResult, RedemptionResult::Current(Current_Data {
            redemption: candidate.clone(), persistence_readback: PersistenceReadback {
                persistence_status: PersistenceStatus::Unproved(StoredStateId("candidate".into())), last_save: LastSave::Never,
            },
        }));
        for active in [ActiveRedemption::Absent, ActiveRedemption::Redemption(Redemption {
            redemption_state: RedemptionState::Submitted, ..candidate.clone()
        })] {
            roundtrip!(RedemptionResult, RedemptionResult::NotCommitted(NotCommitted_Data {
                active_redemption: active, redemption_transition: RedemptionTransition { redemption: candidate.clone(), save_receipt: SaveReceipt {
                    save_id: SaveId("completion-save".into()), save_outcome: SaveOutcome::NotReplaced(BeforeReplacement::Replacement),
                } },
            }));
        }
    }
    roundtrip!(RedemptionResult, RedemptionResult::Failure(Failure::Storage(Storage_Data::Unavailable)));
}
