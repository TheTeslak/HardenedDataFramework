use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, Hardened, InlineStore, ReadReport, RepairOutcome, ReplicaStore, Tmr, TrustedStatus,
};
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

    fn snapshot(&self) -> [T; N] {
        *self.replicas.borrow()
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
fn closure_api_serializes_access_and_preserves_reports() {
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(7, store.clone());

    let report = hardened.with_read(|access| access.read_checked());
    assert_eq!(
        report,
        ReadReport::Trusted {
            value: 7,
            status: TrustedStatus::Clean,
        }
    );

    let check = hardened.with_write(|access| {
        access.write(9);
        access.check()
    });
    assert_eq!(check, CheckReport::Consistent);
    assert_eq!(store.snapshot(), [9, 9, 9]);

    store.corrupt(2, 3);
    let outcome = hardened.with_write(|access| {
        assert_eq!(access.check(), CheckReport::RecoverablyInconsistent);
        access.repair()
    });
    assert_eq!(outcome, RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [9, 9, 9]);
}

#[test]
fn from_hardened_wraps_existing_core_value() {
    let core = Hardened::<u8, Tmr, _, 3>::new(5, InlineStore::new(0));
    let hardened = SerializedHardened::from_hardened(core);

    assert_eq!(hardened.read_checked().trusted_value(), Some(5));
    hardened.write(8);
    assert_eq!(hardened.read_checked().trusted_value(), Some(8));
}
