# hdf-sync

`hdf-sync` is a phase-2 synchronization wrapper crate over `hdf-core`.

It provides two narrow adapters:

- `SerializedHardened` for explicit closure-scoped serialized access inside one owner context;
- `CriticalSectionHardened` for platform-provided critical-section entry paths in MCU-style code.

What it does:

- preserves `hdf-core` read, check, write, and repair semantics;
- returns the same `ReadReport`, `CheckReport`, and `RepairOutcome` types from `hdf-core`;
- scopes access through closure entry points so reads and writes do not overlap within the wrapper.

What it does not do:

- it does not make arbitrary `T` atomic;
- it does not claim lock-free behavior;
- it does not provide cross-thread synchronization or make the wrapper `Sync`;
- it does not create hardware critical sections;
- it does not change when `hdf-core` repairs data.

If shared contexts can observe a value mid-write and no wrapper is used, torn or mixed-replica
states remain possible. `hdf-sync` is the explicit boundary for serialized access; it does not
change the underlying truth tables.

## Example

```rust
use hdf_core::{InlineStore, ReadReport, Tmr, TrustedStatus};
use hdf_sync::{CriticalSection, CriticalSectionHardened, SerializedHardened};

let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(7, InlineStore::new(0));

assert_eq!(
    hardened.with_read(|access| access.read_checked()),
    ReadReport::Trusted {
        value: 7,
        status: TrustedStatus::Clean,
    }
);

hardened.with_write(|access| access.write(9));
assert_eq!(hardened.read_checked().trusted_value(), Some(9));

struct DemoCriticalSection;

impl CriticalSection for DemoCriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}

let protected = CriticalSectionHardened::<_, u8, Tmr, _, 3>::new(
    DemoCriticalSection,
    3,
    InlineStore::new(0),
);
assert_eq!(protected.read_checked().trusted_value(), Some(3));
```
