# Structured Data

Small composite values are supported when they stay `Copy + Eq` and their equality meaning matches the application's trust decision.

## Supported patterns

- compact config structs made of integers, bools, and enums;
- state bundles where whole-value equality is the right voting rule;
- small mode/config records that fit explicit DMR/TMR reasoning.

## Ground rules

- voting is whole-value equality only;
- there is no field-wise merge or partial reconstruction;
- `ReadReport` handling stays explicit even for composite values;
- `repair()` only writes back a majority-justified full value.

## Out of scope

- heap-backed types such as `Vec`, `String`, or pointer-rich graphs;
- large blobs where whole-value equality is too coarse or expensive;
- raw floats and other semantically ambiguous values;
- types whose equality does not match the trust decision you actually need.

## Examples

- composite config: `hdf-core/examples/composite_config.rs`
- mode/state example: `hdf-core/examples/tmr_mode.rs`
- selective-hardening guide: `docs/selective-hardening.md`
