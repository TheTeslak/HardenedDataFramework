use hdf_journal::{Journal, JournalEvent, ResetCause, StorageSource, decode_record};

fn main() {
    let mut journal = Journal::<8>::new();

    journal
        .append(JournalEvent::ResetObserved {
            cause: ResetCause::PowerOn,
        })
        .expect("room");
    journal
        .append(JournalEvent::StorageRecordSelected {
            source: StorageSource::Secondary,
            version: 12,
        })
        .expect("room");
    journal
        .append(JournalEvent::RecoverableMismatch)
        .expect("room");
    journal.append(JournalEvent::RepairAttempted).expect("room");
    journal.append(JournalEvent::RepairSucceeded).expect("room");

    for index in 0..journal.len() {
        let record = decode_record(journal.encoded(index).expect("entry exists")).expect("decode");
        println!("{record}");
    }
}
