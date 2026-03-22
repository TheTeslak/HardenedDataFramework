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

    fn set(&self, index: usize, value: T) {
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

fn main() {
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

    let store = DemoStore::<ControlConfig, 3>::new(clean);
    let mut protected = recoverable_in(clean, store.clone());

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("clean config: {value:?}"),
        report => println!("unexpected startup report: {report:?}"),
    }

    store.set(2, outlier);
    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::RecoverableMismatch,
        } => println!("trusted majority config: {value:?}"),
        report => println!("unexpected mismatch report: {report:?}"),
    }

    assert_eq!(protected.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(protected.repair(), RepairOutcome::Repaired);

    store.set(
        0,
        ControlConfig {
            mode: Mode::Standby,
            threshold: 850,
            revision: 9,
            enabled: false,
        },
    );

    println!("updated config report: {:?}", protected.read_checked());
}
