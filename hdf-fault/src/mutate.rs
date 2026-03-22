use hdf_core::ReplicaStore;

use crate::FaultError;

pub trait BitFlip: Copy {
    const BIT_WIDTH: u32;

    fn bit_mask(bit: u32) -> Self;
    fn xor_mask(self, mask: Self) -> Self;
}

macro_rules! impl_bit_flip {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl BitFlip for $ty {
                const BIT_WIDTH: u32 = <$ty>::BITS;

                fn bit_mask(bit: u32) -> Self {
                    1 as $ty << bit
                }

                fn xor_mask(self, mask: Self) -> Self {
                    self ^ mask
                }
            }
        )+
    };
}

impl_bit_flip!(u8, u16, u32, u64, u128, usize);

pub fn mutate_slot<T: Copy, Store, const N: usize>(
    store: &Store,
    index: usize,
    mutate: impl FnOnce(T) -> T,
) -> Result<T, FaultError>
where
    Store: ReplicaStore<T, N>,
{
    if index >= N {
        return Err(FaultError::SlotOutOfRange { index, replicas: N });
    }

    let updated = mutate(store.read_slot(index));
    store.write_slot(index, updated);
    Ok(updated)
}

pub fn flip_bool_slot<Store, const N: usize>(
    store: &Store,
    index: usize,
) -> Result<bool, FaultError>
where
    Store: ReplicaStore<bool, N>,
{
    mutate_slot(store, index, |value| !value)
}

pub fn xor_mask_slot<T, Store, const N: usize>(
    store: &Store,
    index: usize,
    mask: T,
) -> Result<T, FaultError>
where
    T: BitFlip,
    Store: ReplicaStore<T, N>,
{
    mutate_slot(store, index, |value| value.xor_mask(mask))
}

pub fn flip_bit_in_slot<T, Store, const N: usize>(
    store: &Store,
    index: usize,
    bit: u32,
) -> Result<T, FaultError>
where
    T: BitFlip,
    Store: ReplicaStore<T, N>,
{
    if bit >= T::BIT_WIDTH {
        return Err(FaultError::BitOutOfRange {
            bit,
            width: T::BIT_WIDTH,
        });
    }

    xor_mask_slot(store, index, T::bit_mask(bit))
}
