#![allow(dead_code, unused_imports, unused_variables)]
extern crate self as meta_signal_energy_nexus;
use signal_energy_nexus::library as domain;
use signal::{ByteViewable, Restorable, Signal, Signalizable};
#[path = "generated/library.rs"] pub mod library;
#[path = "generated/public.rs"] pub mod public;
#[path = "generated/internal.rs"] pub mod internal;
#[path = "generated/memory_contract.rs"] pub mod memory_contract;
#[path = "generated/programs_contract.rs"] pub mod programs_contract;
#[path = "generated/telemetry.rs"] pub mod telemetry;
#[path = "generated/emission_contract.rs"] pub mod emission_contract;
#[path = "generated/selection.rs"] pub mod selection;
#[path = "generated/translator_runtime.rs"] pub mod translator_runtime;
#[path = "generated/daemon_runtime.rs"] pub mod daemon_runtime;
pub use public::{Call, Response};
pub mod schedule { pub struct Track { pub marker: String } }
pub struct Translator { pub marker: String }
pub struct ProviderDocument { pub marker: String }
pub struct ResetReply { pub marker: String }
pub struct AttemptReporter { pub marker: String }
pub struct TextRequest { pub marker: String }
pub struct Nexus { pub marker: String }
pub struct ReadCompletion { pub marker: String }
pub struct ReadRequest { pub marker: String }
pub struct RedemptionRequest { pub marker: String }
mod telemetry_compile_only { use super::telemetry::*;impl AttemptReporting for AttemptRecord {
    fn record(&self) -> AttemptRecord { panic!("compile-only test double was invoked") }
}
}
mod selection_compile_only { use super::selection::*;impl CreditSelectable for CreditSelection {
    fn select(&self) -> SelectionResult { panic!("compile-only test double was invoked") }
}
impl Levellable for LevelCalculation {
    fn level(&self) -> signal_energy_nexus::library::Level { panic!("compile-only test double was invoked") }
}
impl PaceCalculable for PaceCalculation {
    fn pace(&self) -> signal_energy_nexus::library::Pace { panic!("compile-only test double was invoked") }
}
}
mod translator_runtime_compile_only { use super::translator_runtime::*; use super::telemetry::{AttemptReporting, ReportResult};impl ReadingDecodable for crate::ProviderDocument {
    fn decode(&self) -> signal_energy_nexus::library::ReadingResult { panic!("compile-only test double was invoked") }
}
impl RedemptionDecodable for crate::ResetReply {
    fn decode(&self) -> crate::emission_contract::ConsumeReply { panic!("compile-only test double was invoked") }
}
impl ProviderDemanding for crate::emission_contract::ProviderRead {
    fn demand(&self) -> crate::emission_contract::ProviderRead { panic!("compile-only test double was invoked") }
}
impl RedemptionDemanding for crate::emission_contract::ConsumeRequest {
    fn submission(&self) -> crate::emission_contract::ConsumeRequest { panic!("compile-only test double was invoked") }
}
impl ProgramsChanging for meta_signal_energy_nexus::library::ProgramChange {
    fn change(&self) -> meta_signal_energy_nexus::library::ProgramChange { panic!("compile-only test double was invoked") }
}
impl MetaRequesting for meta_signal_energy_nexus::Call {
    fn call(&self) -> meta_signal_energy_nexus::Call { panic!("compile-only test double was invoked") }
}
impl TextRequesting for crate::TextRequest {
    fn text(&self) -> String { panic!("compile-only test double was invoked") }
    fn caller(&self) -> signal_energy_nexus::library::Caller { panic!("compile-only test double was invoked") }
}
impl ProviderCallable for crate::Translator {
    fn read<N: ProviderDemanding>(
        &self,
        input: N,
    ) -> signal_energy_nexus::library::ReadingResult { panic!("compile-only test double was invoked") }
}
impl RedemptionCallable for crate::Translator {
    fn consume<N: RedemptionDemanding>(
        &self,
        input: N,
    ) -> crate::emission_contract::ConsumeReply { panic!("compile-only test double was invoked") }
}
impl ConfigurationReadable for crate::Translator {
    fn configuration(&self) -> meta_signal_energy_nexus::library::ConfigurationSnapshot { panic!("compile-only test double was invoked") }
}
impl ProgramsConfigurable for crate::Translator {
    fn configure_programs<N: ProgramsChanging>(
        &mut self,
        input: N,
    ) -> std::result::Result<
        meta_signal_energy_nexus::library::TranslatorConfiguration,
        signal_energy_nexus::library::Failure,
    > { panic!("compile-only test double was invoked") }
}
impl MetaAnswerable for crate::Translator {
    fn answer_meta<N: MetaRequesting>(
        &mut self,
        input: N,
    ) -> meta_signal_energy_nexus::Response { panic!("compile-only test double was invoked") }
}
impl TextAnswerable for crate::Translator {
    fn answer_text<N: TextRequesting>(&mut self, input: N) -> TextReply { panic!("compile-only test double was invoked") }
}
impl crate::telemetry::AttemptReportable for crate::AttemptReporter {
    fn report<N: AttemptReporting>(&mut self, input: N) -> ReportResult { panic!("compile-only test double was invoked") }
}
}
mod daemon_runtime_compile_only { use super::daemon_runtime::*;impl ReadDemanding for crate::ReadRequest {
    fn context(&self) -> crate::telemetry::AttemptContext { panic!("compile-only test double was invoked") }
    fn provider(&self) -> signal_energy_nexus::library::Provider { panic!("compile-only test double was invoked") }
}
impl RedemptionDemanding for crate::RedemptionRequest {
    fn provider(&self) -> signal_energy_nexus::library::Provider { panic!("compile-only test double was invoked") }
}
impl EmissionRequesting for crate::emission_contract::Emission {
    fn emission(&self) -> crate::emission_contract::Emission { panic!("compile-only test double was invoked") }
}
impl MemoryChanging for crate::memory_contract::Memory {
    fn candidate(&self) -> crate::memory_contract::Memory { panic!("compile-only test double was invoked") }
}
impl ObservationChanging for crate::ReadCompletion {
    fn completion(&self) -> crate::ReadCompletion { panic!("compile-only test double was invoked") }
}
impl PolicyRequesting for meta_signal_energy_nexus::internal::Call {
    fn call(&self) -> meta_signal_energy_nexus::internal::Call { panic!("compile-only test double was invoked") }
}
impl DemandDescribing for crate::telemetry::Trigger {
    fn trigger(&self) -> crate::telemetry::Trigger { panic!("compile-only test double was invoked") }
}
impl ReadAttemptable for crate::Nexus {
    fn attempt_read<N: ReadDemanding>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::Observation { panic!("compile-only test double was invoked") }
}
impl RedemptionAttemptable for crate::Nexus {
    fn attempt_redemption<N: RedemptionDemanding>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::RedemptionResult { panic!("compile-only test double was invoked") }
}
impl EmissionCallable for crate::Nexus {
    fn emit<N: EmissionRequesting>(
        &self,
        input: N,
    ) -> std::result::Result<
        crate::emission_contract::EmissionOutcome,
        signal_energy_nexus::library::Failure,
    > { panic!("compile-only test double was invoked") }
}
impl MemoryCommittable for crate::Nexus {
    fn commit<N: MemoryChanging>(&mut self, input: N) -> CommitReceipt { panic!("compile-only test double was invoked") }
}
impl ObservationCommittable for crate::Nexus {
    fn record<N: ObservationChanging>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::Observation { panic!("compile-only test double was invoked") }
}
impl PolicyAnswerable for crate::Nexus {
    fn answer_policy<N: PolicyRequesting>(
        &mut self,
        input: N,
    ) -> meta_signal_energy_nexus::internal::Response { panic!("compile-only test double was invoked") }
}
impl DemandAccountable for crate::schedule::Track {
    fn account<N: DemandDescribing>(&mut self, input: N) -> DemandReceipt { panic!("compile-only test double was invoked") }
}
}
macro_rules! roundtrip {
    ($name:ident, $kind:ty, $value:expr) => {
        #[test]
        fn $name() {
            let value: $kind = $value;
            let bytes = value.signalize().unwrap();
            let restored = Signal::<$kind>::from(bytes.bytes().to_vec()).restore().unwrap();
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
                let restored = Potential::<$kind>::from(text.as_str()).actualize(&mut budget).unwrap();
                assert_eq!(restored, value);
            }
        }
    };
}
roundtrip!(daemon_reporting_failure, daemon_runtime::DemandReceipt,
    daemon_runtime::DemandReceipt::ReportingFailure(telemetry::ReportingFailure::CountOverflow));
roundtrip!(daemon_commit_failure, daemon_runtime::CommitReceipt,
    daemon_runtime::CommitReceipt::Failure(domain::Failure::ResponseMalformed));
roundtrip!(emission_reading_result, emission_contract::EmissionOutcome,
    emission_contract::EmissionOutcome::ReadingResult(domain::ReadingResult::Failure(domain::Failure::ResponseMalformed)));
roundtrip!(emission_failure, emission_contract::EmissionOutcome,
    emission_contract::EmissionOutcome::Failure(domain::Failure::ResponseMalformed));
roundtrip!(selection_week_percent, selection::WeekUsage,
    selection::WeekUsage::Percent(domain::Percent(87)));
roundtrip!(selection_selected_reset, selection::SelectionResult,
    selection::SelectionResult::Reset(domain::Reset {
        reset_id: domain::ResetId("selected-credit".into()),
        reset_status: domain::ResetStatus::Available,
        reset_kind: domain::ResetKind::RateLimits,
        expiry: domain::Expiry::Instant(domain::Instant(123456)),
        granted: domain::Granted::Instant(domain::Instant(120000)),
    }));
roundtrip!(telemetry_failure, telemetry::AttemptOutcome,
    telemetry::AttemptOutcome::Failure(domain::Failure::ResponseMalformed));
roundtrip!(telemetry_redemption_result, telemetry::AttemptOutcome,
    telemetry::AttemptOutcome::RedemptionResult(domain::RedemptionResult::Failure(domain::Failure::ResponseMalformed)));
roundtrip!(translator_text_failure, translator_runtime::TextReply,
    translator_runtime::TextReply::Failure(domain::Failure::ResponseMalformed));
roundtrip!(meta_policy_threshold, library::PolicySet,
    library::PolicySet::Threshold(domain::Threshold { level: domain::Level::Fair, left: domain::Left(27) }));
roundtrip!(meta_nexus_failure, library::NexusPart,
    library::NexusPart::Failure(domain::Failure::ResponseMalformed));
roundtrip!(meta_translator_failure, library::TranslatorPart,
    library::TranslatorPart::Failure(domain::Failure::ResponseMalformed));
roundtrip!(ordinary_refusal_failure, signal_energy_nexus::Response,
    signal_energy_nexus::Response::Refused(signal_energy_nexus::Refused_Data::Failure(domain::Failure::ResponseMalformed)));
roundtrip!(public_meta_failure, Response, Response::Refused(domain::Failure::ResponseMalformed));
roundtrip!(internal_meta_failure, internal::Response, internal::Response::Refused(domain::Failure::ResponseMalformed));
