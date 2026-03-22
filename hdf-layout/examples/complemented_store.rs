use hdf_core::{Hardened, ReadReport, Tmr, TrustedStatus};
use hdf_layout::{ComplementedStore, RegionId, ReplicaPlacement};

fn main() {
    let store = ComplementedStore::new(
        0x3Cu8,
        ReplicaPlacement::new([RegionId(1), RegionId(2), RegionId(3)]),
    );

    println!("logical replicas: {:?}", store.read_replicas());
    println!("encoded replicas: {:?}", store.encoded_replicas());

    let protected = Hardened::<u8, Tmr, _, 3>::new(0x3C, store);
    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("trusted clean value: {value:#04x}"),
        report => println!("unexpected report: {report:?}"),
    }
}
