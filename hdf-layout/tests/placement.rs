use hdf_core::ReplicaStore;
use hdf_layout::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId, SplitStore};

#[test]
fn placement_maps_replicas_to_regions() {
    let placement = ReplicaPlacement::new([RegionId(10), RegionId(20), RegionId(30)]);

    assert_eq!(
        placement.regions(),
        [RegionId(10), RegionId(20), RegionId(30)]
    );
    assert_eq!(placement.region_of(0), Some(RegionId(10)));
    assert_eq!(placement.region_of(1), Some(RegionId(20)));
    assert_eq!(placement.region_of(2), Some(RegionId(30)));
    assert_eq!(placement.region_of(3), None);
}

#[test]
fn placement_tracks_optional_bank_and_section_metadata() {
    let placement = ReplicaPlacement::with_sites([
        PlacementSite::with_details(RegionId(1), Some(BankId(0)), Some(SectionId(".data_a"))),
        PlacementSite::with_details(RegionId(2), Some(BankId(1)), Some(SectionId(".data_b"))),
        PlacementSite::new(RegionId(3)),
    ]);

    assert_eq!(placement.site_of(0).unwrap().region(), RegionId(1));
    assert_eq!(placement.bank_of(0), Some(BankId(0)));
    assert_eq!(placement.section_of(1), Some(SectionId(".data_b")));
    assert_eq!(placement.bank_of(2), None);
    assert_eq!(placement.section_of(3), None);
}

#[test]
fn split_store_exposes_placement_and_access_helpers() {
    let store = SplitStore::new(
        0u8,
        ReplicaPlacement::with_sites([
            PlacementSite::with_details(RegionId(1), Some(BankId(0)), Some(SectionId(".bank0"))),
            PlacementSite::with_details(RegionId(4), Some(BankId(1)), Some(SectionId(".bank1"))),
            PlacementSite::new(RegionId(9)),
        ]),
    );

    assert_eq!(
        store.placement().regions(),
        [RegionId(1), RegionId(4), RegionId(9)]
    );
    assert_eq!(store.region_of(1), Some(RegionId(4)));
    assert_eq!(store.bank_of(1), Some(BankId(1)));
    assert_eq!(store.section_of(0), Some(SectionId(".bank0")));
    assert_eq!(store.site_of(2).unwrap().region(), RegionId(9));
    assert_eq!(store.region_of(3), None);

    store.write_slot(0, 7);
    store.write_slot(1, 8);
    store.write_slot(2, 9);

    assert_eq!(store.read_replicas(), [7, 8, 9]);
}
