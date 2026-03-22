use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ptr;

use hdf_core::ReplicaStore;

use crate::placement::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId};

pub trait ComplementValue: Copy + Eq {
    fn complement(self) -> Self;
}

macro_rules! impl_complement_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl ComplementValue for $ty {
                fn complement(self) -> Self {
                    !self
                }
            }
        )+
    };
}

impl_complement_value!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

pub struct ComplementedStore<T, const N: usize> {
    slots: [VolatileCell<T>; N],
    placement: ReplicaPlacement<N>,
    _marker: PhantomData<T>,
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

impl<T: ComplementValue, const N: usize> ComplementedStore<T, N> {
    pub fn new(initial: T, placement: ReplicaPlacement<N>) -> Self {
        let encoded = initial.complement();
        Self {
            slots: core::array::from_fn(|_| VolatileCell::new(encoded)),
            placement,
            _marker: PhantomData,
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

    pub fn encoded_replicas(&self) -> [T; N] {
        core::array::from_fn(|index| self.slots[index].read())
    }

    pub fn write_encoded_slot(&self, index: usize, value: T) {
        self.slots[index].write(value);
    }
}

impl<T: ComplementValue, const N: usize> ReplicaStore<T, N> for ComplementedStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].read().complement()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].write(value.complement());
    }
}

impl<T: ComplementValue, const N: usize> ReplicaStore<T, N> for &ComplementedStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].read().complement()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].write(value.complement());
    }
}
