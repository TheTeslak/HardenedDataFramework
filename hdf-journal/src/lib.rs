#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

pub mod decode;
pub mod event;
pub mod writer;

pub use decode::{decode_record, render_record};
pub use event::{EncodedRecord, EventCode, EventRecord, JournalEvent, ResetCause, StorageSource};
pub use writer::{AppendError, Journal};
