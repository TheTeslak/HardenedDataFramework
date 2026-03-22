use hdf_core::ReplicaStore;

use crate::error::FaultError;
use crate::inject::{apply_pattern, corrupt_slot};

pub fn inject_dmr_conflict<T: Copy, Store>(store: &Store, left: T, right: T)
where
    Store: ReplicaStore<T, 2>,
{
    apply_pattern(store, [left, right]);
}

pub fn inject_tmr_outlier<T: Copy, Store>(
    store: &Store,
    majority: T,
    outlier_index: usize,
    outlier: T,
) -> Result<(), FaultError>
where
    Store: ReplicaStore<T, 3>,
{
    apply_pattern(store, [majority; 3]);
    corrupt_slot(store, outlier_index, outlier)
}

pub fn inject_tmr_no_majority<T: Copy + Eq, Store>(
    store: &Store,
    replicas: [T; 3],
) -> Result<(), FaultError>
where
    Store: ReplicaStore<T, 3>,
{
    if replicas[0] == replicas[1] || replicas[0] == replicas[2] || replicas[1] == replicas[2] {
        return Err(FaultError::TmrNoMajorityRequiresDistinctValues);
    }

    apply_pattern(store, replicas);
    Ok(())
}
