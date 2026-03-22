use hdf_core::ReplicaStore;

use crate::FaultError;

pub fn corrupt_slot<T: Copy, Store, const N: usize>(
    store: &Store,
    index: usize,
    value: T,
) -> Result<(), FaultError>
where
    Store: ReplicaStore<T, N>,
{
    if index >= N {
        return Err(FaultError::SlotOutOfRange { index, replicas: N });
    }

    store.write_slot(index, value);
    Ok(())
}

pub fn apply_pattern<T: Copy, Store, const N: usize>(store: &Store, replicas: [T; N])
where
    Store: ReplicaStore<T, N>,
{
    for (index, value) in replicas.into_iter().enumerate() {
        store.write_slot(index, value);
    }
}

pub fn snapshot<T: Copy, Store, const N: usize>(store: &Store) -> [T; N]
where
    Store: ReplicaStore<T, N>,
{
    core::array::from_fn(|index| store.read_slot(index))
}
