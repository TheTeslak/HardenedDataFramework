use std::mem::size_of;

use hdf_journal::Journal;
use hdf_reference::{ControlConfig, ReferenceApp, SharedLayoutStore};

fn main() {
    println!("Reference integration footprint summary");
    println!("- ControlConfig: {} bytes", size_of::<ControlConfig>());
    println!(
        "- SharedLayoutStore<ControlConfig, 3>: {} bytes",
        size_of::<SharedLayoutStore<ControlConfig, 3>>()
    );
    println!("- Journal<16>: {} bytes", size_of::<Journal<16>>());
    println!(
        "- ReferenceApp<16>: {} bytes",
        size_of::<ReferenceApp<16>>()
    );
}
