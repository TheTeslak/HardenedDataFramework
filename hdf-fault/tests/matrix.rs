use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, Dmr, Hardened, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, Tmr,
    TrustedStatus,
};
use hdf_fault::{
    FaultError, flip_bit_in_slot, flip_bool_slot, inject_tmr_no_majority, inject_tmr_outlier,
    mutate_slot, snapshot, xor_mask_slot,
};
use hdf_layout::{RegionId, ReplicaPlacement, SplitStore};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standby,
    Active,
    Recovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Config {
    threshold: u16,
    flags: u8,
    revision: u8,
}

#[test]
fn bit_flip_helpers_make_integer_faults_explicit() {
    let store = SharedStore::<u32, 2>::new(0);
    let mut protected = Hardened::<u32, Dmr, _, 2>::new(0b1010, store.clone());

    assert_eq!(flip_bit_in_slot(&store, 1, 1), Ok(0b1000));
    assert_eq!(snapshot(&store), [0b1010, 0b1000]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [0b1010, 0b1000],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
    assert_eq!(protected.repair(), RepairOutcome::NotPossible);

    assert_eq!(xor_mask_slot(&store, 0, 0b0101), Ok(0b1111));
    assert_eq!(snapshot(&store), [0b1111, 0b1000]);
    assert_eq!(
        flip_bit_in_slot(&store, 0, 32),
        Err(FaultError::BitOutOfRange { bit: 32, width: 32 })
    );
}

#[test]
fn bool_and_enum_faults_remain_visible_at_report_level() {
    let dmr_store = SharedStore::<bool, 2>::new(false);
    let protected_flag = Hardened::<bool, Dmr, _, 2>::new(true, dmr_store.clone());

    assert_eq!(flip_bool_slot(&dmr_store, 1), Ok(false));
    assert_eq!(snapshot(&dmr_store), [true, false]);
    assert_eq!(
        protected_flag.read_checked(),
        ReadReport::Suspect {
            replicas: [true, false],
            reason: SuspectReason::DmrConflict,
        }
    );

    let tmr_store = SharedStore::<Mode, 3>::new(Mode::Standby);
    let mut protected_mode = Hardened::<Mode, Tmr, _, 3>::new(Mode::Standby, tmr_store.clone());

    inject_tmr_outlier(&tmr_store, Mode::Standby, 2, Mode::Recovery).expect("valid enum outlier");
    assert_eq!(
        protected_mode.read_checked(),
        ReadReport::Trusted {
            value: Mode::Standby,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected_mode.repair(), RepairOutcome::Repaired);
    assert_eq!(
        snapshot(&tmr_store),
        [Mode::Standby, Mode::Standby, Mode::Standby]
    );

    mutate_slot(&tmr_store, 1, |_| Mode::Active).expect("valid enum mutation");
    mutate_slot(&tmr_store, 2, |_| Mode::Recovery).expect("valid enum mutation");
    assert_eq!(
        snapshot(&tmr_store),
        [Mode::Standby, Mode::Active, Mode::Recovery]
    );
    assert_eq!(
        protected_mode.read_checked(),
        ReadReport::Suspect {
            replicas: [Mode::Standby, Mode::Active, Mode::Recovery],
            reason: SuspectReason::NoMajority,
        }
    );
}

#[test]
fn config_struct_patterns_cover_recoverable_and_suspect_tmr_paths() {
    let clean = Config {
        threshold: 1200,
        flags: 0b0000_0011,
        revision: 7,
    };
    let recoverable = Config {
        threshold: 1200,
        flags: 0b0000_0111,
        revision: 7,
    };
    let distinct_b = Config {
        threshold: 1300,
        flags: 0b0000_0011,
        revision: 8,
    };
    let distinct_c = Config {
        threshold: 1400,
        flags: 0b1000_0011,
        revision: 9,
    };

    let store = SharedStore::<Config, 3>::new(clean);
    let mut protected = Hardened::<Config, Tmr, _, 3>::new(clean, store.clone());

    inject_tmr_outlier(&store, clean, 1, recoverable).expect("valid struct outlier");
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: clean,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [clean, clean, clean]);

    inject_tmr_no_majority(&store, [clean, distinct_b, distinct_c]).expect("distinct structs");
    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [clean, distinct_b, distinct_c],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
}

#[test]
fn split_store_backend_can_be_corrupted_and_repaired_without_changing_core_truth_rules() {
    let store = SplitStore::new(
        0u16,
        ReplicaPlacement::new([RegionId(10), RegionId(20), RegionId(30)]),
    );
    let mut protected = Hardened::<u16, Tmr, _, 3>::new(0x00F0, &store);

    assert_eq!(store.region_of(0), Some(RegionId(10)));
    assert_eq!(flip_bit_in_slot(&store, 2, 0), Ok(0x00F1));
    assert_eq!(snapshot(&store), [0x00F0, 0x00F0, 0x00F1]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 0x00F0,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(snapshot(&store), [0x00F0, 0x00F0, 0x00F0]);
}
