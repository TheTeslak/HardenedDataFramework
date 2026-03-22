use hdf_core::{CheckReport, Dmr, Hardened, InlineStore, ReadReport, SuspectReason, TrustedStatus};

fn main() {
    let protected = Hardened::<u16, Dmr, _, 2>::new(1200, InlineStore::new(0));

    match protected.read_checked() {
        ReadReport::Trusted {
            value,
            status: TrustedStatus::Clean,
        } => println!("apply trusted threshold: {value}"),
        ReadReport::Trusted { value, status } => {
            println!("apply trusted threshold {value} with status {status:?}")
        }
        ReadReport::Suspect {
            replicas,
            reason: SuspectReason::DmrConflict,
        } => {
            println!("threshold conflict, require reload: {replicas:?}")
        }
        ReadReport::Suspect { replicas, reason } => {
            println!("suspect replicas {replicas:?}: {reason:?}")
        }
    }

    match protected.check() {
        CheckReport::Consistent => println!("integrity check clean"),
        CheckReport::RecoverablyInconsistent => println!("repair is justified"),
        CheckReport::Suspect => println!("no trusted value can be justified"),
    }
}
