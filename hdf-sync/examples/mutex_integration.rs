use std::sync::Mutex;

use hdf_core::{InlineStore, ReadReport, Tmr, TrustedStatus};
use hdf_sync::SerializedHardened;

fn main() {
    let protected = Mutex::new(SerializedHardened::<u8, Tmr, _, 3>::new(
        7,
        InlineStore::new(0),
    ));

    {
        let guard = protected.lock().expect("mutex poisoned");
        guard.write(9);
    }

    let guard = protected.lock().expect("mutex poisoned");
    match guard.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("mutex-guarded trusted value: {value}"),
        report => println!("unexpected report: {report:?}"),
    }
}
