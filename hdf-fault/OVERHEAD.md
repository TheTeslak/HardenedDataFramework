# Overhead Baseline

This file records the current host-side baseline from:

- command: `cargo run -p hdf-fault --example overhead_report --release`
- date: `2026-03-20`
- environment: `/workspace` Linux host, release build, std runtime

These numbers are only a transparent host baseline. They do not replace target-specific embedded measurement.

## Runtime

Direct core / `InlineStore`

- `read_checked()`: `1.39 ns/op`
- `check()`: `0.50 ns/op`
- `write()`: `1.19 ns/op`

`SerializedHardened` / `InlineStore`

- `read_checked()`: `1.47 ns/op`
- `check()`: `0.63 ns/op`
- `write()`: `1.68 ns/op`

`CriticalSectionHardened` / `InlineStore`

- `read_checked()`: `1.43 ns/op`
- `check()`: `0.47 ns/op`
- `write()`: `1.78 ns/op`

Direct core / `SplitStore`

- `read_checked()`: `1.47 ns/op`
- `check()`: `0.47 ns/op`
- `write()`: `1.64 ns/op`

Direct core / `ComplementedStore`

- `read_checked()`: `1.40 ns/op`
- `check()`: `0.47 ns/op`
- `write()`: `1.41 ns/op`

Repair after injected single-slot corruption

- core: `7.07 ns/op`
- serialized: `7.40 ns/op`
- critical-section: `7.85 ns/op`
- split: `9.53 ns/op`
- complemented: `7.55 ns/op`

## Footprint

- `InlineStore<u32, 3>`: `12 bytes`
- `SplitStore<u32, 3>`: `136 bytes`
- `ComplementedStore<u32, 3>`: `136 bytes`
- `Hardened<u32, Tmr, InlineStore<u32, 3>, 3>`: `12 bytes`
- `SerializedHardened<u32, Tmr, InlineStore<u32, 3>, 3>`: `24 bytes`
- `CriticalSectionHardened<NoopCriticalSection, u32, Tmr, InlineStore<u32, 3>, 3>`: `24 bytes`

## Notes

- Repair measurements corrupt one replica before every `repair()` call.
- The host timer resolution is much coarser than embedded cycle counters; use the included harness again on representative targets before making deployment claims.
