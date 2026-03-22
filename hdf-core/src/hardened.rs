use core::marker::PhantomData;

use crate::report::{CheckReport, ReadReport, RepairOutcome};
use crate::scheme::{Scheme, repair_outcome};
use crate::store::ReplicaStore;

#[derive(Debug)]
pub struct Hardened<T, S, Store, const N: usize> {
    store: Store,
    _marker: PhantomData<(T, S)>,
}

impl<T, S, Store, const N: usize> Hardened<T, S, Store, N>
where
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    pub fn new(initial: T, store: Store) -> Self {
        let hardened = Self {
            store,
            _marker: PhantomData,
        };

        for index in 0..N {
            hardened.store.write_slot(index, initial);
        }

        hardened
    }

    pub fn read_checked(&self) -> ReadReport<T, N> {
        S::read_report(self.read_replicas())
    }

    pub fn check(&self) -> CheckReport {
        S::check(&self.read_replicas())
    }

    pub fn write(&mut self, value: T) {
        for index in 0..N {
            self.store.write_slot(index, value);
        }
    }

    pub fn repair(&mut self) -> RepairOutcome {
        let replicas = self.read_replicas();
        let outcome = repair_outcome::<N, S, T>(&replicas);

        if matches!(outcome, RepairOutcome::Repaired) {
            let value = S::repair_value(&replicas)
                .expect("recoverable state must yield a trusted repair value");
            for index in 0..N {
                self.store.write_slot(index, value);
            }
        }

        outcome
    }

    fn read_replicas(&self) -> [T; N] {
        core::array::from_fn(|index| self.store.read_slot(index))
    }
}

#[cfg(test)]
impl<T, S, Store, const N: usize> Hardened<T, S, Store, N>
where
    T: Copy + Eq,
    S: Scheme<N>,
    Store: ReplicaStore<T, N>,
{
    fn debug_read_replicas(&self) -> [T; N] {
        self.read_replicas()
    }

    fn debug_write_replica(&self, index: usize, value: T) {
        self.store.write_slot(index, value);
    }
}

#[cfg(test)]
mod tests {
    use crate::report::{CheckReport, ReadReport, RepairOutcome, SuspectReason, TrustedStatus};
    use crate::scheme::{Dmr, Tmr};
    use crate::store::InlineStore;

    use super::Hardened;

    #[test]
    fn new_initializes_every_dmr_replica() {
        let hardened = Hardened::<u8, Dmr, _, 2>::new(7, InlineStore::new(0));
        assert_eq!(hardened.debug_read_replicas(), [7, 7]);
    }

    #[test]
    fn new_initializes_every_tmr_replica() {
        let hardened = Hardened::<u8, Tmr, _, 3>::new(7, InlineStore::new(0));
        assert_eq!(hardened.debug_read_replicas(), [7, 7, 7]);
    }

    #[test]
    fn dmr_read_matrix_matches_spec() {
        let hardened = Hardened::<u8, Dmr, _, 2>::new(1, InlineStore::new(0));
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 1,
                status: TrustedStatus::Clean,
            }
        );

        hardened.debug_write_replica(1, 2);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Suspect {
                replicas: [1, 2],
                reason: SuspectReason::DmrConflict,
            }
        );
    }

    #[test]
    fn dmr_check_and_repair_matrix_matches_spec() {
        let mut hardened = Hardened::<u8, Dmr, _, 2>::new(9, InlineStore::new(0));
        assert_eq!(hardened.check(), CheckReport::Consistent);
        assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);

        hardened.debug_write_replica(1, 3);
        assert_eq!(hardened.check(), CheckReport::Suspect);
        assert_eq!(hardened.repair(), RepairOutcome::NotPossible);
        assert_eq!(hardened.debug_read_replicas(), [9, 3]);
    }

    #[test]
    fn tmr_read_matrix_matches_spec() {
        let hardened = Hardened::<u8, Tmr, _, 3>::new(4, InlineStore::new(0));
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 4,
                status: TrustedStatus::Clean,
            }
        );

        hardened.debug_write_replica(2, 8);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 4,
                status: TrustedStatus::RecoverableMismatch,
            }
        );

        hardened.debug_write_replica(1, 8);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 8,
                status: TrustedStatus::RecoverableMismatch,
            }
        );

        hardened.debug_write_replica(0, 1);
        hardened.debug_write_replica(1, 2);
        hardened.debug_write_replica(2, 3);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Suspect {
                replicas: [1, 2, 3],
                reason: SuspectReason::NoMajority,
            }
        );
    }

    #[test]
    fn tmr_truth_table_is_exhaustive() {
        let hardened = Hardened::<u8, Tmr, _, 3>::new(1, InlineStore::new(0));

        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 1,
                status: TrustedStatus::Clean,
            }
        );

        hardened.debug_write_replica(2, 2);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 1,
                status: TrustedStatus::RecoverableMismatch,
            }
        );

        hardened.debug_write_replica(1, 2);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Trusted {
                value: 2,
                status: TrustedStatus::RecoverableMismatch,
            }
        );

        hardened.debug_write_replica(0, 3);
        hardened.debug_write_replica(1, 4);
        hardened.debug_write_replica(2, 5);
        assert_eq!(
            hardened.read_checked(),
            ReadReport::Suspect {
                replicas: [3, 4, 5],
                reason: SuspectReason::NoMajority,
            }
        );
    }

    #[test]
    fn tmr_check_and_repair_matrix_matches_spec() {
        let mut hardened = Hardened::<u8, Tmr, _, 3>::new(5, InlineStore::new(0));
        assert_eq!(hardened.check(), CheckReport::Consistent);
        assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);

        hardened.debug_write_replica(2, 9);
        assert_eq!(hardened.check(), CheckReport::RecoverablyInconsistent);
        assert_eq!(hardened.repair(), RepairOutcome::Repaired);
        assert_eq!(hardened.debug_read_replicas(), [5, 5, 5]);

        hardened.debug_write_replica(0, 1);
        hardened.debug_write_replica(1, 2);
        hardened.debug_write_replica(2, 3);
        assert_eq!(hardened.check(), CheckReport::Suspect);
        assert_eq!(hardened.repair(), RepairOutcome::NotPossible);
        assert_eq!(hardened.debug_read_replicas(), [1, 2, 3]);
    }

    #[test]
    fn read_checked_and_check_do_not_mutate_storage() {
        let hardened = Hardened::<u8, Tmr, _, 3>::new(2, InlineStore::new(0));
        hardened.debug_write_replica(1, 3);

        let before = hardened.debug_read_replicas();
        let _ = hardened.read_checked();
        let after_read = hardened.debug_read_replicas();
        let _ = hardened.check();
        let after_check = hardened.debug_read_replicas();

        assert_eq!(before, [2, 3, 2]);
        assert_eq!(after_read, before);
        assert_eq!(after_check, before);
    }

    #[test]
    fn write_overwrites_all_replicas_after_corruption() {
        let mut hardened = Hardened::<u8, Tmr, _, 3>::new(6, InlineStore::new(0));
        hardened.debug_write_replica(0, 1);
        hardened.debug_write_replica(1, 2);
        hardened.write(9);
        assert_eq!(hardened.debug_read_replicas(), [9, 9, 9]);
    }

    #[test]
    fn repeated_repair_is_idempotent_after_fixing_tmr() {
        let mut hardened = Hardened::<u8, Tmr, _, 3>::new(4, InlineStore::new(0));
        hardened.debug_write_replica(2, 8);
        assert_eq!(hardened.repair(), RepairOutcome::Repaired);
        assert_eq!(hardened.repair(), RepairOutcome::NoRepairNeeded);
        assert_eq!(hardened.debug_read_replicas(), [4, 4, 4]);
    }
}
