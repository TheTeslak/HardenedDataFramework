use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{CheckReport, ReadReport, ReplicaStore, SuspectReason, Tmr, TrustedStatus};
use hdf_fault::{apply_pattern, inject_tmr_no_majority, inject_tmr_outlier, snapshot};
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
fn injected_patterns_flow_through_serialized_reads_and_repairs() {
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(5, store.clone());

    inject_tmr_outlier(&store, 5, 0, 9).expect("valid outlier scenario");
    assert_eq!(snapshot(&store), [9, 5, 5]);
    assert_eq!(
        hardened.read_checked(),
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(hardened.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(hardened.repair(), hdf_core::RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [5, 5, 5]);
}

#[test]
fn torn_state_patterns_remain_visible_to_wrapper_callers() {
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = SerializedHardened::<u8, Tmr, _, 3>::new(4, store.clone());

    apply_pattern(&store, [4, 9, 4]);
    assert_eq!(
        hardened.with_read(|access| access.read_checked()),
        ReadReport::Trusted {
            value: 4,
            status: TrustedStatus::RecoverableMismatch,
        }
    );

    inject_tmr_no_majority(&store, [1, 2, 3]).expect("distinct no-majority pattern");
    assert_eq!(snapshot(&store), [1, 2, 3]);
    assert_eq!(
        hardened.with_read(|access| access.read_checked()),
        ReadReport::Suspect {
            replicas: [1, 2, 3],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(
        hardened.with_read(|access| access.check()),
        CheckReport::Suspect
    );
}
