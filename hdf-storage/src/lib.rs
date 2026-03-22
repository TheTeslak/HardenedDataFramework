#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

pub mod load;
pub mod persist;
pub mod record;
pub mod report;

pub use load::{classify_record, load_into_core, load_pair};
pub use persist::{next_record, prepare_record};
pub use record::{PersistentRecord, RecordData};
pub use report::{LoadReason, LoadReport, RecordStatus, SlotId};
