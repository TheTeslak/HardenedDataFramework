# Validation Matrix

## Continuous checks

- formatting: `cargo fmt --all --check`
- tests and examples: `cargo test --all-targets`
- docs: `cargo doc --workspace --no-deps`

## `no_std` crates

The following crates are intended to remain `no_std`:

- `hdf-core`
- `hdf-sync`
- `hdf-layout`
- `hdf-fault`
- `hdf-storage`
- `hdf-journal`

Recommended validation commands:

- `cargo check -p hdf-core --lib`
- `cargo check -p hdf-sync --lib`
- `cargo check -p hdf-layout --lib`
- `cargo check -p hdf-fault --lib`
- `cargo check -p hdf-storage --lib`
- `cargo check -p hdf-journal --lib`

## Host-side integration validation

- `cargo test -p hdf-reference --all-targets`
- `cargo run -p hdf-reference --example reference_demo`
- `cargo run -p hdf-reference --example footprint_report --release`

## Measurement refresh points

- `hdf-fault/OVERHEAD.md`
- `hdf-reference/MEASUREMENTS.md`

Host numbers are transparency tools, not substitutes for target-specific embedded validation.
