use core::fmt;

pub const ENCODED_RECORD_LEN: usize = 12;

pub type EncodedRecord = [u8; ENCODED_RECORD_LEN];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageSource {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResetCause {
    PowerOn,
    Watchdog,
    Software,
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalEvent {
    IntegrityClean,
    RecoverableMismatch,
    SuspectNoTrustedValue,
    RepairAttempted,
    RepairSucceeded,
    RepairNotPossible,
    StorageRecordSelected { source: StorageSource, version: u32 },
    StorageRecoveryFailed,
    ResetObserved { cause: ResetCause },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum EventCode {
    IntegrityClean = 1,
    RecoverableMismatch = 2,
    SuspectNoTrustedValue = 3,
    RepairAttempted = 4,
    RepairSucceeded = 5,
    RepairNotPossible = 6,
    StorageRecordSelected = 7,
    StorageRecoveryFailed = 8,
    ResetObserved = 9,
}

impl EventCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::IntegrityClean),
            2 => Some(Self::RecoverableMismatch),
            3 => Some(Self::SuspectNoTrustedValue),
            4 => Some(Self::RepairAttempted),
            5 => Some(Self::RepairSucceeded),
            6 => Some(Self::RepairNotPossible),
            7 => Some(Self::StorageRecordSelected),
            8 => Some(Self::StorageRecoveryFailed),
            9 => Some(Self::ResetObserved),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EventRecord {
    pub sequence: u32,
    pub event: JournalEvent,
}

impl EventRecord {
    pub fn encode(self) -> EncodedRecord {
        let mut encoded = [0u8; ENCODED_RECORD_LEN];
        encoded[0..4].copy_from_slice(&self.sequence.to_le_bytes());

        match self.event {
            JournalEvent::IntegrityClean => encoded[4] = EventCode::IntegrityClean as u8,
            JournalEvent::RecoverableMismatch => encoded[4] = EventCode::RecoverableMismatch as u8,
            JournalEvent::SuspectNoTrustedValue => {
                encoded[4] = EventCode::SuspectNoTrustedValue as u8
            }
            JournalEvent::RepairAttempted => encoded[4] = EventCode::RepairAttempted as u8,
            JournalEvent::RepairSucceeded => encoded[4] = EventCode::RepairSucceeded as u8,
            JournalEvent::RepairNotPossible => encoded[4] = EventCode::RepairNotPossible as u8,
            JournalEvent::StorageRecordSelected { source, version } => {
                encoded[4] = EventCode::StorageRecordSelected as u8;
                encoded[5] = match source {
                    StorageSource::Primary => 0,
                    StorageSource::Secondary => 1,
                };
                encoded[8..12].copy_from_slice(&version.to_le_bytes());
            }
            JournalEvent::StorageRecoveryFailed => {
                encoded[4] = EventCode::StorageRecoveryFailed as u8
            }
            JournalEvent::ResetObserved { cause } => {
                encoded[4] = EventCode::ResetObserved as u8;
                encoded[5] = match cause {
                    ResetCause::PowerOn => 0,
                    ResetCause::Watchdog => 1,
                    ResetCause::Software => 2,
                    ResetCause::Unknown(code) => code,
                };
            }
        }

        encoded
    }

    pub fn decode(encoded: EncodedRecord) -> Option<Self> {
        let sequence = u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        let code = EventCode::from_u8(encoded[4])?;

        let event = match code {
            EventCode::IntegrityClean => JournalEvent::IntegrityClean,
            EventCode::RecoverableMismatch => JournalEvent::RecoverableMismatch,
            EventCode::SuspectNoTrustedValue => JournalEvent::SuspectNoTrustedValue,
            EventCode::RepairAttempted => JournalEvent::RepairAttempted,
            EventCode::RepairSucceeded => JournalEvent::RepairSucceeded,
            EventCode::RepairNotPossible => JournalEvent::RepairNotPossible,
            EventCode::StorageRecordSelected => JournalEvent::StorageRecordSelected {
                source: if encoded[5] == 0 {
                    StorageSource::Primary
                } else {
                    StorageSource::Secondary
                },
                version: u32::from_le_bytes([encoded[8], encoded[9], encoded[10], encoded[11]]),
            },
            EventCode::StorageRecoveryFailed => JournalEvent::StorageRecoveryFailed,
            EventCode::ResetObserved => JournalEvent::ResetObserved {
                cause: match encoded[5] {
                    0 => ResetCause::PowerOn,
                    1 => ResetCause::Watchdog,
                    2 => ResetCause::Software,
                    other => ResetCause::Unknown(other),
                },
            },
        };

        Some(Self { sequence, event })
    }
}

impl fmt::Display for JournalEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegrityClean => f.write_str("integrity clean"),
            Self::RecoverableMismatch => f.write_str("recoverable mismatch"),
            Self::SuspectNoTrustedValue => f.write_str("suspect/no trusted value"),
            Self::RepairAttempted => f.write_str("repair attempted"),
            Self::RepairSucceeded => f.write_str("repair succeeded"),
            Self::RepairNotPossible => f.write_str("repair not possible"),
            Self::StorageRecordSelected { source, version } => {
                write!(f, "storage selected {:?} record v{}", source, version)
            }
            Self::StorageRecoveryFailed => f.write_str("storage recovery failed"),
            Self::ResetObserved { cause } => write!(f, "reset observed: {:?}", cause),
        }
    }
}

impl fmt::Display for EventRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} {}", self.sequence, self.event)
    }
}
