use hdf_reference::{ControlConfig, ControlMode, CycleOutcome, ReferenceApp};
use hdf_storage::{PersistentRecord, next_record};

fn cfg(mode: ControlMode, threshold: u16, revision: u8, output_enabled: bool) -> ControlConfig {
    ControlConfig::new(mode, threshold, revision, output_enabled)
}

fn main() {
    let primary = PersistentRecord::new(1, cfg(ControlMode::Standby, 900, 7, true));
    let secondary = next_record(1, cfg(ControlMode::Active, 950, 8, true));

    let mut app = ReferenceApp::<16>::boot(primary, secondary).expect("valid persisted config");
    println!("boot report: {:?}", app.boot_report());
    println!(
        "runtime placement slot 0 -> {:?}",
        app.runtime_snapshot()[0]
    );

    println!("cycle 1: {:?}", app.step());

    app.inject_outlier(2, cfg(ControlMode::Safe, 950, 9, false));
    println!("cycle 2 after single fault: {:?}", app.step());

    app.inject_pattern([
        cfg(ControlMode::Standby, 800, 5, true),
        cfg(ControlMode::Active, 850, 6, true),
        cfg(ControlMode::Safe, 900, 7, false),
    ]);
    println!("cycle 3 after no-majority fault: {:?}", app.step());
    println!("outputs enabled: {}", app.outputs_enabled());

    for index in 0..app.journal_len() {
        println!("journal[{index}]: {}", app.journal_record(index).unwrap());
    }

    match app.step() {
        CycleOutcome::Nominal { .. }
        | CycleOutcome::Recovered { .. }
        | CycleOutcome::SafeHold { .. } => {}
    }
}
