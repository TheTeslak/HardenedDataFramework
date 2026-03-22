# hdf-layout

`hdf-layout` is the phase scaffold for layout-aware storage backends in HDF.

It adds placement metadata and storage backends that can spread replicas across named regions,
optional banks/sections, and optional complemented encoding for bitwise-compatible values,
while keeping the DMR/TMR truth rules in `hdf-core` unchanged.

What it does:

- provides `RegionId` and `ReplicaPlacement<N>` as explicit replica-to-region metadata;
- provides `BankId`, `SectionId`, and `PlacementSite` for richer placement descriptions;
- provides `SplitStore<T, N>` as a placement-aware backend implementing `hdf_core::ReplicaStore`;
- provides `ComplementedStore<T, N>` as an explicit encoded-policy backend for complemented integer values;
- stays usable today with `hdf_core::Hardened` without changing `ReadReport`, `CheckReport`, or `RepairOutcome` semantics.

What it does not do:

- it does not change scheme logic, majority rules, or repair behavior from `hdf-core`;
- it does not claim strong physical fault isolation, persistence, or hardware-enforced separation;
- it does not claim complemented encoding improves logical voting semantics;
- it does not enforce linker placement on its own; `SectionId` is descriptive integration metadata.

## Example

```rust
use hdf_core::{Hardened, ReadReport, Tmr, TrustedStatus};
use hdf_layout::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId, SplitStore};

let placement = ReplicaPlacement::with_sites([
    PlacementSite::with_details(RegionId(0), Some(BankId(0)), Some(SectionId(".replica_a"))),
    PlacementSite::with_details(RegionId(1), Some(BankId(1)), Some(SectionId(".replica_b"))),
    PlacementSite::new(RegionId(2)),
]);
let store = SplitStore::new(0u8, placement);
let protected = Hardened::<u8, Tmr, _, 3>::new(42, store);

assert_eq!(protected.read_checked(), ReadReport::Trusted {
    value: 42,
    status: TrustedStatus::Clean,
});
```

`SectionId` is intended to line up with linker or memory-placement documentation when a target
platform has distinct sections or banks. The crate records this intent in metadata but does not
pretend the compiler or linker enforced separation unless the surrounding build does so.

For encoded-policy experiments, `ComplementedStore` keeps the logical API unchanged while storing
the complemented bit pattern internally for supported integer types. Clean, corrupted, and repair
paths still flow through the same `hdf-core` reports.
