#![doc = include_str!("../README.md")]

pub mod app;
pub mod critical;
pub mod model;
pub mod store;

pub use app::ReferenceApp;
pub use critical::DemoCriticalSection;
pub use model::{ControlConfig, ControlMode, CycleOutcome};
pub use store::SharedLayoutStore;
