#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadReport<T, const N: usize> {
    Trusted {
        value: T,
        status: TrustedStatus,
    },
    Suspect {
        replicas: [T; N],
        reason: SuspectReason,
    },
}

impl<T: Copy, const N: usize> ReadReport<T, N> {
    pub fn is_trusted(&self) -> bool {
        matches!(self, Self::Trusted { .. })
    }

    pub fn is_suspect(&self) -> bool {
        matches!(self, Self::Suspect { .. })
    }

    pub fn trusted_value(&self) -> Option<T> {
        match self {
            Self::Trusted { value, .. } => Some(*value),
            Self::Suspect { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustedStatus {
    Clean,
    RecoverableMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuspectReason {
    DmrConflict,
    NoMajority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckReport {
    Consistent,
    RecoverablyInconsistent,
    Suspect,
}

impl CheckReport {
    pub fn needs_repair(self) -> bool {
        matches!(self, Self::RecoverablyInconsistent)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairOutcome {
    NoRepairNeeded,
    Repaired,
    NotPossible,
}
