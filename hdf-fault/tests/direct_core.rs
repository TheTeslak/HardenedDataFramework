use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, Dmr, Hardened, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, Tmr,
    TrustedStatus,
};
use hdf_fault::{
    FaultError, apply_pattern, corrupt_slot, inject_dmr_conflict, inject_tmr_no_majority,
    inject_tmr_outlier, snapshot,
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
fn dmr_single_slot_corruption_is_detectable_from_either_position() {
    for corrupted_index in 0..2 {
        let store = SharedStore::<u8, 2>::new(0);
        let mut protected = Hardened::<u8, Dmr, _, 2>::new(5, store.clone());

        corrupt_slot(&store, corrupted_index, 9).expect("valid DMR slot index");

        assert_eq!(
            snapshot(&store),
            if corrupted_index == 0 { [9, 5] } else { [5, 9] }
        );
        assert_eq!(
            protected.read_checked(),
            ReadReport::Suspect {
                replicas: snapshot(&store),
                reason: SuspectReason::DmrConflict,
            }
        );
        assert_eq!(protected.check(), CheckReport::Suspect);
        assert_eq!(protected.repair(), RepairOutcome::NotPossible);
    }
}

#[test]
fn tmr_outlier_is_recoverable_for_each_replica_position() {
    for outlier_index in 0..3 {
        let store = SharedStore::<u8, 3>::new(0);
        let mut protected = Hardened::<u8, Tmr, _, 3>::new(7, store.clone());

        inject_tmr_outlier(&store, 7, outlier_index, 9).expect("valid TMR outlier index");

        let mut expected = [7, 7, 7];
        expected[outlier_index] = 9;
        assert_eq!(snapshot(&store), expected);
        assert_eq!(
            protected.read_checked(),
            ReadReport::Trusted {
                value: 7,
                status: TrustedStatus::RecoverableMismatch,
            }
        );
        assert_eq!(protected.check(), CheckReport::RecoverablyInconsistent);
        assert_eq!(protected.repair(), RepairOutcome::Repaired);
        assert_eq!(snapshot(&store), [7, 7, 7]);
    }
}

#[test]
fn tmr_distinguishes_majority_shift_from_no_majority() {
    let store = SharedStore::<u8, 3>::new(0);
    let mut protected = Hardened::<u8, Tmr, _, 3>::new(4, store.clone());

    inject_tmr_outlier(&store, 9, 2, 4).expect("valid majority-shift pattern");
    assert_eq!(snapshot(&store), [9, 9, 4]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 9,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [9, 9, 9]);

    inject_tmr_no_majority(&store, [1, 2, 3]).expect("distinct no-majority pattern");
    assert_eq!(snapshot(&store), [1, 2, 3]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [1, 2, 3],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
    assert_eq!(protected.repair(), RepairOutcome::NotPossible);
}

#[test]
fn repeated_corruption_repair_and_write_sequences_do_not_accumulate_hidden_state() {
    let store = SharedStore::<u8, 3>::new(0);
    let mut protected = Hardened::<u8, Tmr, _, 3>::new(8, store.clone());

    inject_tmr_outlier(&store, 8, 1, 3).expect("valid outlier scenario");
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [8, 8, 8]);

    apply_pattern(&store, [2, 2, 8]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 2,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [2, 2, 2]);

    protected.write(6);
    assert_eq!(snapshot(&store), [6, 6, 6]);
}

#[test]
fn helper_errors_are_explicit_for_invalid_fault_patterns() {
    let dmr_store = SharedStore::<u8, 2>::new(0);
    assert_eq!(
        corrupt_slot(&dmr_store, 3, 1),
        Err(FaultError::SlotOutOfRange {
            index: 3,
            replicas: 2,
        })
    );

    let tmr_store = SharedStore::<u8, 3>::new(0);
    assert_eq!(
        inject_tmr_no_majority(&tmr_store, [1, 1, 2]),
        Err(FaultError::TmrNoMajorityRequiresDistinctValues)
    );

    inject_dmr_conflict(&dmr_store, 4, 9);
    assert_eq!(snapshot(&dmr_store), [4, 9]);
}
