use hdf_core::{
    CheckReport, Hardened, ReadReport, RepairOutcome, SuspectReason, Tmr, TrustedStatus,
};
use hdf_layout::{BankId, ComplementedStore, PlacementSite, RegionId, ReplicaPlacement, SectionId};

#[test]
fn complemented_store_round_trips_logical_values_and_exposes_encoded_slots() {
    let placement = ReplicaPlacement::with_sites([
        PlacementSite::with_details(RegionId(1), Some(BankId(0)), Some(SectionId(".r0"))),
        PlacementSite::with_details(RegionId(2), Some(BankId(1)), Some(SectionId(".r1"))),
        PlacementSite::new(RegionId(3)),
    ]);
    let store = ComplementedStore::<u8, 3>::new(0x3C, placement);

    assert_eq!(store.region_of(1), Some(RegionId(2)));
    assert_eq!(store.bank_of(1), Some(BankId(1)));
    assert_eq!(store.section_of(0), Some(SectionId(".r0")));
    assert_eq!(store.read_replicas(), [0x3C, 0x3C, 0x3C]);
    assert_eq!(store.encoded_replicas(), [!0x3C, !0x3C, !0x3C]);
}

#[test]
fn complemented_store_preserves_tmr_clean_recoverable_and_suspect_reports() {
    let store = ComplementedStore::<u8, 3>::new(
        0x55,
        ReplicaPlacement::new([RegionId(10), RegionId(20), RegionId(30)]),
    );
    let mut protected = Hardened::<u8, Tmr, _, 3>::new(0x55, &store);

    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 0x55,
            status: TrustedStatus::Clean,
        }
    );

    store.write_encoded_slot(2, !0x54);
    assert_eq!(store.read_replicas(), [0x55, 0x55, 0x54]);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Trusted {
            value: 0x55,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(protected.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(protected.repair(), RepairOutcome::Repaired);
    assert_eq!(store.read_replicas(), [0x55, 0x55, 0x55]);
    assert_eq!(store.encoded_replicas(), [!0x55, !0x55, !0x55]);

    store.write_encoded_slot(0, !0x10);
    store.write_encoded_slot(1, !0x20);
    store.write_encoded_slot(2, !0x30);
    assert_eq!(
        protected.read_checked(),
        ReadReport::Suspect {
            replicas: [0x10, 0x20, 0x30],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(protected.check(), CheckReport::Suspect);
    assert_eq!(protected.repair(), RepairOutcome::NotPossible);
}
