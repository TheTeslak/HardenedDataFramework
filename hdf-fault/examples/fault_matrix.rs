use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    Dmr, Hardened, ReadReport, RepairOutcome, ReplicaStore, SuspectReason, Tmr, TrustedStatus,
};
use hdf_fault::{
    flip_bit_in_slot, flip_bool_slot, inject_tmr_no_majority, inject_tmr_outlier, snapshot,
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

fn print_report<T: core::fmt::Debug + Copy, const N: usize>(label: &str, report: ReadReport<T, N>) {
    match report {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("{label}: trusted clean {value:?}"),
        ReadReport::Trusted {
            value,
            status: TrustedStatus::RecoverableMismatch,
        } => println!("{label}: trusted recoverable {value:?}"),
        ReadReport::Suspect {
            replicas,
            reason: SuspectReason::DmrConflict,
        } => println!("{label}: DMR conflict {replicas:?}"),
        ReadReport::Suspect {
            replicas,
            reason: SuspectReason::NoMajority,
        } => println!("{label}: no majority {replicas:?}"),
    }
}

fn main() {
    let bool_store = SharedStore::<bool, 2>::new(false);
    let bool_protected = Hardened::<bool, Dmr, _, 2>::new(true, bool_store.clone());
    println!("bool clean: {:?}", snapshot(&bool_store));
    flip_bool_slot(&bool_store, 1).expect("valid bool slot");
    print_report("bool after toggle", bool_protected.read_checked());

    let bits_store = SharedStore::<u32, 2>::new(0);
    let bits_protected = Hardened::<u32, Dmr, _, 2>::new(0b1010, bits_store.clone());
    flip_bit_in_slot(&bits_store, 1, 1).expect("valid u32 bit");
    println!("u32 replicas: {:?}", snapshot(&bits_store));
    print_report("u32 after bit flip", bits_protected.read_checked());

    let mode_store = SharedStore::<Mode, 3>::new(Mode::Standby);
    let mut mode_protected = Hardened::<Mode, Tmr, _, 3>::new(Mode::Standby, mode_store.clone());
    inject_tmr_outlier(&mode_store, Mode::Standby, 2, Mode::Active).expect("valid enum outlier");
    print_report("mode with outlier", mode_protected.read_checked());
    assert_eq!(mode_protected.repair(), RepairOutcome::Repaired);
    println!("mode after repair: {:?}", snapshot(&mode_store));
    inject_tmr_no_majority(&mode_store, [Mode::Standby, Mode::Active, Mode::Recovery])
        .expect("distinct enum states");
    print_report("mode no majority", mode_protected.read_checked());

    let config_clean = Config {
        threshold: 1200,
        flags: 0b0000_0011,
        revision: 7,
    };
    let config_b = Config {
        threshold: 1300,
        flags: 0b0000_0111,
        revision: 8,
    };
    let config_c = Config {
        threshold: 1400,
        flags: 0b1000_0011,
        revision: 9,
    };
    let config_store = SharedStore::<Config, 3>::new(config_clean);
    let config_protected = Hardened::<Config, Tmr, _, 3>::new(config_clean, config_store.clone());
    inject_tmr_no_majority(&config_store, [config_clean, config_b, config_c])
        .expect("distinct config states");
    print_report("config no majority", config_protected.read_checked());
}
