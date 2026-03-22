use hdf_core::{ReadReport, RepairOutcome, Tmr};
use hdf_fault::{apply_pattern, corrupt_slot};
use hdf_journal::{EventRecord, Journal, JournalEvent, ResetCause, StorageSource, decode_record};
use hdf_layout::{BankId, PlacementSite, RegionId, ReplicaPlacement, SectionId};
use hdf_storage::{LoadReason, LoadReport, PersistentRecord, SlotId, load_into_core};
use hdf_sync::CriticalSectionHardened;

use crate::critical::DemoCriticalSection;
use crate::model::{ControlConfig, ControlMode, CycleOutcome};
use crate::store::SharedLayoutStore;

pub struct ReferenceApp<const LOG: usize> {
    runtime_store: SharedLayoutStore<ControlConfig, 3>,
    protected: CriticalSectionHardened<
        DemoCriticalSection,
        ControlConfig,
        Tmr,
        SharedLayoutStore<ControlConfig, 3>,
        3,
    >,
    journal: Journal<LOG>,
    boot_report: LoadReport<ControlConfig>,
    outputs_enabled: bool,
}

impl<const LOG: usize> ReferenceApp<LOG> {
    pub fn boot(
        primary: PersistentRecord<ControlConfig>,
        secondary: PersistentRecord<ControlConfig>,
    ) -> Result<Self, LoadReport<ControlConfig>> {
        Self::boot_with_placement(primary, secondary, default_placement())
    }

    pub fn boot_with_placement(
        primary: PersistentRecord<ControlConfig>,
        secondary: PersistentRecord<ControlConfig>,
        placement: ReplicaPlacement<3>,
    ) -> Result<Self, LoadReport<ControlConfig>> {
        let runtime_store = SharedLayoutStore::new(primary.value(), placement);
        let (core, boot_report) =
            load_into_core::<_, Tmr, _, 3>(primary, secondary, runtime_store.clone())?;
        let protected = CriticalSectionHardened::from_hardened(DemoCriticalSection, core);

        let mut app = Self {
            runtime_store,
            protected,
            journal: Journal::new(),
            boot_report,
            outputs_enabled: false,
        };

        app.push_event(JournalEvent::ResetObserved {
            cause: ResetCause::PowerOn,
        });
        app.log_boot_report();
        Ok(app)
    }

    pub fn boot_report(&self) -> LoadReport<ControlConfig> {
        self.boot_report
    }

    pub fn outputs_enabled(&self) -> bool {
        self.outputs_enabled
    }

    pub fn runtime_snapshot(&self) -> [ControlConfig; 3] {
        self.runtime_store.snapshot()
    }

    pub fn runtime_site(&self, index: usize) -> Option<PlacementSite> {
        self.runtime_store.site_of(index)
    }

    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    pub fn journal_record(&self, index: usize) -> Option<EventRecord> {
        self.journal.encoded(index).and_then(decode_record)
    }

    pub fn inject_outlier(&self, index: usize, value: ControlConfig) {
        let _ = corrupt_slot(&self.runtime_store, index, value);
    }

    pub fn inject_pattern(&self, replicas: [ControlConfig; 3]) {
        apply_pattern(&self.runtime_store, replicas);
    }

    pub fn step(&mut self) -> CycleOutcome {
        match self.protected.read_checked() {
            ReadReport::Trusted {
                value,
                status: hdf_core::TrustedStatus::Clean,
            } => {
                self.outputs_enabled = value.output_enabled && value.mode != ControlMode::Safe;
                self.push_event(JournalEvent::IntegrityClean);
                CycleOutcome::Nominal { config: value }
            }
            ReadReport::Trusted {
                value,
                status: hdf_core::TrustedStatus::RecoverableMismatch,
            } => {
                self.outputs_enabled = value.output_enabled && value.mode != ControlMode::Safe;
                self.push_event(JournalEvent::RecoverableMismatch);
                self.push_event(JournalEvent::RepairAttempted);
                match self.protected.repair() {
                    RepairOutcome::NoRepairNeeded | RepairOutcome::Repaired => {
                        self.push_event(JournalEvent::RepairSucceeded);
                    }
                    RepairOutcome::NotPossible => {
                        self.outputs_enabled = false;
                        self.push_event(JournalEvent::RepairNotPossible);
                    }
                }
                CycleOutcome::Recovered { config: value }
            }
            ReadReport::Suspect { replicas, reason } => {
                self.outputs_enabled = false;
                self.push_event(JournalEvent::SuspectNoTrustedValue);
                CycleOutcome::SafeHold { reason, replicas }
            }
        }
    }

    fn push_event(&mut self, event: JournalEvent) {
        let _ = self.journal.append(event);
    }

    fn log_boot_report(&mut self) {
        if let LoadReport::Trusted {
            version,
            source,
            reason,
            ..
        } = self.boot_report
        {
            let selected = match source {
                SlotId::Primary => StorageSource::Primary,
                SlotId::Secondary => StorageSource::Secondary,
            };
            self.push_event(JournalEvent::StorageRecordSelected {
                source: selected,
                version,
            });

            if matches!(reason, LoadReason::MatchingCopies) {
                self.push_event(JournalEvent::IntegrityClean);
            }
        }
    }
}

fn default_placement() -> ReplicaPlacement<3> {
    ReplicaPlacement::with_sites([
        PlacementSite::with_details(RegionId(0), Some(BankId(0)), Some(SectionId(".ctrl_a"))),
        PlacementSite::with_details(RegionId(1), Some(BankId(1)), Some(SectionId(".ctrl_b"))),
        PlacementSite::with_details(RegionId(2), Some(BankId(2)), Some(SectionId(".ctrl_c"))),
    ])
}
