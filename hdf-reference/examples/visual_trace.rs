use hdf_core::SuspectReason;
use hdf_journal::EventRecord;
use hdf_layout::PlacementSite;
use hdf_reference::{ControlConfig, ControlMode, CycleOutcome, ReferenceApp};
use hdf_storage::{PersistentRecord, next_record};

fn cfg(mode: ControlMode, threshold: u16, revision: u8, output_enabled: bool) -> ControlConfig {
    ControlConfig::new(mode, threshold, revision, output_enabled)
}

fn main() {
    let primary = PersistentRecord::new(1, cfg(ControlMode::Standby, 900, 7, true));
    let secondary = next_record(1, cfg(ControlMode::Active, 950, 8, true));

    let mut app = ReferenceApp::<16>::boot(primary, secondary).expect("valid persisted config");

    println!("HDF visual walkthrough");
    print_rule();
    print_boot(primary, secondary, app.boot_report());
    print_rule();
    print_placement(&app);
    print_rule();
    print_snapshot("after boot", app.runtime_snapshot());

    let cycle_1 = app.step();
    print_cycle("cycle 1", cycle_1, app.outputs_enabled());
    print_snapshot("after cycle 1", app.runtime_snapshot());

    app.inject_outlier(2, cfg(ControlMode::Safe, 950, 9, false));
    print_snapshot("after single-slot fault", app.runtime_snapshot());
    let cycle_2 = app.step();
    print_cycle("cycle 2", cycle_2, app.outputs_enabled());
    print_snapshot("after repair", app.runtime_snapshot());

    app.inject_pattern([
        cfg(ControlMode::Standby, 800, 5, true),
        cfg(ControlMode::Active, 850, 6, true),
        cfg(ControlMode::Safe, 900, 7, false),
    ]);
    print_snapshot("after no-majority fault", app.runtime_snapshot());
    let cycle_3 = app.step();
    print_cycle("cycle 3", cycle_3, app.outputs_enabled());

    print_rule();
    print_journal(&app);
}

fn print_rule() {
    println!("============================================================");
}

fn print_boot(
    primary: PersistentRecord<ControlConfig>,
    secondary: PersistentRecord<ControlConfig>,
    report: hdf_storage::LoadReport<ControlConfig>,
) {
    println!("Boot selection");
    println!(
        "primary   v{:>2}  {}",
        primary.version(),
        format_config(primary.value())
    );
    println!(
        "secondary v{:>2}  {}",
        secondary.version(),
        format_config(secondary.value())
    );
    println!("decision      {:?}", report);
}

fn print_placement(app: &ReferenceApp<16>) {
    println!("Replica placement");
    println!("slot | region | bank | section");
    println!("-----+--------+------+-----------");
    for index in 0..3 {
        let site = app.runtime_site(index).expect("site exists");
        print_site_row(index, site);
    }
}

fn print_site_row(index: usize, site: PlacementSite) {
    println!(
        "{:>4} | {:>6} | {:>4} | {}",
        index,
        site.region().0,
        site.bank()
            .map(|bank| bank.0.to_string())
            .unwrap_or_else(|| "-".into()),
        site.section().map(|section| section.0).unwrap_or("-")
    );
}

fn print_snapshot(label: &str, replicas: [ControlConfig; 3]) {
    println!("{label}");
    println!("slot | mode     | threshold | rev | out");
    println!("-----+----------+-----------+-----+-----");
    for (index, replica) in replicas.into_iter().enumerate() {
        println!(
            "{:>4} | {:<8} | {:>9} | {:>3} | {}",
            index,
            mode_name(replica.mode),
            replica.threshold,
            replica.revision,
            if replica.output_enabled { "on" } else { "off" }
        );
    }
    println!();
}

fn print_cycle(label: &str, outcome: CycleOutcome, outputs_enabled: bool) {
    let summary = match outcome {
        CycleOutcome::Nominal { config } => format!("Nominal      -> {}", format_config(config)),
        CycleOutcome::Recovered { config } => {
            format!("Recovered    -> {}", format_config(config))
        }
        CycleOutcome::SafeHold { reason, replicas } => {
            format!("SafeHold({}) -> {:?}", reason_name(reason), replicas)
        }
    };

    println!("{label:>8}: {summary}");
    println!(
        "outputs enabled: {}",
        if outputs_enabled { "yes" } else { "no" }
    );
    println!();
}

fn print_journal(app: &ReferenceApp<16>) {
    println!("Journal timeline");
    for index in 0..app.journal_len() {
        let record = app.journal_record(index).expect("journal record exists");
        print_record(record);
    }
}

fn print_record(record: EventRecord) {
    println!("#{:02} {}", record.sequence, record.event);
}

fn format_config(config: ControlConfig) -> String {
    format!(
        "mode={}, threshold={}, rev={}, out={}",
        mode_name(config.mode),
        config.threshold,
        config.revision,
        if config.output_enabled { "on" } else { "off" }
    )
}

fn mode_name(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Standby => "Standby",
        ControlMode::Active => "Active",
        ControlMode::Safe => "Safe",
    }
}

fn reason_name(reason: SuspectReason) -> &'static str {
    match reason {
        SuspectReason::DmrConflict => "DMR conflict",
        SuspectReason::NoMajority => "No majority",
    }
}
