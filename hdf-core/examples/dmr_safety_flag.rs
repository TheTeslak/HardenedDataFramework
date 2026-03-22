use std::cell::Cell;
use std::rc::Rc;

use hdf_core::{Dmr, Hardened, ReadReport, ReplicaStore, SuspectReason, TrustedStatus};

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
    let (store, raw_slots) = DemoStore::new([false, false]);
    let protected = Hardened::<bool, Dmr, _, 2>::new(true, store);

    println!("clean read:");
    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("safety flag is trusted: {value}"),
        report => println!("unexpected report: {report:?}"),
    }

    raw_slots[1].set(false);

    println!("conflict read:");
    match protected.read_checked() {
        ReadReport::Suspect {
            reason: SuspectReason::DmrConflict,
            replicas,
        } => println!("safety flag conflict: {replicas:?}"),
        report => println!("unexpected report: {report:?}"),
    }
}
