use std::cell::Cell;
use std::rc::Rc;

use hdf_core::{Hardened, ReadReport, RepairOutcome, ReplicaStore, Tmr, TrustedStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standby,
    Active,
}

#[derive(Clone)]
struct DemoStore<T, const N: usize> {
    slots: Rc<[Cell<T>; N]>,
}

impl<T: Copy, const N: usize> DemoStore<T, N> {
    fn new(values: [T; N]) -> (Self, Rc<[Cell<T>; N]>) {
        let slots = Rc::new(values.map(Cell::new));
        (
            Self {
                slots: Rc::clone(&slots),
            },
            slots,
        )
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for DemoStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].get()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].set(value);
    }
}

fn main() {
    let (store, raw_slots) = DemoStore::new([Mode::Active, Mode::Active, Mode::Active]);
    let mut protected = Hardened::<Mode, Tmr, _, 3>::new(Mode::Standby, store);

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("clean mode: {value:?}"),
        report => println!("unexpected startup report: {report:?}"),
    }

    raw_slots[2].set(Mode::Active);

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::RecoverableMismatch,
        } => println!("recoverable mismatch, trusted mode: {value:?}"),
        report => println!("unexpected updated report: {report:?}"),
    }

    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    println!("repair outcome: {:?}", protected.read_checked());
}
