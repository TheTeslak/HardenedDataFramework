use hdf_core::{
    CheckReport, Dmr, Hardened, InlineStore, ReadReport, RepairOutcome, ReplicaStore,
    SuspectReason, Tmr, TrustedStatus,
};
use hdf_layout::{RegionId, ReplicaPlacement, SplitStore};

struct InlineStoreRef<'a, T, const N: usize>(&'a InlineStore<T, N>);

impl<T: Copy, const N: usize> ReplicaStore<T, N> for InlineStoreRef<'_, T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.0.read_slot(index)
    }

    fn write_slot(&self, index: usize, value: T) {
        self.0.write_slot(index, value);
    }
}

#[test]
fn dmr_split_store_matches_inline_store_reports() {
    let inline_store = InlineStore::<u8, 2>::new(0);
    let split_store = SplitStore::new(0u8, ReplicaPlacement::new([RegionId(0), RegionId(1)]));

    let mut inline = Hardened::<u8, Dmr, _, 2>::new(7, InlineStoreRef(&inline_store));
    let mut split = Hardened::<u8, Dmr, _, 2>::new(7, &split_store);

    assert_eq!(split.read_checked(), inline.read_checked());
    assert_eq!(split.check(), inline.check());
    let split_repair = split.repair();
    let inline_repair = inline.repair();
    assert_eq!(split_repair, inline_repair);

    inline_store.write_slot(1, 9);
    split_store.write_slot(1, 9);

    assert_eq!(
        split.read_checked(),
        ReadReport::Suspect {
            replicas: [7, 9],
            reason: SuspectReason::DmrConflict,
        }
    );
    assert_eq!(split.read_checked(), inline.read_checked());
    assert_eq!(split.check(), CheckReport::Suspect);
    assert_eq!(split.check(), inline.check());
    let split_repair = split.repair();
    let inline_repair = inline.repair();
    assert_eq!(split_repair, RepairOutcome::NotPossible);
    assert_eq!(split_repair, inline_repair);
    assert_eq!(split_store.read_replicas(), inline_store.read_replicas());
}

#[test]
fn tmr_split_store_matches_inline_store_reports_and_repair() {
    let inline_store = InlineStore::<u8, 3>::new(0);
    let split_store = SplitStore::new(
        0u8,
        ReplicaPlacement::new([RegionId(10), RegionId(20), RegionId(30)]),
    );

    let mut inline = Hardened::<u8, Tmr, _, 3>::new(5, InlineStoreRef(&inline_store));
    let mut split = Hardened::<u8, Tmr, _, 3>::new(5, &split_store);

    assert_eq!(
        split.read_checked(),
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::Clean,
        }
    );
    assert_eq!(split.read_checked(), inline.read_checked());
    assert_eq!(split.check(), inline.check());
    let split_repair = split.repair();
    let inline_repair = inline.repair();
    assert_eq!(split_repair, inline_repair);

    inline_store.write_slot(2, 8);
    split_store.write_slot(2, 8);

    assert_eq!(
        split.read_checked(),
        ReadReport::Trusted {
            value: 5,
            status: TrustedStatus::RecoverableMismatch,
        }
    );
    assert_eq!(split.read_checked(), inline.read_checked());
    assert_eq!(split.check(), CheckReport::RecoverablyInconsistent);
    assert_eq!(split.check(), inline.check());
    let split_repair = split.repair();
    let inline_repair = inline.repair();
    assert_eq!(split_repair, RepairOutcome::Repaired);
    assert_eq!(inline_repair, RepairOutcome::Repaired);
    assert_eq!(split_store.read_replicas(), inline_store.read_replicas());

    inline_store.write_slot(0, 1);
    inline_store.write_slot(1, 2);
    inline_store.write_slot(2, 3);
    split_store.write_slot(0, 1);
    split_store.write_slot(1, 2);
    split_store.write_slot(2, 3);

    assert_eq!(
        split.read_checked(),
        ReadReport::Suspect {
            replicas: [1, 2, 3],
            reason: SuspectReason::NoMajority,
        }
    );
    assert_eq!(split.read_checked(), inline.read_checked());
    assert_eq!(split.check(), CheckReport::Suspect);
    assert_eq!(split.check(), inline.check());
    let split_repair = split.repair();
    let inline_repair = inline.repair();
    assert_eq!(split_repair, RepairOutcome::NotPossible);
    assert_eq!(split_repair, inline_repair);
    assert_eq!(split_store.read_replicas(), inline_store.read_replicas());
}
