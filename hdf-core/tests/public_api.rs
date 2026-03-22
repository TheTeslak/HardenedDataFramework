use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, Dmr, Hardened, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, Tmr,
    TrustedStatus,
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
fn dmr_read_checked_reports_clean_and_conflict_states() {
    let store = SharedStore::<u8, 2>::new(0);
    let hardened = Hardened::<u8, Dmr, _, 2>::new(7, store.clone());

    let clean = hardened.read_checked();
    assert_eq!(
        clean,
        ReadReport::Trusted {
            value: 7,
            status: TrustedStatus::Clean,
        }
    );
    assert!(clean.is_trusted());
    assert!(!clean.is_suspect());
    assert_eq!(clean.trusted_value(), Some(7));

    store.corrupt(1, 9);

    let conflict = hardened.read_checked();
    assert_eq!(
        conflict,
        ReadReport::Suspect {
            replicas: [7, 9],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert!(!conflict.is_trusted());
    assert!(conflict.is_suspect());
    assert_eq!(conflict.trusted_value(), None);
}

#[test]
fn dmr_check_and_repair_follow_public_contract() {
    let store = SharedStore::<u8, 2>::new(0);
    let mut hardened = Hardened::<u8, Dmr, _, 2>::new(4, store.clone());

    let clean_check = hardened.check();
    assert_eq!(clean_check, CheckReport::Consistent);
    assert!(!clean_check.needs_repair());
    assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);
    assert_eq!(store.snapshot(), [4, 4]);

    store.corrupt(0, 1);

    let suspect_check = hardened.check();
    assert_eq!(suspect_check, CheckReport::Suspect);
    assert!(!suspect_check.needs_repair());
    assert_eq!(hardened.repair(), RepairOutcome::NotPossible);
    assert_eq!(store.snapshot(), [1, 4]);
}

#[test]
fn tmr_read_checked_reports_clean_recoverable_and_no_majority_states() {
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = Hardened::<u8, Tmr, _, 3>::new(5, store.clone());

    let clean = hardened.read_checked();
    assert_eq!(
        clean,
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::Clean,
        }
    );
    assert!(clean.is_trusted());
    assert_eq!(clean.trusted_value(), Some(5));

    store.corrupt(2, 8);

    let recoverable = hardened.read_checked();
    assert_eq!(
        recoverable,
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert!(recoverable.is_trusted());
    assert_eq!(recoverable.trusted_value(), Some(5));

    store.corrupt(0, 1);
    store.corrupt(1, 2);
    store.corrupt(2, 3);

    let no_majority = hardened.read_checked();
    assert_eq!(
        no_majority,
        ReadReport::Suspect {
            replicas: [1, 2, 3],
            reason: SuspectReason::NoMajority,
        }
    );
    assert!(no_majority.is_suspect());
    assert_eq!(no_majority.trusted_value(), None);
}

#[test]
fn tmr_check_and_repair_follow_public_contract() {
    let store = SharedStore::<u8, 3>::new(0);
    let mut hardened = Hardened::<u8, Tmr, _, 3>::new(6, store.clone());

    let clean_check = hardened.check();
    assert_eq!(clean_check, CheckReport::Consistent);
    assert!(!clean_check.needs_repair());
    assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);
    assert_eq!(store.snapshot(), [6, 6, 6]);

    store.corrupt(1, 9);

    let recoverable_check = hardened.check();
    assert_eq!(recoverable_check, CheckReport::RecoverablyInconsistent);
    assert!(recoverable_check.needs_repair());
    assert_eq!(hardened.repair(), RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [6, 6, 6]);

    store.corrupt(0, 1);
    store.corrupt(1, 2);
    store.corrupt(2, 3);

    let suspect_check = hardened.check();
    assert_eq!(suspect_check, CheckReport::Suspect);
    assert!(!suspect_check.needs_repair());
    assert_eq!(hardened.repair(), RepairOutcome::NotPossible);
    assert_eq!(store.snapshot(), [1, 2, 3]);
}
