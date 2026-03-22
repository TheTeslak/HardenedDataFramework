use crate::report::{CheckReport, ReadReport, RepairOutcome, SuspectReason, TrustedStatus};

pub trait Scheme<const N: usize> {
    fn read_report<T: Copy + Eq>(replicas: [T; N]) -> ReadReport<T, N>;
    fn check<T: Copy + Eq>(replicas: &[T; N]) -> CheckReport;
    fn repair_value<T: Copy + Eq>(replicas: &[T; N]) -> Option<T>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Dmr;

impl Scheme<2> for Dmr {
    fn read_report<T: Copy + Eq>(replicas: [T; 2]) -> ReadReport<T, 2> {
        let [a, b] = replicas;
        if a == b {
            ReadReport::Trusted {
                value: a,
                status: TrustedStatus::Clean,
            }
        } else {
            ReadReport::Suspect {
                replicas: [a, b],
                reason: SuspectReason::DmrConflict,
            }
        }
    }

    fn check<T: Copy + Eq>(replicas: &[T; 2]) -> CheckReport {
        let [a, b] = *replicas;
        if a == b {
            CheckReport::Consistent
        } else {
            CheckReport::Suspect
        }
    }

    fn repair_value<T: Copy + Eq>(replicas: &[T; 2]) -> Option<T> {
        let [a, b] = *replicas;
        if a == b { Some(a) } else { None }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tmr;

impl Scheme<3> for Tmr {
    fn read_report<T: Copy + Eq>(replicas: [T; 3]) -> ReadReport<T, 3> {
        let [a, b, c] = replicas;
        if a == b && b == c {
            return ReadReport::Trusted {
                value: a,
                status: TrustedStatus::Clean,
            };
        }

        if let Some(value) = Self::repair_value(&replicas) {
            return ReadReport::Trusted {
                value,
                status: TrustedStatus::RecoverableMismatch,
            };
        }

        ReadReport::Suspect {
            replicas,
            reason: SuspectReason::NoMajority,
        }
    }

    fn check<T: Copy + Eq>(replicas: &[T; 3]) -> CheckReport {
        let [a, b, c] = *replicas;
        if a == b && b == c {
            CheckReport::Consistent
        } else if a == b || a == c || b == c {
            CheckReport::RecoverablyInconsistent
        } else {
            CheckReport::Suspect
        }
    }

    fn repair_value<T: Copy + Eq>(replicas: &[T; 3]) -> Option<T> {
        let [a, b, c] = *replicas;
        if a == b || a == c {
            Some(a)
        } else if b == c {
            Some(b)
        } else {
            None
        }
    }
}

pub(crate) fn repair_outcome<const N: usize, S, T>(replicas: &[T; N]) -> RepairOutcome
where
    S: Scheme<N>,
    T: Copy + Eq,
{
    match S::check(replicas) {
        CheckReport::Consistent => RepairOutcome::NoRepairNeeded,
        CheckReport::RecoverablyInconsistent => RepairOutcome::Repaired,
        CheckReport::Suspect => RepairOutcome::NotPossible,
    }
}
