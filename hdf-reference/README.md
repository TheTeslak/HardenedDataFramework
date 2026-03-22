# hdf-reference

`hdf-reference` is the phase-9 reference integration for HDF.

It demonstrates one realistic MCU-style workflow on the host:

1. load a persisted config through `hdf-storage`;
2. materialize it into RAM hardening with `hdf-core`;
3. wrap shared access with `hdf-sync`;
4. describe runtime placement with `hdf-layout`;
5. inject corruption with `hdf-fault` during scenario testing;
6. record recovery events with `hdf-journal`.

The crate does not add new trust semantics. It only composes the existing crates into a reference application flow.

## Integration steps

1. Prepare two persisted records and call `ReferenceApp::boot(...)`.
2. Let `hdf-storage` pick a trusted record or fail explicitly.
3. Store the selected config in a layout-aware runtime store.
4. Access the runtime config only through the synchronized wrapper.
5. On each control-cycle step, branch on the explicit `ReadReport` outcome.
6. Log clean, recoverable, suspect, and repair events into the journal.
7. Disable outputs when a suspect state appears.

## Fault walkthrough

- single outlier -> `RecoverableMismatch`, explicit repair, outputs remain enabled;
- no majority -> suspect path, no trusted config, outputs forced off;
- storage corruption -> rollback to the older valid persisted record when possible.

## Measurement notes

See `hdf-reference/MEASUREMENTS.md` for the current host-side size summary and release-binary proxy.

For a more readable console walkthrough, run `cargo run -p hdf-reference --example visual_trace`.

## Example

```rust
use hdf_reference::{ControlConfig, ControlMode, CycleOutcome, ReferenceApp};
use hdf_storage::{PersistentRecord, next_record};

let primary = PersistentRecord::new(1, ControlConfig::new(ControlMode::Standby, 900, 7, true));
let secondary = next_record(1, ControlConfig::new(ControlMode::Active, 950, 8, true));

let mut app = ReferenceApp::<16>::boot(primary, secondary).expect("valid persisted config");

match app.step() {
    CycleOutcome::Nominal { config } => assert_eq!(config.mode, ControlMode::Active),
    outcome => panic!("unexpected startup outcome: {outcome:?}"),
}
```
