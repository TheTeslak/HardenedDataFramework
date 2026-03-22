# hdf-fault

`hdf-fault` is the phase-4 scaffold for deterministic replica corruption and verification helpers.

It adds narrow, explicit utilities for:

- mutating individual replica slots through the public `ReplicaStore` contract;
- applying value-level or bit-level corruption to supported replica types;
- applying known replica patterns for DMR and TMR scenarios;
- capturing raw replica snapshots for verification.

What it does:

- supports deterministic corruption in tests and examples;
- supports explicit bit flips for integer replica types and safe value mutation closures for other `Copy` data;
- keeps `hdf-core` and `hdf-sync` semantics unchanged;
- makes DMR conflict, TMR outlier, and TMR no-majority scenarios reproducible.

What it does not do:

- it does not change `ReadReport`, `CheckReport`, or `RepairOutcome` behavior;
- it does not add hidden repair logic or metadata;
- it does not claim hardware fault injection;
- it does not treat the included host-side overhead numbers as a substitute for target-specific measurement.

## Example

```rust
use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{Hardened, ReadReport, ReplicaStore, Tmr, TrustedStatus};
use hdf_fault::{flip_bit_in_slot, inject_tmr_no_majority, inject_tmr_outlier, snapshot};

#[derive(Clone)]
struct SharedStore<T, const N: usize> {
    replicas: Rc<RefCell<[T; N]>>,
}

impl<T: Copy, const N: usize> SharedStore<T, N> {
    fn new(initial: T) -> Self {
        Self {
            replicas: Rc::new(RefCell::new([initial; N])),
        }
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for SharedStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.replicas.borrow()[index]
    }

    fn write_slot(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
    }
}

let store = SharedStore::<u8, 3>::new(0);
let mut protected = Hardened::<u8, Tmr, _, 3>::new(7, store.clone());

inject_tmr_outlier(&store, 7, 2, 9)?;
assert_eq!(snapshot(&store), [7, 7, 9]);
assert_eq!(
    protected.read_checked(),
    ReadReport::Trusted {
        value: 7,
        status: TrustedStatus::RecoverableMismatch,
    }
);
assert!(protected.repair().eq(&hdf_core::RepairOutcome::Repaired));

flip_bit_in_slot(&store, 0, 0)?;
assert_eq!(snapshot(&store), [6, 7, 7]);

inject_tmr_no_majority(&store, [1, 2, 3])?;
assert_eq!(snapshot(&store), [1, 2, 3]);
assert!(protected.read_checked().is_suspect());
# Ok::<(), hdf_fault::FaultError>(())
```

For a more visible walkthrough, run `cargo run -p hdf-fault --example fault_matrix`.

For host-side overhead measurement, run `cargo run -p hdf-fault --example overhead_report --release` and
see `hdf-fault/OVERHEAD.md` for the current checked-in baseline.
