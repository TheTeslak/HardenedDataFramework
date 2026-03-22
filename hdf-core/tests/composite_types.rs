use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, TrustedStatus,
    detect_only_in, recoverable_in,
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

    fn set(&self, index: usize, value: T) {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standby,
    Active,
    Recovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ControlConfig {
    mode: Mode,
    threshold: u16,
    revision: u8,
    enabled: bool,
}

#[test]
fn composite_dmr_values_remain_detect_only() {
    let clean = ControlConfig {
        mode: Mode::Standby,
        threshold: 1200,
        revision: 1,
        enabled: true,
    };
    let conflicting = ControlConfig {
        mode: Mode::Standby,
        threshold: 1250,
        revision: 1,
        enabled: true,
    };

    let store = SharedStore::<ControlConfig, 2>::new(clean);
    let protected = detect_only_in(clean, store.clone());

    store.set(1, conflicting);

    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [clean, conflicting],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
}

#[test]
fn composite_tmr_values_support_recoverable_majority_and_repair() {
    let clean = ControlConfig {
        mode: Mode::Active,
        threshold: 900,
        revision: 7,
        enabled: true,
    };
    let outlier = ControlConfig {
        mode: Mode::Recovery,
        threshold: 900,
        revision: 8,
        enabled: true,
    };

    let store = SharedStore::<ControlConfig, 3>::new(clean);
    let mut protected = recoverable_in(clean, store.clone());

    store.set(2, outlier);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: clean,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [clean, clean, clean]);
}

#[test]
fn composite_voting_is_whole_value_equality_not_field_wise_merge() {
    let base = ControlConfig {
        mode: Mode::Active,
        threshold: 700,
        revision: 2,
        enabled: true,
    };
    let different_threshold = ControlConfig {
        mode: Mode::Active,
        threshold: 701,
        revision: 2,
        enabled: true,
    };
    let different_mode = ControlConfig {
        mode: Mode::Recovery,
        threshold: 700,
        revision: 2,
        enabled: true,
    };

    let store = SharedStore::<ControlConfig, 3>::new(base);
    let protected = recoverable_in(base, store.clone());

    store.set(0, base);
    store.set(1, different_threshold);
    store.set(2, different_mode);

    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [base, different_threshold, different_mode],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
}
