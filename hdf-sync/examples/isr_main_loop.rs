use std::cell::RefCell;
use std::rc::Rc;

use hdf_core::{ReadReport, ReplicaStore, Tmr, TrustedStatus};
use hdf_sync::{CriticalSection, CriticalSectionHardened};

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

struct DemoCriticalSection;

impl CriticalSection for DemoCriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standby,
    Active,
}

fn main() {
    let protected = CriticalSectionHardened::<_, Mode, Tmr, _, 3>::new(
        DemoCriticalSection,
        Mode::Standby,
        SharedStore::new(Mode::Standby),
    );

    protected.with_write(|access| access.write(Mode::Active));

    match protected.with_read(|access| access.read_checked()) {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("main loop sees mode {value:?}"),
        report => println!("unexpected report: {report:?}"),
    }
}
