use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, Dmr, Hardened, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, Tmr,
    TrustedStatus,
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

    fn corrupt(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
    }

    fn snapshot(&self) -> [T; N] {
        *self.replicas.borrow()
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
fn dmr_wrapper_matches_direct_core_reports() {
    let direct_store = SharedStore::<u8, 2>::new(0);
    let wrapped_store = SharedStore::<u8, 2>::new(0);

    let mut direct = Hardened::<u8, Dmr, _, 2>::new(7, direct_store.clone());
    let wrapped = SerializedHardened::<u8, Dmr, _, 2>::new(7, wrapped_store.clone());

    assert_eq!(wrapped.read_checked(), direct.read_checked());
    assert_eq!(wrapped.check(), direct.check());
    assert_eq!(wrapped.repair(), direct.repair());

    direct_store.corrupt(1, 9);
    wrapped_store.corrupt(1, 9);

    assert_eq!(
        wrapped.read_checked(),
        ReadReport::Suspect {
            replicas: [7, 9],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert_eq!(wrapped.read_checked(), direct.read_checked());
    assert_eq!(wrapped.check(), direct.check());
    assert_eq!(wrapped.repair(), direct.repair());
    assert_eq!(wrapped_store.snapshot(), direct_store.snapshot());
}

#[test]
fn tmr_wrapper_matches_direct_core_reports_and_repair() {
    let direct_store = SharedStore::<u8, 3>::new(0);
    let wrapped_store = SharedStore::<u8, 3>::new(0);

    let mut direct = Hardened::<u8, Tmr, _, 3>::new(5, direct_store.clone());
    let wrapped = SerializedHardened::<u8, Tmr, _, 3>::new(5, wrapped_store.clone());

    assert_eq!(
        wrapped.read_checked(),
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::Clean,
        }
    );
    assert_eq!(wrapped.read_checked(), direct.read_checked());
    assert_eq!(wrapped.check(), direct.check());

    direct_store.corrupt(2, 8);
    wrapped_store.corrupt(2, 8);

    assert_eq!(
        wrapped.read_checked(),
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(wrapped.read_checked(), direct.read_checked());
    assert_eq!(wrapped.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(wrapped.check(), direct.check());
    assert_eq!(wrapped.repair(), RepairOutcome::Repaired);
    assert_eq!(direct.repair(), RepairOutcome::Repaired);
    assert_eq!(wrapped_store.snapshot(), [5, 5, 5]);
    assert_eq!(wrapped_store.snapshot(), direct_store.snapshot());

    direct_store.corrupt(0, 1);
    direct_store.corrupt(1, 2);
    direct_store.corrupt(2, 3);
    wrapped_store.corrupt(0, 1);
    wrapped_store.corrupt(1, 2);
    wrapped_store.corrupt(2, 3);

    assert_eq!(
        wrapped.read_checked(),
        ReadReport::Suspect {
            replicas: [1, 2, 3],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(wrapped.read_checked(), direct.read_checked());
    assert_eq!(wrapped.check(), direct.check());
    assert_eq!(wrapped.repair(), direct.repair());
    assert_eq!(wrapped_store.snapshot(), direct_store.snapshot());
}
