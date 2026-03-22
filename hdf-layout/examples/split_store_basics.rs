use hdf_core::{Hardened, ReadReport, Tmr, TrustedStatus};
use hdf_layout::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId, SplitStore};

fn main() {
    let placement = ReplicaPlacement::with_sites([
        PlacementSite::with_details(RegionId(100), Some(BankId(0)), Some(SectionId(".mode_a"))),
        PlacementSite::with_details(RegionId(200), Some(BankId(1)), Some(SectionId(".mode_b"))),
        PlacementSite::new(RegionId(300)),
    ]);
    let store = SplitStore::new(0u8, placement);

    for index in 0..3 {
        println!(
            "replica {index} -> region {:?}, bank {:?}, section {:?}",
            store.region_of(index),
            store.bank_of(index),
            store.section_of(index)
        );
    }

    let protected = Hardened::<u8, Tmr, _, 3>::new(42, store);

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("trusted clean value: {value}"),
        report => println!("unexpected report: {report:?}"),
    }
}
