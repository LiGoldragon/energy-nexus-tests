#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct LatestAttempt {
    pub provider: signal_energy_nexus::library::Provider,
    pub attempt_id: signal_energy_nexus::library::AttemptId,
    pub at: signal_energy_nexus::library::At,
    pub read_outcome: signal_energy_nexus::library::ReadOutcome,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ReadingReference(pub String);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct LatestSuccess {
    pub provider: signal_energy_nexus::library::Provider,
    pub reading_reference: ReadingReference,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct FormatVersion(pub i64);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct ArchiveBytes(pub std::vec::Vec<i64>);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct LegacyArchive {
    pub format_version: FormatVersion,
    pub archive_bytes: ArchiveBytes,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct Memory {
    pub latest_attempt_vector: std::vec::Vec<LatestAttempt>,
    pub reading_vector: std::vec::Vec<signal_energy_nexus::library::Reading>,
    pub latest_success_vector: std::vec::Vec<LatestSuccess>,
    pub policy_configuration: meta_signal_energy_nexus::library::PolicyConfiguration,
    pub configuration_revision: meta_signal_energy_nexus::library::ConfigurationRevision,
    pub redemption_vector: std::vec::Vec<signal_energy_nexus::library::Redemption>,
    pub legacy_archive_vector: std::vec::Vec<LegacyArchive>,
}
