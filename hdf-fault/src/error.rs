use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultError {
    SlotOutOfRange { index: usize, replicas: usize },
    BitOutOfRange { bit: u32, width: u32 },
    TmrNoMajorityRequiresDistinctValues,
}

impl fmt::Display for FaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotOutOfRange { index, replicas } => {
                write!(
                    f,
                    "slot index {index} is out of range for {replicas} replicas"
                )
            }
            Self::BitOutOfRange { bit, width } => {
                write!(f, "bit index {bit} is out of range for a {width}-bit value")
            }
            Self::TmrNoMajorityRequiresDistinctValues => {
                f.write_str("TMR no-majority scenarios require three distinct values")
            }
        }
    }
}
