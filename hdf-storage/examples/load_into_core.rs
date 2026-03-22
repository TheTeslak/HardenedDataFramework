use hdf_core::{InlineStore, ReadReport, Tmr, TrustedStatus};
use hdf_storage::{PersistentRecord, RecordData, load_into_core, next_record};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Config {
    threshold: u16,
    flags: u8,
    revision: u8,
}

impl RecordData for Config {
    fn checksum(&self) -> u32 {
        self.threshold as u32 ^ ((self.flags as u32) << 8) ^ ((self.revision as u32) << 16)
    }
}

fn main() {
    let primary = PersistentRecord::new(
        1,
        Config {
            threshold: 1000,
            flags: 0x03,
            revision: 7,
        },
    );
    let secondary = next_record(
        1,
        Config {
            threshold: 1100,
            flags: 0x07,
            revision: 8,
        },
    );

    let (protected, report) = load_into_core::<_, Tmr, _, 3>(
        primary,
        secondary,
        InlineStore::new(Config {
            threshold: 0,
            flags: 0,
            revision: 0,
        }),
    )
    .expect("valid persisted config should load into RAM hardening");

    println!("storage report: {report:?}");
    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("in-memory protected config: {value:?}"),
        report => println!("unexpected RAM report: {report:?}"),
    }
}
