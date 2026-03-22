use crate::record::{PersistentRecord, RecordData};

pub fn prepare_record<T: RecordData>(version: u32, value: T) -> PersistentRecord<T> {
    PersistentRecord::new(version, value)
}

pub fn next_record<T: RecordData>(current_version: u32, value: T) -> PersistentRecord<T> {
    PersistentRecord::new(current_version.wrapping_add(1), value)
}
