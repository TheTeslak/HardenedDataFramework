use hdf_core::SuspectReason;
use hdf_storage::RecordData;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlMode {
    Standby,
    Active,
    Safe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlConfig {
    pub mode: ControlMode,
    pub threshold: u16,
    pub revision: u8,
    pub output_enabled: bool,
}

impl ControlConfig {
    pub const fn new(
        mode: ControlMode,
        threshold: u16,
        revision: u8,
        output_enabled: bool,
    ) -> Self {
        Self {
            mode,
            threshold,
            revision,
            output_enabled,
        }
    }
}

impl RecordData for ControlConfig {
    fn checksum(&self) -> u32 {
        let mode = match self.mode {
            ControlMode::Standby => 1u32,
            ControlMode::Active => 2u32,
            ControlMode::Safe => 3u32,
        };

        mode ^ (self.threshold as u32)
            ^ ((self.revision as u32) << 8)
            ^ ((self.output_enabled as u32) << 16)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CycleOutcome {
    Nominal {
        config: ControlConfig,
    },
    Recovered {
        config: ControlConfig,
    },
    SafeHold {
        reason: SuspectReason,
        replicas: [ControlConfig; 3],
    },
}
