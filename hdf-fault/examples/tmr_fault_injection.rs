use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{Hardened, ReadReport, RepairOutcome, ReplicaStore, Tmr, TrustedStatus};
use hdf_fault::{inject_tmr_no_majority, inject_tmr_outlier, snapshot};

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

fn main() {
    let store = SharedStore::<u8, 3>::new(0);
    let mut protected = Hardened::<u8, Tmr, _, 3>::new(7, store.clone());

    println!("clean replicas: {:?}", snapshot(&store));

    inject_tmr_outlier(&store, 7, 1, 9).expect("valid outlier scenario");
    println!("after outlier: {:?}", snapshot(&store));
    println!("report: {:?}", protected.read_checked());

    let repair = protected.repair();
    assert_eq!(repair, RepairOutcome::Repaired);
    println!("after repair: {:?}", snapshot(&store));

    inject_tmr_no_majority(&store, [1, 2, 3]).expect("distinct no-majority pattern");
    println!("after no-majority: {:?}", snapshot(&store));

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("unexpected clean value: {value}"),
        ReadReport::Trusted {
            value,
            status: TrustedStatus::RecoverableMismatch,
        } => println!("recoverable trusted value: {value}"),
        ReadReport::Suspect { replicas, reason } => {
            println!("suspect replicas {replicas:?}: {reason:?}")
        }
    }
}
