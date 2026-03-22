# Releasing

## Versioning Baseline

- workspace crates currently use `0.1.0`
- semver promises apply only after a crate is declared stable
- until then, compatibility is managed crate-by-crate with explicit changelog notes

## Release Checklist

1. Run `cargo fmt --all`.
2. Run `cargo test --all-targets`.
3. Run `cargo doc --workspace --no-deps`.
4. Refresh measurement files such as `hdf-fault/OVERHEAD.md` and `hdf-reference/MEASUREMENTS.md` when behavior or footprint changes.
5. Update `CHANGELOG.md`, `README.md`, and any affected crate READMEs.
6. Review `TODO.md` and mark the roadmap honestly.

## Stabilization Guidance

- release crates individually if some later crates remain experimental;
- keep `hdf-core` semantics stable before widening convenience layers;
- do not mark a crate stable if its examples, tests, and non-goals are not documented clearly.
