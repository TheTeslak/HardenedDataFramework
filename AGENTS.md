# AGENTS

## Project

HDF is a Rust workspace for hardened embedded data handling. The current implemented crates are `hdf-core` for single-owner redundancy logic and `hdf-sync` for shared-access and synchronization helpers on top of core semantics. The next phases are expected to add `hdf-layout` for physical replica placement and later storage/journaling-focused crates.

## Current phase

Current workspace focus: `hdf-core` and `hdf-sync` are active. After `hdf-sync`, the next target phase is `hdf-layout`.

## Wave rules

- Every task must declare explicit `paths` and `depends_on`.
- Tasks in the same wave must have no overlapping `paths`.
- A shared file has exactly one owner in a wave.
- If one task needs another task's output, put it in a later wave.
- Conservative conflict rule: if there is any doubt about overlap or dependency order, execute sequentially.

## Roles

### `explore`

- Goal: inspect architecture, dependency edges, and candidate task boundaries before coding.
- Typical paths: `spec.md`, `docs/`, `Cargo.toml`, `hdf-core/**`, `hdf-sync/**`.
- Do not touch: crate source files during implementation waves unless the task explicitly assigns ownership.

### `rust-core`

- Goal: implement or refactor Rust behavior inside the active crate scope.
- Typical paths: `hdf-core/src/**`, `hdf-core/tests/**`, `hdf-core/examples/**`, `hdf-sync/src/**`, `hdf-sync/tests/**`.
- Do not touch: shared top-level docs, workspace manifests, or another crate's files unless those paths are explicitly assigned.

### `docs`

- Goal: keep crate and workspace documentation aligned with implemented behavior.
- Typical paths: `README.md`, `docs/**`, `hdf-core/README.md`, `hdf-sync/README.md`, `AGENTS.md`.
- Do not touch: Rust source, tests, generated artifacts, or manifests unless the documentation task explicitly owns them.

### `verify`

- Goal: validate behavior, API shape, and phase outputs without broad code edits.
- Typical paths: `hdf-core/tests/**`, `hdf-sync/tests/**`, `hdf-core/examples/**`, verification notes in `docs/` if assigned.
- Do not touch: production source paths owned by implementation roles, except for minimal test-only changes assigned to the verification wave.

### `layout`

- Goal: own the upcoming physical-layout phase that separates logical redundancy from memory placement policy.
- Typical paths: future `hdf-layout/**`, layout-related workspace docs, and any explicitly assigned integration points.
- Do not touch: `hdf-core/**` or `hdf-sync/**` internals outside agreed integration seams.

## Path ownership by area

- `hdf-core/**`: primary owner is `rust-core` during core waves.
- `hdf-sync/**`: primary owner is `rust-core` during sync waves.
- `docs/**`, `AGENTS.md`, crate READMEs: primary owner is `docs`.
- `Cargo.toml`, `Cargo.lock`, shared top-level docs: assign to one explicit owner per wave because they are shared files.
