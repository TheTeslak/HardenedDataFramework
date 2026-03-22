use hdf_core::SuspectReason;
use hdf_journal::JournalEvent;
use hdf_reference::{ControlConfig, ControlMode, CycleOutcome, ReferenceApp};
use hdf_storage::{LoadReason, LoadReport, PersistentRecord, SlotId, next_record};

fn cfg(mode: ControlMode, threshold: u16, revision: u8, output_enabled: bool) -> ControlConfig {
    ControlConfig::new(mode, threshold, revision, output_enabled)
}

#[test]
fn boot_selects_newer_record_and_logs_storage_selection() {
    let primary = PersistentRecord::new(4, cfg(ControlMode::Standby, 900, 7, true));
    let secondary = next_record(4, cfg(ControlMode::Active, 950, 8, true));

    let app = ReferenceApp::<16>::boot(primary, secondary).expect("valid boot path");

    assert_eq!(
        app.boot_report(),
        LoadReport::Trusted {
            value: secondary.value(),
            version: 5,
            source: SlotId::Secondary,
            reason: LoadReason::SecondaryNewer,
        }
    );
    assert_eq!(
        app.journal_record(0).unwrap().event,
        JournalEvent::ResetObserved {
            cause: hdf_journal::ResetCause::PowerOn,
        }
    );
    assert_eq!(
        app.journal_record(1).unwrap().event,
        JournalEvent::StorageRecordSelected {
            source: hdf_journal::StorageSource::Secondary,
            version: 5,
        }
    );
}

#[test]
fn recoverable_runtime_fault_repairs_and_stays_operational() {
    let primary = PersistentRecord::new(1, cfg(ControlMode::Active, 1000, 1, true));
    let secondary = PersistentRecord::new(1, cfg(ControlMode::Active, 1000, 1, true));
    let mut app = ReferenceApp::<16>::boot(primary, secondary).expect("valid boot path");

    let outlier = cfg(ControlMode::Safe, 1000, 2, false);
    app.inject_outlier(2, outlier);

    assert_eq!(
        app.step(),
        CycleOutcome::Recovered {
            config: cfg(ControlMode::Active, 1000, 1, true),
        }
    );
    assert!(app.outputs_enabled());
    assert_eq!(
        app.runtime_snapshot(),
        [cfg(ControlMode::Active, 1000, 1, true); 3]
    );
    assert_eq!(
        app.journal_record(3).unwrap().event,
        JournalEvent::RecoverableMismatch
    );
    assert_eq!(
        app.journal_record(4).unwrap().event,
        JournalEvent::RepairAttempted
    );
    assert_eq!(
        app.journal_record(5).unwrap().event,
        JournalEvent::RepairSucceeded
    );
}

#[test]
fn suspect_runtime_fault_forces_safe_behavior() {
    let primary = PersistentRecord::new(2, cfg(ControlMode::Active, 1100, 4, true));
    let secondary = PersistentRecord::new(2, cfg(ControlMode::Active, 1100, 4, true));
    let mut app = ReferenceApp::<16>::boot(primary, secondary).expect("valid boot path");

    let a = cfg(ControlMode::Standby, 800, 5, true);
    let b = cfg(ControlMode::Active, 850, 6, true);
    let c = cfg(ControlMode::Safe, 900, 7, false);
    app.inject_pattern([a, b, c]);

    assert_eq!(
        app.step(),
        CycleOutcome::SafeHold {
            reason: SuspectReason::NoMajority,
            replicas: [a, b, c],
        }
    );
    assert!(!app.outputs_enabled());
    assert_eq!(
        app.journal_record(3).unwrap().event,
        JournalEvent::SuspectNoTrustedValue
    );
}

#[test]
fn corrupted_newer_storage_copy_rolls_back_to_older_valid_record() {
    let older = PersistentRecord::new(7, cfg(ControlMode::Standby, 700, 1, true));
    let torn =
        PersistentRecord::with_checksum(8, cfg(ControlMode::Active, 710, 2, true), 0xDEAD_BEEF);

    let mut app = ReferenceApp::<16>::boot(older, torn).expect("rollback boot path");
    assert_eq!(
        app.step(),
        CycleOutcome::Nominal {
            config: older.value()
        }
    );
    assert_eq!(app.runtime_snapshot(), [older.value(); 3]);
}
