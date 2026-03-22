use hdf_core::{InlineStore, Tmr};
use hdf_storage::{
    LoadReason, LoadReport, PersistentRecord, RecordData, RecordStatus, SlotId, classify_record,
    load_into_core, load_pair, next_record, prepare_record,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Config {
    threshold: u16,
    flags: u8,
    revision: u8,
}

impl RecordData for Config {
    fn checksum(&self) -> u32 {
        self.threshold as u32 ^ ((self.flags as u32) << 8) ^ ((self.revision as u32) << 16)
    }
}

#[test]
fn classify_record_distinguishes_valid_and_corrupted_records() {
    let valid = prepare_record(3, 42u32);
    let corrupted = PersistentRecord::with_checksum(4, 99u32, 0xDEAD_BEEF);

    assert_eq!(classify_record(valid), RecordStatus::Valid { version: 3 });
    assert_eq!(
        classify_record(corrupted),
        RecordStatus::Corrupted { version: 4 }
    );
}

#[test]
fn load_pair_selects_newer_valid_record_and_marks_stale_peer() {
    let primary = prepare_record(7, 100u32);
    let secondary = next_record(7, 120u32);

    assert_eq!(
        load_pair(primary, secondary),
        LoadReport::Trusted {
            value: 120,
            version: 8,
            source: SlotId::Secondary,
            reason: LoadReason::SecondaryNewer,
        }
    );
}

#[test]
fn interrupted_update_rolls_back_to_older_valid_record_when_newer_copy_is_corrupted() {
    let stable = prepare_record(9, 55u32);
    let torn = PersistentRecord::with_checksum(10, 77u32, 0x1234_5678);

    assert_eq!(
        load_pair(stable, torn),
        LoadReport::Trusted {
            value: 55,
            version: 9,
            source: SlotId::Primary,
            reason: LoadReason::OtherCopyCorrupted,
        }
    );
}

#[test]
fn load_pair_reports_conflict_for_same_version_different_valid_values() {
    let primary = prepare_record(5, 10u32);
    let secondary = prepare_record(5, 11u32);

    assert_eq!(
        load_pair(primary, secondary),
        LoadReport::Conflict {
            primary: RecordStatus::Valid { version: 5 },
            secondary: RecordStatus::Valid { version: 5 },
        }
    );
}

#[test]
fn load_pair_reports_no_usable_record_when_both_copies_are_corrupted() {
    let primary = PersistentRecord::with_checksum(1, 10u32, 0xAAAA_5555);
    let secondary = PersistentRecord::with_checksum(2, 11u32, 0xBBBB_6666);

    assert_eq!(
        load_pair(primary, secondary),
        LoadReport::NoUsableRecord {
            primary: RecordStatus::Corrupted { version: 1 },
            secondary: RecordStatus::Corrupted { version: 2 },
        }
    );
}

#[test]
fn load_into_core_makes_storage_to_ram_boundary_explicit() {
    let older = prepare_record(
        4,
        Config {
            threshold: 1000,
            flags: 0x03,
            revision: 7,
        },
    );
    let newer = next_record(
        4,
        Config {
            threshold: 1100,
            flags: 0x07,
            revision: 8,
        },
    );

    let (protected, report) = load_into_core::<_, Tmr, _, 3>(
        older,
        newer,
        InlineStore::new(Config {
            threshold: 0,
            flags: 0,
            revision: 0,
        }),
    )
    .expect("valid record pair should load into core");

    assert_eq!(
        protected.read_checked().trusted_value(),
        Some(newer.value())
    );
    assert_eq!(
        report,
        LoadReport::Trusted {
            value: newer.value(),
            version: 5,
            source: SlotId::Secondary,
            reason: LoadReason::SecondaryNewer,
        }
    );
}
