use std::cell::{Cell, RefCell};
use std::rc::Rc;

use hdf_core::{CheckReport, ReadReport, RepairOutcome, ReplicaStore, Tmr, TrustedStatus};
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

    fn corrupt(&self, index: usize, value: T) {
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

#[derive(Clone)]
struct CountingCriticalSection {
    entries: Rc<Cell<usize>>,
}

impl CountingCriticalSection {
    fn new() -> Self {
        Self {
            entries: Rc::new(Cell::new(0)),
        }
    }

    fn entry_count(&self) -> usize {
        self.entries.get()
    }
}

impl CriticalSection for CountingCriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.entries.set(self.entries.get() + 1);
        f()
    }
}

#[test]
fn critical_section_wrapper_serializes_access_through_entry_hook() {
    let gate = CountingCriticalSection::new();
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = CriticalSectionHardened::<_, u8, Tmr, _, 3>::new(gate.clone(), 7, store.clone());

    let report = hardened.with_read(|access| access.read_checked());
    assert_eq!(
        report,
        ReadReport::Trusted {
            value: 7,
            status: TrustedStatus::Clean,
        }
    );

    store.corrupt(1, 9);
    let outcome = hardened.with_write(|access| {
        assert_eq!(access.check(), CheckReport::RecoverablyInconsistent);
        access.repair()
    });
    assert_eq!(outcome, RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [7, 7, 7]);
    assert_eq!(gate.entry_count(), 2);
}

#[test]
fn critical_section_wrapper_convenience_methods_match_guarded_access() {
    let gate = CountingCriticalSection::new();
    let store = SharedStore::<u8, 3>::new(0);
    let hardened = CriticalSectionHardened::<_, u8, Tmr, _, 3>::new(gate.clone(), 4, store.clone());

    hardened.write(8);
    assert_eq!(hardened.read_checked().trusted_value(), Some(8));

    store.corrupt(2, 2);
    assert_eq!(hardened.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(hardened.repair(), RepairOutcome::Repaired);
    assert_eq!(store.snapshot(), [8, 8, 8]);
    assert!(gate.entry_count() >= 4);
}
