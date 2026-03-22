use hdf_storage::{LoadReport, PersistentRecord, RecordData, SlotId, load_pair, next_record};

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
        3,
        Config {
            threshold: 1000,
            flags: 0x03,
            revision: 7,
        },
    );
    let secondary = next_record(
        3,
        Config {
            threshold: 1100,
            flags: 0x07,
            revision: 8,
        },
    );

    match load_pair(primary, secondary) {
        LoadReport::Trusted {
            value,
            version,
            source: SlotId::Secondary,
            ..
        } => println!("loaded newer persisted config v{version}: {value:?}"),
        report => println!("unexpected load report: {report:?}"),
    }
}
