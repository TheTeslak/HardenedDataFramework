use hdf_core::{Hardened, ReplicaStore, Scheme};

use crate::record::{PersistentRecord, RecordData};
use crate::report::{LoadReason, LoadReport, RecordStatus, SlotId};

pub fn classify_record<T: RecordData>(record: PersistentRecord<T>) -> RecordStatus {
    if record.is_valid() {
        RecordStatus::Valid {
            version: record.version(),
        }
    } else {
        RecordStatus::Corrupted {
            version: record.version(),
        }
    }
}

pub fn load_pair<T: RecordData>(
    primary: PersistentRecord<T>,
    secondary: PersistentRecord<T>,
) -> LoadReport<T> {
    let primary_status = classify_record(primary);
    let secondary_status = classify_record(secondary);

    match (primary.is_valid(), secondary.is_valid()) {
        (true, true) => select_valid_pair(primary, secondary),
        (true, false) => LoadReport::Trusted {
            value: primary.value(),
            version: primary.version(),
            source: SlotId::Primary,
            reason: LoadReason::OtherCopyCorrupted,
        },
        (false, true) => LoadReport::Trusted {
            value: secondary.value(),
            version: secondary.version(),
            source: SlotId::Secondary,
            reason: LoadReason::OtherCopyCorrupted,
        },
        (false, false) => LoadReport::NoUsableRecord {
            primary: primary_status,
            secondary: secondary_status,
        },
    }
}

fn select_valid_pair<T: RecordData>(
    primary: PersistentRecord<T>,
    secondary: PersistentRecord<T>,
) -> LoadReport<T> {
    if primary.version() == secondary.version() {
        if primary.value() == secondary.value() {
            return LoadReport::Trusted {
                value: primary.value(),
                version: primary.version(),
                source: SlotId::Primary,
                reason: LoadReason::MatchingCopies,
            };
        }

        return LoadReport::Conflict {
            primary: RecordStatus::Valid {
                version: primary.version(),
            },
            secondary: RecordStatus::Valid {
                version: secondary.version(),
            },
        };
    }

    if primary.version() > secondary.version() {
        LoadReport::Trusted {
            value: primary.value(),
            version: primary.version(),
            source: SlotId::Primary,
            reason: LoadReason::PrimaryNewer,
        }
    } else {
        LoadReport::Trusted {
            value: secondary.value(),
            version: secondary.version(),
            source: SlotId::Secondary,
            reason: LoadReason::SecondaryNewer,
        }
    }
}

pub fn load_into_core<T, S, Store, const N: usize>(
    primary: PersistentRecord<T>,
    secondary: PersistentRecord<T>,
    store: Store,
) -> Result<(Hardened<T, S, Store, N>, LoadReport<T>), LoadReport<T>>
where
    T: RecordData,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    let report = load_pair(primary, secondary);
    match report {
        LoadReport::Trusted { value, .. } => Ok((Hardened::new(value, store), report)),
        LoadReport::Conflict { .. } | LoadReport::NoUsableRecord { .. } => Err(report),
    }
}
