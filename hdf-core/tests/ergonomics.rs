use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, TrustedStatus,
    detect_only, detect_only_in, recoverable, recoverable_in,
};

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
fn detect_only_helper_keeps_dmr_reports_explicit() {
    let store = SharedStore::<bool, 2>::new(false);
    let protected = detect_only_in(true, store.clone());

    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: true,
            status: TrustedStatus::Clean,
        }
    );

    store.corrupt(1, false);
    assert_eq!(store.snapshot(), [true, false]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [true, false],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
}

#[test]
fn recoverable_helper_keeps_tmr_reports_and_repair_explicit() {
    let store = SharedStore::<u16, 3>::new(0);
    let mut protected = recoverable_in(1200u16, store.clone());

    store.corrupt(2, 1300);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 1200,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [1200, 1200, 1200]);
}

#[test]
fn inline_helper_constructors_match_direct_usage() {
    let mut detect = detect_only(9u8);
    let mut recover = recoverable(7u8);

    assert_eq!(detect.read_checked().trusted_value(), Some(9));
    assert_eq!(recover.read_checked().trusted_value(), Some(7));

    detect.write(3);
    recover.write(5);

    assert_eq!(detect.read_checked().trusted_value(), Some(3));
    assert_eq!(recover.read_checked().trusted_value(), Some(5));
}
