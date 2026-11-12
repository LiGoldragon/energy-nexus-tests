#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum SyntaxProblem {
    Malformed,
    UnknownOperation,
    BudgetExceeded,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Position(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct SyntaxRefused_Data {
    pub syntax_problem: SyntaxProblem,
    pub position: Position,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum TextReply {
    Answer(String),
    SyntaxRefused(SyntaxRefused_Data),
    Failure(signal_energy_nexus::library::Failure),
}
#[rustfmt::skip]
pub trait ReadingDecodable {
    fn decode(&self) -> signal_energy_nexus::library::ReadingResult;
}
#[rustfmt::skip]
pub trait RedemptionDecodable {
    fn decode(&self) -> crate::emission_contract::ConsumeReply;
}
#[rustfmt::skip]
pub trait ProviderDemanding {
    fn demand(&self) -> crate::emission_contract::ProviderRead;
}
#[rustfmt::skip]
pub trait RedemptionDemanding {
    fn submission(&self) -> crate::emission_contract::ConsumeRequest;
}
#[rustfmt::skip]
pub trait ProviderCallable {
    fn read<N: ProviderDemanding>(
        &self,
        input: N,
    ) -> signal_energy_nexus::library::ReadingResult;
}
#[rustfmt::skip]
pub trait RedemptionCallable {
    fn consume<N: RedemptionDemanding>(
        &self,
        input: N,
    ) -> crate::emission_contract::ConsumeReply;
}
#[rustfmt::skip]
pub trait ConfigurationReadable {
    fn configuration(&self) -> meta_signal_energy_nexus::library::ConfigurationSnapshot;
}
#[rustfmt::skip]
pub trait ProgramsChanging {
    fn change(&self) -> meta_signal_energy_nexus::library::ProgramChange;
}
#[rustfmt::skip]
pub trait ProgramsConfigurable {
    fn configure_programs<N: ProgramsChanging>(
        &mut self,
        input: N,
    ) -> std::result::Result<
        meta_signal_energy_nexus::library::TranslatorConfiguration,
        signal_energy_nexus::library::Failure,
    >;
}
#[rustfmt::skip]
pub trait MetaRequesting {
    fn call(&self) -> meta_signal_energy_nexus::Call;
}
#[rustfmt::skip]
pub trait MetaAnswerable {
    fn answer_meta<N: MetaRequesting>(
        &mut self,
        input: N,
    ) -> meta_signal_energy_nexus::Response;
}
#[rustfmt::skip]
pub trait TextRequesting {
    fn text(&self) -> String;
    fn caller(&self) -> signal_energy_nexus::library::Caller;
}
#[rustfmt::skip]
pub trait TextAnswerable {
    fn answer_text<N: TextRequesting>(&mut self, input: N) -> TextReply;
}
#[rustfmt::skip]
const _: () = {
    fn assert_providerdocument_readingdecodable<T: ReadingDecodable>() {}
    let _ = assert_providerdocument_readingdecodable::<crate::ProviderDocument>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_resetreply_redemptiondecodable<T: RedemptionDecodable>() {}
    let _ = assert_resetreply_redemptiondecodable::<crate::ResetReply>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_providerread_providerdemanding<T: ProviderDemanding>() {}
    let _ = assert_providerread_providerdemanding::<
        crate::emission_contract::ProviderRead,
    >;
};
#[rustfmt::skip]
const _: () = {
    fn assert_consumerequest_redemptiondemanding<T: RedemptionDemanding>() {}
    let _ = assert_consumerequest_redemptiondemanding::<
        crate::emission_contract::ConsumeRequest,
    >;
};
#[rustfmt::skip]
const _: () = {
    fn assert_programchange_programschanging<T: ProgramsChanging>() {}
    let _ = assert_programchange_programschanging::<
        meta_signal_energy_nexus::library::ProgramChange,
    >;
};
#[rustfmt::skip]
const _: () = {
    fn assert_call_metarequesting<T: MetaRequesting>() {}
    let _ = assert_call_metarequesting::<meta_signal_energy_nexus::Call>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_textrequest_textrequesting<T: TextRequesting>() {}
    let _ = assert_textrequest_textrequesting::<crate::TextRequest>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_translator_providercallable<T: ProviderCallable>() {}
    let _ = assert_translator_providercallable::<crate::Translator>;
    fn assert_translator_redemptioncallable<T: RedemptionCallable>() {}
    let _ = assert_translator_redemptioncallable::<crate::Translator>;
    fn assert_translator_configurationreadable<T: ConfigurationReadable>() {}
    let _ = assert_translator_configurationreadable::<crate::Translator>;
    fn assert_translator_programsconfigurable<T: ProgramsConfigurable>() {}
    let _ = assert_translator_programsconfigurable::<crate::Translator>;
    fn assert_translator_metaanswerable<T: MetaAnswerable>() {}
    let _ = assert_translator_metaanswerable::<crate::Translator>;
    fn assert_translator_textanswerable<T: TextAnswerable>() {}
    let _ = assert_translator_textanswerable::<crate::Translator>;
};
#[rustfmt::skip]
const _: () = {
    fn assert_attemptreporter_attemptreportable<T: crate::telemetry::AttemptReportable>() {}
    let _ = assert_attemptreporter_attemptreportable::<crate::AttemptReporter>;
};
