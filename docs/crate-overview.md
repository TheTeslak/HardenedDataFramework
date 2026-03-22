# Crate Overview

- `hdf-core`: logical redundancy and explicit trust reports for RAM-resident values
- `hdf-sync`: synchronized access wrappers that preserve core reports unchanged
- `hdf-layout`: placement-aware and complemented backends without changing DMR/TMR truth tables
- `hdf-fault`: deterministic corruption helpers for verification and examples
- `hdf-storage`: persistent record classification, selection, and load-into-core flow
- `hdf-journal`: encoded observability for integrity and recovery events
- `hdf-reference`: realistic integration of the crates into one MCU-style workflow

Boundary rule: if a new feature changes the meaning of `ReadReport`, `CheckReport`, or `RepairOutcome`, it does not belong in a companion crate; it belongs in a spec discussion first.
