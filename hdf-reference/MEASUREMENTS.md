# Measurements

Source commands:

- `cargo build -p hdf-reference --examples --release`
- `target/release/examples/footprint_report`
- `stat -c%s target/release/examples/reference_demo`

Environment:

- host: Linux `/workspace`
- build: `release`
- note: these are host-side proxies, not MCU flash/RAM numbers

## In-memory size proxy

- `ControlConfig`: `6 bytes`
- `SharedLayoutStore<ControlConfig, 3>`: `8 bytes`
- `Journal<16>`: `208 bytes`
- `ReferenceApp<16>`: `256 bytes`

## Code-size proxy

- `target/release/examples/reference_demo`: `453040 bytes`

## Notes

- The journal footprint dominates the demo state because it reserves a fixed 16-entry buffer.
- The release example size is a host binary, not a direct flash number.
- The point of this report is transparency and repeatability; use the same commands on a representative target build before making deployment claims.
