use core::cell::RefCell;

use hdf_core::{CheckReport, Hardened, ReadReport, RepairOutcome, ReplicaStore, Scheme};

use crate::access::{ReadAccess, WriteAccess};

#[derive(Debug)]
pub struct SerializedHardened<T, S, Store, const N: usize> {
    inner: RefCell<Hardened<T, S, Store, N>>,
}

impl<T, S, Store, const N: usize> SerializedHardened<T, S, Store, N>
where
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    pub fn from_hardened(hardened: Hardened<T, S, Store, N>) -> Self {
        Self {
            inner: RefCell::new(hardened),
        }
    }

    pub fn new(initial: T, store: Store) -> Self {
        Self::from_hardened(Hardened::new(initial, store))
    }

    pub fn with_read<R>(&self, f: impl FnOnce(&ReadAccess<'_, T, S, Store, N>) -> R) -> R {
        let access = ReadAccess::new(self.inner.borrow());
        f(&access)
    }

    pub fn with_write<R>(&self, f: impl FnOnce(&mut WriteAccess<'_, T, S, Store, N>) -> R) -> R {
        let mut access = WriteAccess::new(self.inner.borrow_mut());
        f(&mut access)
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
