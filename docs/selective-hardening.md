# Selective Hardening

Use hardening for values where corruption changes control flow, safety posture, or recovery decisions.

## Good candidates

- mode flags and state-machine states that gate behavior;
- thresholds, limits, and calibration values read often but updated rarely;
- small counters where one trusted majority is acceptable;
- compact `Copy + Eq` configs that fit in explicit DMR/TMR reasoning.

## Which helper to choose

- `detect_only(...)` for values where a mismatch must stop or escalate because there is no justified winner;
- `recoverable(...)` for values where a TMR majority can justify a trusted result and explicit `repair()` is acceptable.

## Keep handling explicit

- always branch on `ReadReport`;
- treat `TrustedStatus::RecoverableMismatch` as a warning path, not the same as clean;
- keep `repair()` explicit so callers decide when rewriting replicas is acceptable.

## Poor candidates

- large buffers or bulk telemetry;
- heap-backed or pointer-rich data;
- semantically ambiguous numeric types such as raw floats;
- fast-changing values where replication cost is higher than the value's importance;
- data already protected better at another layer, such as protocol framing or storage journaling.

## Example mapping

- state machine: `hdf-core/examples/tmr_mode.rs`
- mode flag: `hdf-core/examples/dmr_safety_flag.rs`
- threshold handling: `hdf-core/examples/read_report_branching.rs`
- counter: `hdf-core/examples/recoverable_counter.rs`
