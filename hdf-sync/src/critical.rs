use core::cell::RefCell;

use hdf_core::{CheckReport, Hardened, ReadReport, RepairOutcome, ReplicaStore, Scheme};

use crate::access::{ReadAccess, WriteAccess};

pub trait CriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R;
}

#[derive(Debug)]
pub struct CriticalSectionHardened<C, T, S, Store, const N: usize> {
    critical_section: C,
    inner: RefCell<Hardened<T, S, Store, N>>,
}

impl<C, T, S, Store, const N: usize> CriticalSectionHardened<C, T, S, Store, N>
where
    C: CriticalSection,
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    pub fn from_hardened(critical_section: C, hardened: Hardened<T, S, Store, N>) -> Self {
        Self {
            critical_section,
            inner: RefCell::new(hardened),
        }
    }

    pub fn new(critical_section: C, initial: T, store: Store) -> Self {
        Self::from_hardened(critical_section, Hardened::new(initial, store))
    }

    pub fn with_read<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&ReadAccess<'_, T, S, Store, N>) -> R,
    {
        self.critical_section.enter(|| {
            let access = ReadAccess::new(self.inner.borrow());
            f(&access)
        })
    }

    pub fn with_write<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut WriteAccess<'_, T, S, Store, N>) -> R,
    {
        self.critical_section.enter(|| {
            let mut access = WriteAccess::new(self.inner.borrow_mut());
            f(&mut access)
        })
    }

    pub fn read_checked(&self) -> ReadReport<T, N> {
        self.with_read(|access| access.read_checked())
    }

    pub fn check(&self) -> CheckReport {
        self.with_read(|access| access.check())
    }

    pub fn write(&self, value: T) {
        self.with_write(|access| access.write(value));
    }

    pub fn repair(&self) -> RepairOutcome {
        self.with_write(|access| access.repair())
    }
}
