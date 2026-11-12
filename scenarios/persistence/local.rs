use energy_nexus_wire::{Admission, Earlier, Envelope, Persisted, Upgrade};
use energy_nexus_wire::persistence::*;
use signal_energy_nexus::library::*;
use std::{cell::RefCell, fs::File, io, path::PathBuf, rc::Rc};

#[derive(Default, Debug)]
enum CandidateFault { #[default] None, Invalid, Serialization }
#[derive(Default, Debug)]
struct FixtureMemory { value: i64, fault: CandidateFault }
impl Persisted for FixtureMemory {
    const ENVELOPE: Envelope = Envelope { magic: *b"ENRSAVE1", version: 1 };
    type Uncarried = String;
    fn from_archive(bytes: &[u8]) -> Result<Self, String> {
        let value = i64::from_le_bytes(bytes.try_into().map_err(|_| "bad fixture length")?);
        Ok(Self { value, fault: CandidateFault::None })
    }
    fn upgrade(_: Earlier, _: &[u8]) -> Result<Upgrade<Self>, String> { Err("no fixture upgrade".into()) }
    fn to_archive(&self) -> Result<rkyv::util::AlignedVec, String> {
        if matches!(self.fault, CandidateFault::Serialization) { return Err("fixture serialization refused".into()); }
        let mut archive = rkyv::util::AlignedVec::new();
        archive.extend_from_slice(&self.value.to_le_bytes());
        Ok(archive)
    }
    fn validate(&self) -> Result<(), String> {
        if matches!(self.fault, CandidateFault::Invalid) { Err("fixture validation refused".into()) } else { Ok(()) }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation { Create, Write, FileSync, Rename, ParentOpen, DirectorySync }
struct InstrumentedTarget {
    real: LocalSaveTarget,
    failure: Option<Operation>,
    events: Rc<RefCell<Vec<Operation>>>,
}
struct InstrumentedTemporary {
    real: TemporaryFile,
    failure: Option<Operation>,
    events: Rc<RefCell<Vec<Operation>>>,
}
struct InstrumentedDirectory {
    real: File,
    failure: Option<Operation>,
    events: Rc<RefCell<Vec<Operation>>>,
}
trait FaultChecking {
    fn check(&self, operation: Operation) -> io::Result<()>;
}
macro_rules! fault_checking {
    ($kind:ty) => { impl FaultChecking for $kind {
        fn check(&self, operation: Operation) -> io::Result<()> {
            self.events.borrow_mut().push(operation);
            if self.failure == Some(operation) { Err(io::Error::other("injected local failure")) } else { Ok(()) }
        }
    }};
}
fault_checking!(InstrumentedTarget);
fault_checking!(InstrumentedTemporary);
fault_checking!(InstrumentedDirectory);
impl SaveTargeting for InstrumentedTarget {
    type Temporary = InstrumentedTemporary;
    type Directory = InstrumentedDirectory;
    fn create_temporary(&self, identity: &SaveId) -> io::Result<Self::Temporary> {
        self.check(Operation::Create)?;
        Ok(InstrumentedTemporary { real: self.real.create_temporary(identity)?, failure: self.failure, events: self.events.clone() })
    }
    fn replace(&self, temporary: &mut Self::Temporary) -> io::Result<()> {
        self.check(Operation::Rename)?;
        self.real.replace(&mut temporary.real)
    }
    fn open_parent(&self) -> io::Result<Self::Directory> {
        self.check(Operation::ParentOpen)?;
        Ok(InstrumentedDirectory { real: self.real.open_parent()?, failure: self.failure, events: self.events.clone() })
    }
}
impl TemporaryWriting for InstrumentedTemporary {
    fn write_contents(&mut self, bytes: &[u8]) -> io::Result<()> { self.check(Operation::Write)?; self.real.write_contents(bytes) }
    fn synchronize_contents(&self) -> io::Result<()> { self.check(Operation::FileSync)?; self.real.synchronize_contents() }
}
impl DirectorySynchronizing for InstrumentedDirectory {
    fn synchronize_directory(&self) -> io::Result<()> { self.check(Operation::DirectorySync)?; self.real.synchronize_directory() }
}

#[test]
fn all_definite_pre_rename_failures_keep_original_file_memory_and_durability() {
    let cases = [
        (None, CandidateFault::Invalid, BeforeReplacement::Validation, vec![]),
        (None, CandidateFault::Serialization, BeforeReplacement::Serialization, vec![]),
        (Some(Operation::Create), CandidateFault::None, BeforeReplacement::TemporaryCreate, vec![Operation::Create]),
        (Some(Operation::Write), CandidateFault::None, BeforeReplacement::TemporaryWrite, vec![Operation::Create, Operation::Write]),
        (Some(Operation::FileSync), CandidateFault::None, BeforeReplacement::TemporarySync, vec![Operation::Create, Operation::Write, Operation::FileSync]),
        (Some(Operation::Rename), CandidateFault::None, BeforeReplacement::Replacement, vec![Operation::Create, Operation::Write, Operation::FileSync, Operation::Rename]),
    ];
    for (failure, fault, stage, expected_events) in cases {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state");
        let target = LocalSaveTarget { path: path.clone() };
        let mut owner = LocalMemory::<FixtureMemory>::open(&path).unwrap();
        assert_eq!(owner.readback, PersistenceReadback { persistence_status: PersistenceStatus::Volatile, last_save: LastSave::Never });
        owner.commit(SaveId("seed".into()), FixtureMemory { value: 1, fault: CandidateFault::None }, &target);
        let before = std::fs::read(&path).unwrap();
        let previous_status = owner.readback.persistence_status.clone();
        let instrumented = InstrumentedTarget { real: target, failure, events: Rc::default() };
        let observation = owner.commit(SaveId("initiator-preserved".into()), FixtureMemory { value: 2, fault }, &instrumented);
        assert_eq!(observation.receipt, SaveReceipt { save_id: SaveId("initiator-preserved".into()), save_outcome: SaveOutcome::NotReplaced(stage) });
        assert!(observation.native_failure.is_some());
        assert_eq!(owner.memory.value, 1);
        assert_eq!(owner.readback.persistence_status, previous_status);
        assert_eq!(owner.readback.last_save, LastSave::SaveReceipt(observation.receipt));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(*instrumented.events.borrow(), expected_events);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1, "owned temporary cleaned");
    }
}

#[test]
fn post_rename_failures_adopt_candidate_with_uncertainty_and_restart_observes_it() {
    for (failure, stage) in [(Operation::ParentOpen, AfterReplacement::ParentOpen), (Operation::DirectorySync, AfterReplacement::ParentSync)] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state");
        let mut owner = LocalMemory::<FixtureMemory>::open(&path).unwrap();
        let target = InstrumentedTarget { real: LocalSaveTarget { path: path.clone() }, failure: Some(failure), events: Rc::default() };
        let observation = owner.commit(SaveId("same-original-key".into()), FixtureMemory { value: 42, fault: CandidateFault::None }, &target);
        assert_eq!(observation.receipt.save_id, SaveId("same-original-key".into()));
        let SaveOutcome::ReplacedDurabilityUncertain(replaced) = &observation.receipt.save_outcome else { panic!("replacement observation lost") };
        assert_eq!(replaced.after_replacement, stage);
        assert_eq!(owner.memory.value, 42);
        assert_eq!(owner.readback.persistence_status, PersistenceStatus::Unproved(replaced.stored_state_id.clone()));
        assert_eq!(owner.readback.last_save, LastSave::SaveReceipt(observation.receipt.clone()));
        let restarted = LocalMemory::<FixtureMemory>::open(&path).unwrap();
        assert_eq!(restarted.memory.value, 42);
        assert_eq!(restarted.readback.persistence_status, PersistenceStatus::Unproved(replaced.stored_state_id.clone()));
        assert_eq!(restarted.readback.last_save, LastSave::Never);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        assert!(std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "restarted_reader_fixture", "--nocapture"])
            .env("ENERGY_PERSISTENCE_FIXTURE", &path).status().unwrap().success());
    }
}

#[test]
fn restarted_reader_fixture() {
    let Some(path) = std::env::var_os("ENERGY_PERSISTENCE_FIXTURE") else { return };
    let owner = LocalMemory::<FixtureMemory>::open(&PathBuf::from(path)).unwrap();
    assert_eq!(owner.memory.value, 42);
    assert!(matches!(owner.readback.persistence_status, PersistenceStatus::Unproved(_)));
    assert_eq!(owner.readback.last_save, LastSave::Never);
}

#[test]
fn durable_requires_file_sync_then_rename_then_directory_sync_and_load_is_unproved() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state");
    let mut owner = LocalMemory::<FixtureMemory>::open(&path).unwrap();
    let target = InstrumentedTarget { real: LocalSaveTarget { path: path.clone() }, failure: None, events: Rc::default() };
    let observation = owner.commit(SaveId("supplied-by-initiator".into()), FixtureMemory { value: 42, fault: CandidateFault::None }, &target);
    let SaveOutcome::Durable(identity) = &observation.receipt.save_outcome else { panic!("expected durable receipt") };
    assert!(observation.native_failure.is_none());
    assert_eq!(identity.0.len(), 64, "SHA-256 of exact envelope and archive");
    assert_eq!(owner.readback.persistence_status, PersistenceStatus::Durable(identity.clone()));
    assert_eq!(*target.events.borrow(), [Operation::Create, Operation::Write, Operation::FileSync, Operation::Rename, Operation::ParentOpen, Operation::DirectorySync]);
    let restarted = LocalMemory::<FixtureMemory>::open(&path).unwrap();
    assert_eq!(restarted.readback.persistence_status, PersistenceStatus::Unproved(identity.clone()));
    assert_eq!(restarted.readback.last_save, LastSave::Never);
    assert_eq!(restarted.memory.value, 42);
    assert!(std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "restarted_reader_fixture", "--nocapture"])
        .env("ENERGY_PERSISTENCE_FIXTURE", &path).status().unwrap().success());
}

#[test]
fn temporary_collision_is_refused_without_removing_foreign_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state");
    let target = LocalSaveTarget { path: path.clone() };
    let identity = SaveId("already-owned".into());
    let foreign = target.create_temporary(&identity).unwrap();
    let candidate = FixtureMemory { value: 42, fault: CandidateFault::None };
    let observation = SaveAttempt { save_id: identity.clone(), candidate: &candidate, target: &target }.save();
    assert_eq!(observation.receipt, SaveReceipt { save_id: identity, save_outcome: SaveOutcome::NotReplaced(BeforeReplacement::TemporaryCreate) });
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    assert!(!path.exists());
    drop(foreign);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn invalid_or_unreadable_entries_are_not_replaced_with_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state");
    std::fs::write(&path, b"bad").unwrap();
    assert!(LocalMemory::<FixtureMemory>::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"bad");
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(directory.path().join("missing"), &path).unwrap();
    assert!(LocalMemory::<FixtureMemory>::open(&path).is_err());
    assert!(path.symlink_metadata().unwrap().file_type().is_symlink());
    assert!(matches!(FixtureMemory::admit_bytes(&path, b"bad"), Err(_)));
    let _schema_marker: Option<Admission<FixtureMemory>> = None;
}
