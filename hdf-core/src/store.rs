use crate::cell::VolatileCell;

pub trait ReplicaStore<T, const N: usize> {
    fn read_slot(&self, index: usize) -> T;
    fn write_slot(&self, index: usize, value: T);
}

#[derive(Debug)]
pub struct InlineStore<T, const N: usize> {
    slots: [VolatileCell<T>; N],
}

impl<T: Copy, const N: usize> InlineStore<T, N> {
    pub fn new(initial: T) -> Self {
        Self {
            slots: core::array::from_fn(|_| VolatileCell::new(initial)),
        }
    }

    pub fn read_replicas(&self) -> [T; N] {
        core::array::from_fn(|index| self.read_slot(index))
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for InlineStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.slots[index].read()
    }

    fn write_slot(&self, index: usize, value: T) {
        self.slots[index].write(value);
    }
}
