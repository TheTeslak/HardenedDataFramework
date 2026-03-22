use core::cell::UnsafeCell;
use core::ptr;

use hdf_core::ReplicaStore;

use crate::placement::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId};

pub struct SplitStore<T, const N: usize> {
    slots: [VolatileCell<T>; N],
    placement: ReplicaPlacement<N>,
}

struct VolatileCell<T> {
    value: UnsafeCell<T>,
}

impl<T> VolatileCell<T> {
    fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile(self.value.get()) }
    }

    fn write(&self, value: T) {
        unsafe { ptr::write_volatile(self.value.get(), value) };
    }
}

impl<T: Copy, const N: usize> SplitStore<T, N> {
    pub fn new(initial: T, placement: ReplicaPlacement<N>) -> Self {
        Self {
            slots: core::array::from_fn(|_| VolatileCell::new(initial)),
            placement,
        }
    }

    pub fn placement(&self) -> &ReplicaPlacement<N> {
        &self.placement
    }

    pub fn region_of(&self, index: usize) -> Option<RegionId> {
        self.placement.region_of(index)
    }

    pub fn bank_of(&self, index: usize) -> Option<BankId> {
        self.placement.bank_of(index)
    }

    pub fn section_of(&self, index: usize) -> Option<SectionId> {
        self.placement.section_of(index)
    }

    pub fn site_of(&self, index: usize) -> Option<PlacementSite> {
        self.placement.site_of(index)
    }

    pub fn read_replicas(&self) -> [T; N] {
        core::array::from_fn(|index| self.read_slot(index))
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for SplitStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].read()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].write(value);
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for &SplitStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].read()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].write(value);
    }
}
