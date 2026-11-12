#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ProgramsMemory {
    pub configuration_revision: meta_signal_energy_nexus::library::ConfigurationRevision,
    pub programs: meta_signal_energy_nexus::library::Programs,
}
