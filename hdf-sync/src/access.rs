use core::cell::{Ref, RefMut};

use hdf_core::{CheckReport, Hardened, ReadReport, RepairOutcome, ReplicaStore, Scheme};

pub struct ReadAccess<'a, T, S, Store, const N: usize> {
    inner: Ref<'a, Hardened<T, S, Store, N>>,
}

impl<'a, T, S, Store, const N: usize> ReadAccess<'a, T, S, Store, N> {
    pub(crate) fn new(inner: Ref<'a, Hardened<T, S, Store, N>>) -> Self {
        Self { inner }
    }
}

impl<T, S, Store, const N: usize> ReadAccess<'_, T, S, Store, N>
where
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    pub fn read_checked(&self) -> ReadReport<T, N> {
        self.inner.read_checked()
    }

    pub fn check(&self) -> CheckReport {
        self.inner.check()
    }
}

pub struct WriteAccess<'a, T, S, Store, const N: usize> {
    inner: RefMut<'a, Hardened<T, S, Store, N>>,
}

impl<'a, T, S, Store, const N: usize> WriteAccess<'a, T, S, Store, N> {
    pub(crate) fn new(inner: RefMut<'a, Hardened<T, S, Store, N>>) -> Self {
        Self { inner }
    }
}

impl<T, S, Store, const N: usize> WriteAccess<'_, T, S, Store, N>
where
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    pub fn read_checked(&self) -> ReadReport<T, N> {
        self.inner.read_checked()
    }

    pub fn check(&self) -> CheckReport {
        self.inner.check()
    }

    pub fn write(&mut self, value: T) {
        self.inner.write(value);
    }

    pub fn repair(&mut self) -> RepairOutcome {
        self.inner.repair()
    }
}
