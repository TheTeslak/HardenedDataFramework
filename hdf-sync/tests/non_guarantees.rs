use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use hdf_core::{CheckReport, ReadReport, ReplicaStore, Tmr, TrustedStatus};
use hdf_sync::SerializedHardened;

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

    fn corrupt(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
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

#[test]
fn nested_write_access_panics_instead_of_acting_like_a_reentrant_lock() {
    let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(7, hdf_core::InlineStore::new(0));

    let result = catch_unwind(AssertUnwindSafe(|| {
        hardened.with_write(|_| {
            hardened.with_write(|_| ());
        });
    }));

    assert!(result.is_err());
}

#[test]
fn read_checked_does_not_hide_or_repair_a_recoverable_mismatch() {
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(4, store.clone());

    store.corrupt(2, 9);

    let report = hardened.read_checked();
    assert_eq!(
        report,
        ReadReport::Trusted {
            value: 4,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(hardened.check(), CheckReport::RecoverablyInconsistent);
}
