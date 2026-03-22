# Contributing

## Workflow

1. Read `spec.md` before changing semantics.
2. Keep changes scoped to the crate or document that owns the behavior.
3. Add or update tests and runnable examples with every behavior change.
4. Keep guarantees and non-goals explicit in docs.

## Local Verification

- `cargo fmt --all`
- `cargo test --all-targets`
- `cargo doc --workspace --no-deps`

For focused work, also run the crate-local command you changed, for example `cargo test -p hdf-storage --all-targets`.

## Design Rules

- do not hide trust decisions behind convenience APIs;
- do not add concurrency, storage, or layout semantics to `hdf-core`;
- prefer new companion crates over widening core semantics;
- if a behavior cannot be guaranteed honestly, document it as a non-goal.

## Pull Request Expectations

- explain why the change exists;
- list touched crates and examples;
- mention verification commands that passed;
- update `CHANGELOG.md` when user-visible behavior changes.
