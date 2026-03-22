use std::rc::Rc;

use hdf_core::ReplicaStore;
use hdf_layout::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId, SplitStore};

#[derive(Clone)]
pub struct SharedLayoutStore<T, const N: usize> {
    inner: Rc<SplitStore<T, N>>,
}

impl<T: Copy, const N: usize> SharedLayoutStore<T, N> {
    pub fn new(initial: T, placement: ReplicaPlacement<N>) -> Self {
        Self {
            inner: Rc::new(SplitStore::new(initial, placement)),
        }
    }

    pub fn placement(&self) -> &ReplicaPlacement<N> {
        self.inner.placement()
    }

    pub fn region_of(&self, index: usize) -> Option<RegionId> {
        self.inner.region_of(index)
    }

    pub fn bank_of(&self, index: usize) -> Option<BankId> {
        self.inner.bank_of(index)
    }

    pub fn section_of(&self, index: usize) -> Option<SectionId> {
        self.inner.section_of(index)
    }

    pub fn site_of(&self, index: usize) -> Option<PlacementSite> {
        self.inner.site_of(index)
    }

    pub fn snapshot(&self) -> [T; N] {
        self.inner.read_replicas()
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for SharedLayoutStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.inner.read_slot(index)
    }

    fn write_slot(&self, index: usize, value: T) {
        self.inner.write_slot(index, value);
    }
}
