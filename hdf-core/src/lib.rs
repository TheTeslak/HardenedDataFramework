#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

mod cell;
pub mod ergonomic;
pub mod hardened;
pub mod report;
pub mod scheme;
pub mod store;

pub use ergonomic::{
    DetectOnly, DetectOnlyInline, Recoverable, RecoverableInline, detect_only, detect_only_in,
    recoverable, recoverable_in,
};
pub use hardened::Hardened;
pub use report::{CheckReport, ReadReport, RepairOutcome, SuspectReason, TrustedStatus};
pub use scheme::{Dmr, Scheme, Tmr};
pub use store::{InlineStore, ReplicaStore};

/// ```compile_fail
/// use hdf_core::recoverable;
///
/// let _ = recoverable(1.0f32);
/// ```
///
/// Phase-1/6 support remains limited to `Copy + Eq` values. Ambiguous float semantics stay out of scope.
fn _unsupported_type_doc_tests() {}
