use hdf_journal::{EventRecord, JournalEvent, ResetCause, render_record};

fn main() {
    let encoded = EventRecord {
        sequence: 17,
        event: JournalEvent::ResetObserved {
            cause: ResetCause::Watchdog,
        },
    }
    .encode();

    let decoded = render_record(encoded).expect("valid encoded event");
    println!("decoded event: {decoded}");
}
