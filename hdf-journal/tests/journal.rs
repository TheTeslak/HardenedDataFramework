use hdf_journal::{
    AppendError, EventRecord, Journal, JournalEvent, ResetCause, StorageSource, decode_record,
};

#[test]
fn encode_decode_round_trip_preserves_events() {
    let records = [
        EventRecord {
            sequence: 1,
            event: JournalEvent::IntegrityClean,
        },
        EventRecord {
            sequence: 2,
            event: JournalEvent::RecoverableMismatch,
        },
        EventRecord {
            sequence: 3,
            event: JournalEvent::SuspectNoTrustedValue,
        },
        EventRecord {
            sequence: 4,
            event: JournalEvent::RepairAttempted,
        },
        EventRecord {
            sequence: 5,
            event: JournalEvent::RepairSucceeded,
        },
        EventRecord {
            sequence: 6,
            event: JournalEvent::RepairNotPossible,
        },
        EventRecord {
            sequence: 7,
            event: JournalEvent::StorageRecordSelected {
                source: StorageSource::Primary,
                version: 42,
            },
        },
        EventRecord {
            sequence: 8,
            event: JournalEvent::StorageRecoveryFailed,
        },
        EventRecord {
            sequence: 9,
            event: JournalEvent::ResetObserved {
                cause: ResetCause::Watchdog,
            },
        },
    ];

    for record in records {
        assert_eq!(decode_record(record.encode()), Some(record));
    }
}

#[test]
fn journal_preserves_order_and_reports_full_buffer() {
    let mut journal = Journal::<2>::new();

    assert_eq!(journal.append(JournalEvent::IntegrityClean), Ok(0));
    assert_eq!(journal.append(JournalEvent::RepairSucceeded), Ok(1));
    assert_eq!(
        journal.append(JournalEvent::RepairNotPossible),
        Err(AppendError::Full)
    );

    assert_eq!(journal.len(), 2);
    assert_eq!(
        decode_record(journal.encoded(0).unwrap()).unwrap(),
        EventRecord {
            sequence: 0,
            event: JournalEvent::IntegrityClean,
        }
    );
    assert_eq!(
        decode_record(journal.encoded(1).unwrap()).unwrap(),
        EventRecord {
            sequence: 1,
            event: JournalEvent::RepairSucceeded,
        }
    );
}
