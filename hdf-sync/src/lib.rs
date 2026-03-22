#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

pub mod access;
pub mod critical;
pub mod wrapper;

pub use access::{ReadAccess, WriteAccess};
pub use critical::{CriticalSection, CriticalSectionHardened};
pub use hdf_core::{
    CheckReport, Dmr, Hardened, InlineStore, ReadReport, RepairOutcome, ReplicaStore, Scheme,
    SuspectReason, Tmr, TrustedStatus,
};
pub use wrapper::SerializedHardened;

/// ```compile_fail
/// use hdf_core::{InlineStore, Tmr};
/// use hdf_sync::SerializedHardened;
///
/// fn assert_sync<T: Sync>() {}
///
/// assert_sync::<SerializedHardened<u8, Tmr, InlineStore<u8, 3>, 3>>();
/// ```
///
/// `SerializedHardened` uses `RefCell` to serialize access through closures.
/// That keeps access scoped, but it does not make the wrapper `Sync`.
fn _doc_tests() {}
