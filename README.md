# Hardened Data Framework (do not use)

HDF is a Rust workspace for selectively protecting critical embedded data through redundant storage, explicit integrity reporting, and scheme-aware recovery.

The workspace is split by concern:

- `hdf-core` - RAM-resident DMR/TMR hardening with explicit `ReadReport`/`CheckReport`/`RepairOutcome`
- `hdf-sync` - synchronized wrapper paths for shared access
- `hdf-layout` - placement-aware and complemented backends
- `hdf-fault` - deterministic corruption injection and verification helpers
- `hdf-storage` - persistent record selection and load-into-core flow
- `hdf-journal` - compact observability for integrity and recovery events
- `hdf-reference` - end-to-end reference integration across the crates

## Getting Started

```rust
use hdf_core::{ReadReport, TrustedStatus, recoverable};

let protected = recoverable(42u16);

match protected.read_checked() {
    ReadReport::Trusted {
        value,
        status: TrustedStatus::Clean,
    } => assert_eq!(value, 42),
    report => panic!("unexpected report: {report:?}"),
}
```

## Key Principles

- trust is always explicit through reports, never hidden behind plain-value reads;
- DMR is detect-only, TMR can justify recovery and explicit repair;
- storage, synchronization, layout, and journaling stay separate from the core semantics;
- documentation and examples are expected to state both guarantees and non-goals honestly.

## Useful Commands

- `cargo test --all-targets`
- `cargo run -p hdf-reference --example reference_demo`
- `cargo run -p hdf-reference --example visual_trace`
- `cargo run -p hdf-fault --example fault_matrix`
- `cargo run -p hdf-fault --example overhead_report --release`

## Documentation

- crate boundaries: `docs/crate-overview.md`
- selective hardening guide: `docs/selective-hardening.md`
- structured data guide: `docs/structured-data.md`
- validation matrix: `docs/validation-matrix.md`
- roadmap: `TODO.md`
- release process: `RELEASING.md`
