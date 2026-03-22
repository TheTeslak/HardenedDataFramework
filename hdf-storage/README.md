# hdf-storage

`hdf-storage` is the phase-7 scaffold for persistent critical state.

It intentionally uses a separate API family from `hdf-core` because non-volatile storage has
different failure modes and recovery boundaries.

What it does:

- models versioned persistent records with explicit integrity metadata;
- distinguishes valid, stale-by-version, corrupted, and conflicting record situations;
- provides an explicit `load_pair(...)` / `load_into_core(...)` flow instead of pretending storage behaves like RAM.

What it does not do:

- it does not mirror `Hardened::read_checked()` or other RAM-core methods;
- it does not hide the boundary between persistent recovery and in-memory hardening;
- it does not claim journaling, wear leveling, or power-loss atomicity beyond this A/B-style scaffold.

## Example

```rust
use hdf_core::{InlineStore, Tmr};
use hdf_storage::{LoadReason, LoadReport, PersistentRecord, RecordData, load_into_core};

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

let older = PersistentRecord::new(
    4,
    Config {
        threshold: 1000,
        flags: 0x03,
        revision: 7,
    },
);
let newer = PersistentRecord::new(
    5,
    Config {
        threshold: 1100,
        flags: 0x07,
        revision: 8,
    },
);

let (protected, report) = load_into_core::<_, Tmr, _, 3>(older, newer, InlineStore::new(Config {
    threshold: 0,
    flags: 0,
    revision: 0,
}))
.expect("a valid record should load");

assert_eq!(protected.read_checked().trusted_value(), Some(newer.value()));
assert_eq!(
    report,
    LoadReport::Trusted {
        value: newer.value(),
        version: 5,
        source: hdf_storage::SlotId::Secondary,
        reason: LoadReason::SecondaryNewer,
    }
);
```
