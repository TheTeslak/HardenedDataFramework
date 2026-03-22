use core::cell::UnsafeCell;
use core::ptr;

#[derive(Debug)]
pub(crate) struct VolatileCell<T> {
    value: UnsafeCell<T>,
}

impl<T> VolatileCell<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub(crate) fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile(self.value.get()) }
    }

    pub(crate) fn write(&self, value: T) {
        unsafe { ptr::write_volatile(self.value.get(), value) };
    }
}
