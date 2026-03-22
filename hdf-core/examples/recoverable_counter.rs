use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{
    CheckReport, ReadReport, RepairOutcome, ReplicaStore, TrustedStatus, recoverable_in,
};

#[derive(Clone)]
struct DemoStore<T, const N: usize> {
    replicas: Rc<RefCell<[T; N]>>,
}

impl<T: Copy, const N: usize> DemoStore<T, N> {
    fn new(initial: T) -> Self {
        Self {
            replicas: Rc::new(RefCell::new([initial; N])),
        }
    }

    fn corrupt(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for DemoStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.replicas.borrow()[index]
    }

    fn write_slot(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
    }
}

fn main() {
    let store = DemoStore::<u32, 3>::new(0);
    let mut counter = recoverable_in(120, store.clone());

    match counter.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("clean counter: {value}"),
        report => println!("unexpected startup report: {report:?}"),
    }

    store.corrupt(2, 121);

    match counter.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::RecoverableMismatch,
        } => println!("trusted counter with one outlier: {value}"),
        report => println!("unexpected mismatch report: {report:?}"),
    }

    assert_eq!(counter.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(counter.repair(), RepairOutcome::Repaired);

    counter.write(122);
    println!("advanced counter: {:?}", counter.read_checked());
}
