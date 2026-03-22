#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotId {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordStatus {
    Valid { version: u32 },
    Corrupted { version: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadReason {
    MatchingCopies,
    PrimaryNewer,
    SecondaryNewer,
    OtherCopyCorrupted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadReport<T> {
    Trusted {
        value: T,
        version: u32,
        source: SlotId,
        reason: LoadReason,
    },
    Conflict {
        primary: RecordStatus,
        secondary: RecordStatus,
    },
    NoUsableRecord {
        primary: RecordStatus,
        secondary: RecordStatus,
    },
}
