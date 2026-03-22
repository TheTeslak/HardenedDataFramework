use hdf_core::{Hardened, InlineStore, ReadReport, Tmr, TrustedStatus};

fn main() {
    let protected = Hardened::<u8, Tmr, _, 3>::new(42, InlineStore::new(0));

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("trusted clean value: {value}"),
        report => println!("unexpected report: {report:?}"),
    }
}
