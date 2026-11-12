#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum DemandReceipt {
    Counted,
    ReportingFailure(crate::telemetry::ReportingFailure),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum CommitReceipt {
    Saved,
    Failure(signal_energy_nexus::library::Failure),
}
#[rustfmt::skip]
pub trait ReadDemanding {
    fn context(&self) -> crate::telemetry::AttemptContext;
    fn provider(&self) -> signal_energy_nexus::library::Provider;
}
#[rustfmt::skip]
pub trait ReadAttemptable {
    fn attempt_read<N: ReadDemanding>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::Observation;
}
#[rustfmt::skip]
pub trait RedemptionDemanding {
    fn provider(&self) -> signal_energy_nexus::library::Provider;
}
#[rustfmt::skip]
pub trait RedemptionAttemptable {
    fn attempt_redemption<N: RedemptionDemanding>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::RedemptionResult;
}
#[rustfmt::skip]
pub trait EmissionRequesting {
    fn emission(&self) -> crate::emission_contract::Emission;
}
#[rustfmt::skip]
pub trait EmissionCallable {
    fn emit<N: EmissionRequesting>(
        &self,
        input: N,
    ) -> std::result::Result<
        crate::emission_contract::EmissionOutcome,
        signal_energy_nexus::library::Failure,
    >;
}
#[rustfmt::skip]
pub trait MemoryChanging {
    fn candidate(&self) -> crate::memory_contract::Memory;
}
#[rustfmt::skip]
pub trait MemoryCommittable {
    fn commit<N: MemoryChanging>(&mut self, input: N) -> CommitReceipt;
}
#[rustfmt::skip]
pub trait ObservationChanging {
    fn completion(&self) -> crate::ReadCompletion;
}
#[rustfmt::skip]
pub trait ObservationCommittable {
    fn record<N: ObservationChanging>(
        &mut self,
        input: N,
    ) -> signal_energy_nexus::library::Observation;
}
#[rustfmt::skip]
pub trait PolicyRequesting {
    fn call(&self) -> meta_signal_energy_nexus::internal::Call;
}
#[rustfmt::skip]
pub trait PolicyAnswerable {
    fn answer_policy<N: PolicyRequesting>(
        &mut self,
        input: N,
    ) -> meta_signal_energy_nexus::internal::Response;
}
#[rustfmt::skip]
pub trait DemandDescribing {
    fn trigger(&self) -> crate::telemetry::Trigger;
}
#[rustfmt::skip]
pub trait DemandAccountable {
    fn account<N: DemandDescribing>(&mut self, input: N) -> DemandReceipt;
}
#[rustfmt::skip]
const _: () = {
    fn assert_readrequest_readdemanding<T: ReadDemanding>() {}
    let _ = assert_readrequest_readdemanding::<crate::ReadRequest>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_redemptionrequest_redemptiondemanding<T: RedemptionDemanding>() {}
    let _ = assert_redemptionrequest_redemptiondemanding::<crate::RedemptionRequest>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_emission_emissionrequesting<T: EmissionRequesting>() {}
    let _ = assert_emission_emissionrequesting::<crate::emission_contract::Emission>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_memory_memorychanging<T: MemoryChanging>() {}
    let _ = assert_memory_memorychanging::<crate::memory_contract::Memory>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_readcompletion_observationchanging<T: ObservationChanging>() {}
    let _ = assert_readcompletion_observationchanging::<crate::ReadCompletion>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_call_policyrequesting<T: PolicyRequesting>() {}
    let _ = assert_call_policyrequesting::<meta_signal_energy_nexus::internal::Call>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_trigger_demanddescribing<T: DemandDescribing>() {}
    let _ = assert_trigger_demanddescribing::<crate::telemetry::Trigger>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_nexus_readattemptable<T: ReadAttemptable>() {}
    let _ = assert_nexus_readattemptable::<crate::Nexus>;
    fn assert_nexus_redemptionattemptable<T: RedemptionAttemptable>() {}
    let _ = assert_nexus_redemptionattemptable::<crate::Nexus>;
    fn assert_nexus_emissioncallable<T: EmissionCallable>() {}
    let _ = assert_nexus_emissioncallable::<crate::Nexus>;
    fn assert_nexus_memorycommittable<T: MemoryCommittable>() {}
    let _ = assert_nexus_memorycommittable::<crate::Nexus>;
    fn assert_nexus_observationcommittable<T: ObservationCommittable>() {}
    let _ = assert_nexus_observationcommittable::<crate::Nexus>;
    fn assert_nexus_policyanswerable<T: PolicyAnswerable>() {}
    let _ = assert_nexus_policyanswerable::<crate::Nexus>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_track_demandaccountable<T: DemandAccountable>() {}
    let _ = assert_track_demandaccountable::<crate::schedule::Track>;
};
