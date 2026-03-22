# hdf-core

`hdf-core` is the phase-1 RAM-resident core of the Hardened Data Framework.

It provides:

- selective hardening for small `Copy + Eq` values;
- detect-only DMR and recoverable TMR semantics;
- explicit integrity reporting via `ReadReport` and `CheckReport`;
- explicit `repair()` rather than hidden writeback during reads;
- `InlineStore` as the baseline volatile replica backend;
- `no_std`, allocation-free operation.

The crate intentionally does **not** provide concurrency guarantees, persistence semantics, journaling, or physical-placement claims beyond the baseline inline store.

## Ergonomic helpers

For common selective-hardening choices, the crate also exposes narrow aliases and constructors:

- `detect_only(...)` / `DetectOnly...` for DMR-backed detect-only values;
- `recoverable(...)` / `Recoverable...` for TMR-backed recoverable values.

These helpers do not change trust semantics. Reads still return explicit `ReadReport` values and suspect handling remains the caller's responsibility.

## Structured data

Small composite values work when they remain `Copy + Eq` and when whole-value equality matches the trust decision you want. The framework does not do field-wise voting or partial merge. See `docs/structured-data.md` for practical guidance.

## Example

```rust
use hdf_core::{CheckReport, Hardened, InlineStore, ReadReport, RepairOutcome, Tmr, TrustedStatus};

let mut hardened = Hardened::<u8, Tmr, _, 3>::new(7, InlineStore::new(0));

assert_eq!(
    hardened.read_checked(),
    ReadReport::Trusted {
        value: 7,
        status: TrustedStatus::Clean,
    }
);

hardened.write(9);
assert_eq!(hardened.check(), CheckReport::Consistent);
assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);
```
