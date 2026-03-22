#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

pub mod placement;
pub mod policy;
pub mod store;

pub use placement::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId};
pub use policy::{ComplementValue, ComplementedStore};
pub use store::SplitStore;
