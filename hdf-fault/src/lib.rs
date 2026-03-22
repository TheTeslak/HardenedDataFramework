#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

pub mod error;
pub mod inject;
pub mod mutate;
pub mod scenario;

pub use error::FaultError;
pub use inject::{apply_pattern, corrupt_slot, snapshot};
pub use mutate::{BitFlip, flip_bit_in_slot, flip_bool_slot, mutate_slot, xor_mask_slot};
pub use scenario::{inject_dmr_conflict, inject_tmr_no_majority, inject_tmr_outlier};
