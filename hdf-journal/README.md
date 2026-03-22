# hdf-journal

`hdf-journal` is the phase-8 scaffold for integrity and recovery observability.

It provides:

- compact event definitions for integrity and storage recovery paths;
- fixed-size encoded event records;
- a small append-only journal buffer with explicit full-buffer behavior;
- decode/render helpers for host-side inspection.

What it does not do:

- it does not change `hdf-core`, `hdf-storage`, or `hdf-sync` semantics;
- it does not claim durable storage by itself;
- it does not hide truncation policy: the append API returns `AppendError::Full` when the fixed buffer is exhausted.

## Example

```rust
use hdf_journal::{EventRecord, Journal, JournalEvent, StorageSource, decode_record};

let mut journal = Journal::<4>::new();
let seq = journal
    .append(JournalEvent::StorageRecordSelected {
        source: StorageSource::Secondary,
        version: 7,
    })
    .expect("space available");

let encoded = journal.encoded(0).expect("entry exists");
let decoded = decode_record(encoded).expect("valid encoding");

assert_eq!(seq, 0);
assert_eq!(
    decoded,
    EventRecord {
        sequence: 0,
        event: JournalEvent::StorageRecordSelected {
            source: StorageSource::Secondary,
            version: 7,
        },
    }
);
```
