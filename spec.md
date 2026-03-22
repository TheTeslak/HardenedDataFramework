# Hardened Data Framework Engineering Specification

## 1. Status and Remaining Minor Note

This specification is implementation-ready for the accepted architecture.

The major architectural questions are fixed:

- `hdf-core` is single-owner, `no_std`, allocation-free, and intentionally not concurrency-safe.
- replica access in core uses volatile reads and writes through internal mutable storage.
- logical redundancy is separated from physical replica placement through a store backend boundary.
- DMR is detect-only; TMR can establish a trusted majority and can be explicitly repaired.
- persistent storage and journaling are separate crates, not extensions forced into the RAM core API.

Only minor follow-up remains during implementation: keep early convenience APIs in `hdf-sync`, `hdf-layout`, and later crates narrow so they do not imply guarantees beyond the accepted model.

## 2. Project Definition and Non-Goals

Hardened Data Framework (HDF) is a Rust framework for selective protection of critical embedded data through redundant representation, explicit integrity evaluation, and scheme-aware recovery behavior.

The framework exists for a small set of values whose corruption materially affects system behavior, such as:

- control mode and state-machine state;
- safety and arming flags;
- thresholds and operational parameters;
- command counters and sequencing state;
- last-known-valid values used for fallback or restart.

The project is a focused hardening layer, not a whole-system resilience platform.

In scope:

- RAM-resident critical data;
- `no_std` operation in constrained environments;
- fixed-size values with clear equality semantics;
- DMR and TMR schemes;
- explicit trust/integrity reporting;
- explicit repair where the scheme justifies it;
- fault injection and verification support;
- later companion crates for synchronization, layout, storage, journaling, and reference integration.

Out of scope for the core:

- blanket hardening of all program state;
- task scheduling, restart orchestration, or supervision;
- lock-free multi-context coordination for protected values;
- distributed or network fault tolerance;
- protection against arbitrary adversarial tampering;
- guarantees against multi-point correlated corruption;
- heap-heavy generic container protection;
- forcing Flash/EEPROM semantics into the RAM core API.

## 3. Design Principles

1. **Selective protection, not global protection.** Developers explicitly choose which values to harden.
2. **Deterministic behavior under corruption.** Every checked read yields an explicit verdict.
3. **Detection and recovery are separate concerns.** DMR detects; TMR detects and may recover.
4. **`no_std` first.** The core must run without allocation and without assuming an OS.
5. **Small, inspectable abstractions.** The API should remain narrow, explicit, and understandable.
6. **Single-owner core semantics.** Concurrency is not mixed into core redundancy semantics.
7. **Logical redundancy is separate from physical layout.** Scheme math and placement policy must not be conflated.
8. **Fault tolerance must be testable.** Corruption must be injectable and behavior must be measurable.
9. **Incremental growth by crate boundary.** New domains reuse trust/recovery concepts but keep distinct APIs.

## 4. Fault Model and Guarantees

### 4.1 Fault model assumed by `hdf-core`

`hdf-core` assumes that corruption of one replica is a meaningful engineering fault model for selected RAM-resident values.

The core explicitly targets:

- corruption of one replica or slot value in RAM;
- disagreement among replicas at read/check time;
- detection of inconsistent state;
- trusted-value reconstruction only when the active scheme provides one;
- explicit repair only when the current state is mathematically recoverable.

### 4.2 What core guarantees

Given exclusive access and a correct store backend implementation, `hdf-core` guarantees:

- every `read_checked()` evaluates all required replicas using volatile access;
- `read_checked()` never silently substitutes a guessed value for DMR conflict;
- `read_checked()` does not mutate storage;
- `check()` reports consistency/recoverability state without mutating storage;
- `repair()` only rewrites replicas when the current scheme admits a trusted value;
- `write()` deterministically writes the requested value to all scheme replicas;
- the API surface makes clean, recoverable mismatch, and suspect/unrecoverable states distinguishable.

### 4.3 What core does not guarantee

`hdf-core` does not guarantee:

- atomic multi-replica writes;
- correctness under concurrent read/write access from ISR, thread, or task contexts;
- protection from torn updates observed mid-write;
- protection from correlated spatial faults when replicas are co-located;
- protection against compiler-visible ordinary-memory optimization unless access goes through the core store contract;
- durable persistence, rollback, wear handling, or erase/page semantics.

### 4.4 Volatile stance

Replica slots in `hdf-core` are accessed through `UnsafeCell` plus `core::ptr::read_volatile` and `core::ptr::write_volatile`.

This is fixed architecture, not an optimization detail.

Reason:

- the framework models externally induced corruption and test-time fault injection;
- ordinary non-volatile loads/stores allow optimization that can collapse or elide accesses and checks.

Volatile use here preserves access observability only. It does not provide:

- synchronization;
- atomicity;
- race freedom;
- cross-thread ordering.

Those concerns belong to `hdf-sync` or platform integration.

### 4.5 Physical layout realism

Inline contiguous replica placement is acceptable as:

- a logical baseline;
- a test/demo backend;
- a software corruption model backend.

It must not be described as physically strong protection against spatially correlated RAM faults. Stronger placement is a later layout concern handled by separate backends and policies.

## 5. Architecture Decisions Already Fixed

The following decisions are fixed and are not to be revisited during implementation:

1. `hdf-core` excludes shared-access semantics and requires exclusive access.
2. `read_checked(&self)` is non-mutating.
3. `repair(&mut self)` is explicit and separate from read.
4. DMR never auto-selects a winner on mismatch.
5. TMR majority voting in the core requires only `Eq`; no custom voting trait is needed in phase 1.
6. Phase 1 type support is `T: Copy + Eq`.
7. `f32`/`f64` are not phase-1 core data types unless wrapped as explicit bit-pattern types outside the base API.
8. Logical redundancy is implemented over a store backend boundary rather than assuming adjacent struct fields.
9. Persistent storage and journaling live in separate crates in the same workspace.
10. `hdf-core` is intentionally `!Sync`; shared access belongs to `hdf-sync`.

What remains intentionally minor rather than fixed in detail:

- exact naming of helper constructors and convenience methods;
- exact internal module filenames;
- whether `Send` is allowed for specific implementations;
- whether an explicit `read_and_repair(&mut self)` convenience API appears later;
- exact shape of optional metadata/version guards in `hdf-sync`.

## 6. Public API Specification

This section defines the intended public semantic surface. Exact method names may vary slightly during implementation, but the behavior and type framing are fixed.

### 6.1 Core public types

Conceptual top-level core types:

```rust
pub struct Hardened<T, Scheme, Store> { /* opaque */ }

pub struct Dmr;
pub struct Tmr;

pub trait Scheme {
    const REPLICAS: usize;

    fn evaluate<T: Copy + Eq>(replicas: /* full replica set */) -> /* scheme-specific evaluation result */;
    fn check<T: Copy + Eq>(replicas: &[T; Self::REPLICAS]) -> CheckReport;
    fn repair_value<T: Copy + Eq>(replicas: &[T; Self::REPLICAS]) -> Option<T>;
}

pub trait Store<T, const N: usize> {
    fn read_slot(&self, index: usize) -> T;
    fn write_slot(&self, index: usize, value: T);
}
```

`Scheme` and `Store` are distinct responsibilities and must remain distinct in public design:

- `Scheme` owns only logical redundancy semantics: replica count, trust evaluation, recoverability rules, and repair eligibility.
- `Store` owns only physical realization: where replicas live and how each logical slot is read or written with volatile semantics.
- `Hardened<T, Scheme, Store>` is the composition point. It obtains replicas from `Store`, applies `Scheme`, and exposes the public reports.
- `Scheme` must not encode placement assumptions.
- `Store` must not invent trust policy, majority rules, or DMR/TMR-specific decisions.

The underspecified return type in `evaluate()` is intentional here: the spec fixes responsibilities and observable outputs, not the exact internal helper type used to connect scheme evaluation to public reports.

The replica-count relationship is fixed:

- every scheme defines one compile-time replica count;
- every store instance used with that scheme provides exactly that many logical slots;
- `Hardened<T, Scheme, Store>` is only well-formed when `Store` and `Scheme` agree on the same `N`;
- phase-1 public APIs should make replica-count mismatch impossible by construction where practical, or otherwise reject it deterministically at construction time.

Phase-1 required reporting types:

```rust
pub enum ReadReport<T, const N: usize> {
    Trusted {
        value: T,
        status: TrustedStatus,
    },
    Suspect {
        replicas: [T; N],
        reason: SuspectReason,
    },
}

pub enum TrustedStatus {
    Clean,
    RecoverableMismatch,
}

pub enum SuspectReason {
    DmrConflict,
    NoMajority,
}

pub enum CheckReport {
    Consistent,
    RecoverablyInconsistent,
    Suspect,
}

pub enum RepairOutcome {
    NoRepairNeeded,
    Repaired,
    NotPossible,
}
```

The accepted framing is that `ReadReport`, `TrustedStatus`, `SuspectReason`, `CheckReport`, and `RepairOutcome` remain explicit public vocabulary.

### 6.2 Required core methods

Conceptual API:

```rust
impl<T, Scheme, Store> Hardened<T, Scheme, Store>
where
    T: Copy + Eq,
{
    pub fn new(initial: T, store: Store) -> Self;
    pub fn read_checked(&self) -> ReadReport<T, { Scheme::REPLICAS }>;
    pub fn check(&self) -> CheckReport;
    pub fn write(&mut self, value: T);
    pub fn repair(&mut self) -> RepairOutcome;
}
```

`RepairOutcome` is a required public enum in phase 1. Exact variant names may differ slightly during implementation, but the public meaning is fixed:

- `NoRepairNeeded`: replicas were already consistent enough that no rewrite occurred;
- `Repaired`: a scheme-justified trusted value was written back to all replicas;
- `NotPossible`: current replicas do not justify a trusted repair value.

### 6.3 API requirements by operation

#### `new(initial, store)`

- initializes all scheme replicas to the same value;
- does not perform lazy initialization;
- produces a fully consistent starting state.

#### `read_checked(&self)`

- performs volatile reads of all relevant replicas;
- evaluates trust according to the active scheme;
- never mutates storage;
- returns `Trusted` only when a trustworthy value can be established;
- returns `Suspect` when no trustworthy value exists;
- exposes all replica values in the suspect case.

#### `check(&self)`

- performs volatile reads;
- reports consistency or recoverability state only;
- does not return a trusted application value;
- does not mutate storage.

#### `write(&mut self, value)`

- requires exclusive access;
- writes all replicas deterministically;
- does not attempt to hide intermediate states from unsynchronized concurrent readers.

#### `repair(&mut self)`

- requires exclusive access;
- may rewrite all replicas to a trusted value only if current state is recoverable;
- is a no-op or explicit failure when repair is not justified by the scheme;
- returns `RepairOutcome::NoRepairNeeded`, `RepairOutcome::Repaired`, or `RepairOutcome::NotPossible` according to the normative truth rules in Section 7.

### 6.4 Public helpers that are acceptable but not mandatory in phase 1

If implementation value is clear, phase 1 may expose narrow helpers such as:

- `is_trusted()` / `is_suspect()` on `ReadReport`;
- `trusted_value()` accessor returning `Option<T>`;
- `replicas()` snapshot accessor for suspect reads;
- `needs_repair()` or equivalent on `CheckReport`.

These helpers must not hide the core verdict structure.

## 7. Operation Semantics and Truth Tables

This section is normative.

### 7.1 DMR read semantics

Let replicas be `a` and `b`.

- if `a == b`: return `ReadReport::Trusted { value: a, status: TrustedStatus::Clean }`
- if `a != b`: return `ReadReport::Suspect { replicas: [a, b], reason: SuspectReason::DmrConflict }`

DMR does not produce `TrustedStatus::RecoverableMismatch` in the baseline core, because a mismatch does not establish a trusted winner.

### 7.2 DMR check semantics

- if `a == b`: return `CheckReport::Consistent`
- if `a != b`: return `CheckReport::Suspect`

### 7.3 DMR repair semantics

- if `a == b`: return `RepairOutcome::NoRepairNeeded`
- if `a != b`: return `RepairOutcome::NotPossible`

There is no automatic or speculative repair in DMR.

### 7.4 TMR read semantics

Let replicas be `a`, `b`, and `c`.

- if `a == b && b == c`: return `Trusted(value = a, status = Clean)`
- if exactly one replica differs and two replicas agree:
  - return `Trusted(value = agreed_value, status = RecoverableMismatch)`
- if no two replicas agree:
  - return `Suspect(replicas = [a, b, c], reason = NoMajority)`

Textual truth table:

- `A A A` -> trusted clean
- `A A B` -> trusted recoverable mismatch, trusted value `A`
- `A B A` -> trusted recoverable mismatch, trusted value `A`
- `B A A` -> trusted recoverable mismatch, trusted value `A`
- `A B C` where all differ -> suspect, no majority

### 7.5 TMR check semantics

- all three equal -> `CheckReport::Consistent`
- exactly two equal -> `CheckReport::RecoverablyInconsistent`
- all three different -> `CheckReport::Suspect`

### 7.6 TMR repair semantics

- all three equal -> `RepairOutcome::NoRepairNeeded`
- exactly two equal -> rewrite all replicas to the majority value; return `RepairOutcome::Repaired`
- all three different -> `RepairOutcome::NotPossible`

### 7.7 Read versus repair rules

These rules are fixed:

- `read_checked()` never rewrites storage;
- `check()` never rewrites storage;
- `repair()` is the only baseline operation that rewrites based on an already stored recoverable majority;
- `write()` overwrites all replicas with a caller-supplied value;
- callers that want "read, then repair if needed" must compose these operations explicitly or use a later convenience method with identical semantics.

### 7.8 Trusted versus suspect meaning

`Trusted` means the framework can justify a single value under the active scheme.

`Suspect` means the framework cannot justify a single value and therefore must expose raw replicas instead of pretending certainty.

### 7.9 Last-known-good policy

The core does not retain application-level last-known-good state. If a system wants to fall back to previously trusted data, that policy belongs to a higher layer or to application code.

## 8. Store Backend Contract

The store backend boundary separates redundancy logic from physical placement.

At the public-spec level, `Scheme` and `Store` are the two fixed halves of the contract:

- `Scheme` defines how many replicas exist and what those replicas mean logically.
- `Store` defines how that fixed set of logical replicas is materialized in memory.
- `hdf-core` owns the orchestration, not either side individually.

### 8.1 Contract goals

A store backend must provide:

- fixed replica slots suitable for the chosen scheme;
- volatile readable and writable per-replica access;
- deterministic mapping from logical replica index to physical slot;
- exclusive mutable access behavior compatible with `hdf-core` operations.

Minimum responsibilities are therefore:

- `Scheme`: fixed `REPLICAS`, replica-set evaluation, recoverability decision, repair eligibility.
- `Store`: slot provisioning, slot addressing, volatile slot access, and no hidden state that changes logical results.

### 8.2 Required backend properties

For a store used with phase-1 core:

- slot count must match the scheme replica count;
- each slot must hold a `T` value;
- read/write access used by `hdf-core` must be implemented with volatile semantics;
- backend code must not cache replica values across operations;
- backend must not rely on aliasing assumptions invalidated by `UnsafeCell` use.

Minimum safety invariants required of every phase-1 store implementation:

- **Fixed cardinality:** the set of logical replica slots for one `Hardened` instance does not grow, shrink, or remap nondeterministically after construction.
- **Stable indexing:** repeated access to index `i` refers to the same logical replica slot for the life of the value.
- **Per-slot independence:** writing slot `i` must not implicitly overwrite or mutate slot `j` except through an explicitly documented encoded-policy backend introduced later.
- **Read freshness:** each `read_slot()` reflects current memory contents of that slot as observed through volatile access, not a cached prior value.
- **Write completeness:** after `write_slot(i, v)` returns, a subsequent volatile `read_slot(i)` from the same execution context observes `v` unless external corruption has occurred.
- **No hidden repair:** stores may not normalize, vote, mirror, or auto-correct values behind the core API.
- **No hidden metadata dependency in phase 1:** `InlineStore` and baseline phase-1 stores expose plain replica slots, not side-channel validity bits or journaling state.

### 8.3 Minimal conceptual trait shape

The exact trait design may differ, but implementation must preserve a boundary equivalent to:

```rust
pub trait ReplicaStore<T, const N: usize> {
    fn read_slot(&self, index: usize) -> T;
    fn write_slot(&self, index: usize, value: T);
}
```

The internal implementation of these methods must be volatile and must use internal mutability appropriately. Public design may use associated constants, sealed traits, or typed indices, but it must still expose a contract equivalent to "core can read and write exactly `N` logical replica slots, individually, with no hidden policy."

### 8.4 Phase-1 backend outcome

Phase 1 requires `InlineStore` in `hdf-core`:

- baseline store backend used for correctness and tests;
- likely contiguous placement;
- acceptable for functional validation and software fault models;
- explicitly not positioned as strong physical separation;
- public and constructible in the phase-1 `hdf-core` crate, because it is the mandatory baseline backend for all initial examples, tests, and integrations.

### 8.5 Later backend outcomes

Later backends may include:

- `SplitStore` for separated memory regions or linker-controlled placement;
- encoded or complemented policies for bitwise-compatible types;
- platform-specific placement helpers that still conform to the logical store contract.

These later backends must not change DMR/TMR truth semantics; they only change storage realization and physical fault assumptions.

## 9. Concurrency Model

### 9.1 Core stance

`hdf-core` is not a concurrency-safe abstraction.

- read operations take `&self`;
- write and repair operations take `&mut self`;
- the type is intentionally `!Sync`;
- shared access from ISR/main, thread/thread, or task/task contexts is outside core guarantees.

### 9.2 Why this is fixed

TMR and DMR address corruption of stored replicas. They do not solve:

- readers observing mid-write state;
- torn updates;
- synchronization and ownership coordination;
- lock-free publication of multi-replica updates.

Combining those concerns in the core would distort the API and the fault model.

### 9.3 `hdf-sync` boundary

Shared access belongs to `hdf-sync`, which may provide:

- critical-section based wrappers for single-core embedded systems;
- integration points for external mutex/lock types;
- guarded read/write patterns for RTOS systems;
- optional metadata/version-guard helpers where the semantics are explicit.

`hdf-sync` may coordinate access to `hdf-core`, but it must not claim lock-free full-value atomicity for arbitrary `T`.

### 9.4 Verification implications

Core tests validate logical redundancy under exclusive access.

Sync-layer tests validate that wrapper policies prevent unsafe interleavings at the integration boundary. They do not alter core truth semantics.

## 10. Crate and Module Boundaries

The project is one workspace with distinct crates.

### 10.1 Workspace crates

- `hdf-core`: RAM-resident protected values, schemes, core reports, repair/write/read/check behavior.
- `hdf-sync`: shared-access wrappers and synchronization-oriented integration.
- `hdf-layout`: placement-aware stores and layout policies.
- `hdf-fault`: deterministic corruption injection and validation helpers.
- `hdf-storage`: persistent protected state with its own API and semantics.
- `hdf-journal`: compact observability for corruption and recovery events.
- optional later macro/helper crate only if structured data ergonomics clearly justify it.

### 10.2 `hdf-core` internal module boundaries

Expected module outcomes:

- public `scheme`: `Dmr`, `Tmr`, the public `Scheme` contract, and scheme-specific evaluation logic exposed only through stable reports.
- public `report`: `ReadReport`, `TrustedStatus`, `SuspectReason`, `CheckReport`, and `RepairOutcome`.
- public `store`: public store contract plus `InlineStore`.
- public `hardened`: `Hardened<T, Scheme, Store>` and core constructors/operations.
- internal `cell` or equivalent: `UnsafeCell` wrappers and volatile-access helpers.
- internal `util` or equivalent optional module for non-public helpers such as array handling or typed slot indexing.

Implementation should keep scheme logic separate from store logic and separate both from public reporting types.

### 10.3 `hdf-sync` internal outcomes

- public `wrapper` module with explicit adaptor types such as `CriticalSectionHardened<_, _, _>` and lock-backed wrappers around `Hardened` rather than new redundancy semantics.
- public `ops` or `access` module for operation-style APIs such as `with_read`, `with_write`, or guarded closure-based access when wrappers need to serialize access without exposing interior unsafety.
- public integration traits only where needed to abstract external lock providers; avoid redefining a universal mutex trait unless required by actual integrations.
- internal support modules for metadata/version guards if they exist.
- clear docs stating wrapper guarantees and non-guarantees.

Public direction for `hdf-sync` is intentionally narrow:

- prefer wrapper/adaptor types over modifying `hdf-core` traits;
- prefer explicit serialized-operation APIs over exposing raw shared mutable access;
- preserve core reports unchanged, so wrapped reads still return the same `ReadReport`/`CheckReport` meanings.

### 10.4 `hdf-layout` internal outcomes

- public `store` module with layout-aware backends such as `SplitStore` implementing the core store contract.
- public `placement` module for region/bank/section descriptors, linker-facing helpers, or placement configuration types.
- public optional `policy` module for later encoded/complemented placement policies, gated to explicitly supported types.
- internal platform glue modules as needed.

Public direction for `hdf-layout`:

- extend only the backend/placement surface, not DMR/TMR semantics;
- placement types describe where replicas live, while store types perform the actual slot reads/writes;
- any encoded/complemented extension remains a backend policy choice, not a new voting scheme.

### 10.5 `hdf-storage` internal outcomes

- public `record` module for persistent record shapes and metadata.
- public `report` module for storage-specific outcomes such as trusted, stale, corrupted, or rollback-selected records.
- public `load` or `recover` module for scanning media, selecting the best record, and producing a load result.
- public `persist` or `writer` module for committing updated records with storage-specific sequencing.
- internal codec/checksum/driver-adaptor modules as needed.

`hdf-storage` must reuse the trust/recovery philosophy without pretending NVM behaves like RAM.

Public direction for `hdf-storage` is a separate persistent API family:

- do not mirror `hdf-core::Hardened` method-for-method;
- provide an explicit load/recover flow such as scan -> classify -> select trusted record -> materialize in-memory protected state;
- make "load from storage into RAM hardening" a visible boundary, typically via conversion/adaptor helpers rather than hidden coupling.

### 10.6 `hdf-journal` internal outcomes

- public `event` module defining compact event records.
- public `writer` or `append` module for constrained append APIs.
- public `decode` module for host-side decoding outputs.
- internal encoding/storage-backend helpers as needed.

Required event taxonomy direction:

- integrity observed clean;
- recoverable mismatch observed;
- suspect or no-trusted-value observed;
- repair attempted;
- repair succeeded;
- repair not possible;
- storage recovery selected record or failed to recover, once `hdf-storage` is integrated;
- restart/reset cause only when the platform can provide it without expanding core semantics.

Host-side decoding outputs should at minimum support:

- decoded structured events suitable for analysis tools;
- human-readable text rendering for logs;
- timestamp/sequence preservation when present in the encoded source.

## 11. Testing and Verification Matrix

Testing is a first-class requirement, not a postscript.

### 11.1 Core unit tests

Required for phase 1:

- constructor cases: `new()` initializes every replica identically for DMR and TMR.
- DMR read matrix: clean `A A`, mismatch `A B`, and exact expected `ReadReport` payloads.
- DMR check/repair matrix: `A A` -> `Consistent`/`NoRepairNeeded`, `A B` -> `Suspect`/`NotPossible`.
- TMR read matrix: `A A A`, `A A B`, `A B A`, `B A A`, and `A B C` with exact expected reports.
- TMR check/repair matrix: all-equal, one-outlier, and no-majority cases with exact expected `CheckReport` and `RepairOutcome`.
- operation interaction cases: prior corruption followed by `write()`, repeated `repair()`, and proof that `read_checked()`/`check()` do not mutate slots.

### 11.2 Core boundary and invariants tests

- replica-count contract cases: store/scheme count agreement is enforced and mismatch cannot silently proceed.
- store invariants cases: stable slot indexing, per-slot independence, no hidden mirroring, and read freshness after external slot mutation in tests.
- volatile access cases: store reads observe current slot contents rather than cached values.
- build/configuration cases: API remains `no_std` compatible across supported targets.
- type-system cases: unsupported `T` such as non-`Eq` floats are rejected in phase 1.

### 11.3 Fault injection tests

Required once `hdf-fault` exists:

- single-slot corruption cases covering every logical replica position for DMR and TMR.
- two-slot corruption cases for TMR that distinguish majority-preserving from no-majority outcomes where applicable.
- deterministic torn-state simulations run against both direct core usage and sanctioned sync wrappers.
- repeated corruption/repair/write sequences to ensure no hidden state accumulates.
- suspect-path validation that raw replica snapshots are exposed exactly as stored.

### 11.4 Layout tests

Required once `hdf-layout` exists:

- conformance cases proving `InlineStore` and each layout backend produce identical `ReadReport`, `CheckReport`, and `RepairOutcome` for the same logical replica values.
- placement mapping cases for every replica index and configured region/bank mapping.
- encoded/complemented policy cases covering clean read, corrupted read, and round-trip write/read behavior where supported.

### 11.5 Storage tests

Required once `hdf-storage` exists:

- record-set classification cases: both valid, one valid/one stale, one valid/one corrupted, both corrupted.
- interrupted-update cases across every write phase the persistent format exposes.
- rollback-selection cases proving the chosen record is deterministic and justified by metadata rules.
- metadata/version monotonicity cases, including equal-version ties if the format permits them.
- power-loss style scenarios as far as the driver model allows.

### 11.6 Sync-layer tests

Required once `hdf-sync` exists:

- serialized-access cases for critical-section wrappers in representative ISR/main or task/task patterns.
- parity cases showing wrapped operations return the same `ReadReport`, `CheckReport`, and `RepairOutcome` as direct core use once access is serialized.
- misuse-boundary cases and docs proving the crate does not claim lock-free atomicity or unsynchronized mid-write safety for arbitrary `T`.

### 11.7 Documentation verification

Each phase must include runnable or compile-checked examples covering:

- clean read path;
- recoverable mismatch path for TMR;
- suspect path (`DmrConflict` and `NoMajority` in the relevant examples);
- explicit repair path where applicable;
- branch handling that shows the caller consuming reports explicitly rather than treating the API as plain `T` storage.

### 11.8 Benchmark and footprint verification

Required from `hdf-fault` onward, refined later:

- code size deltas for representative targets;
- runtime overhead for read/check/write/repair paths;
- storage overhead per scheme and backend;
- optional journal/write overhead once later crates exist.

## 12. Examples and Reference Integrations

Examples are required deliverables, not optional extras.

### 12.1 Phase-1 examples

Minimum examples:

Required exact example set:

- `examples/tmr_mode.rs`: protects a mode enum with TMR; demonstrates clean read, one-outlier recoverable mismatch, and explicit `repair()` returning `Repaired`.
- `examples/dmr_safety_flag.rs`: protects a safety flag with DMR; demonstrates clean read and detect-only conflict handling with `SuspectReason::DmrConflict`.
- `examples/read_report_branching.rs`: shows application control flow branching on `ReadReport` and `CheckReport` rather than assuming a plain value.
- `examples/inline_store_basics.rs`: demonstrates explicit construction with `InlineStore`, making the phase-1 backend contract visible.

### 12.2 Selective-hardening examples

Required when ergonomics work begins:

- `examples/state_machine_tmr.rs`: state machine state hardened with TMR.
- `examples/dmr_reloadable_threshold.rs`: threshold/config field hardened with DMR when an external authoritative reload path exists.
- `examples/tmr_command_counter.rs`: command counter hardened with TMR.
- `examples/selective_hardening_guide.rs`: mixed application state showing which fields are hardened and which are intentionally left unhardened.

### 12.3 Structured-data examples

Required once structured support exists:

- `examples/struct_config.rs`: small `Copy + Eq` config struct.
- `examples/composite_state.rs`: composite type showing equality-based protection still works without new voting semantics.
- `examples/unsupported_types.rs`: documentation-oriented example explaining why larger, heap-backed, or ambiguous types are not automatically included.

### 12.4 Reference embedded integration

Required for the dedicated integration phase:

- `examples/firmware-demo/`: small firmware-style example or demo project.
- it must demonstrate protected state transition flow using `hdf-core` and, where shared access exists, `hdf-sync` wrappers.
- it must include at least one injected runtime corruption scenario and the resulting safe application behavior.
- it must include a measured overhead summary for the demonstrated target/configuration.
- it must include integration notes covering placement choice, synchronized access choice, and handling of suspect reports.

## 13. Implementation Sequence

Implementation should proceed in the following order to minimize architectural churn:

1. freeze core semantics and reporting types in code-facing design notes;
2. implement `hdf-core` reports, scheme logic, volatile store access helpers, and `InlineStore`;
3. implement `Hardened<T, Scheme, Store>` read/check/write/repair with exhaustive core tests;
4. add documentation examples for DMR and TMR before expanding the surface;
5. build `hdf-sync` without altering core semantics;
6. build `hdf-layout` for separated placement and optional encoded policies;
7. add `hdf-fault` deterministic corruption support and evidence-driven verification;
8. add structured-data ergonomics only after baseline semantics are stable;
9. implement `hdf-storage` as a separate API family;
10. implement `hdf-journal` observability;
11. deliver reference embedded integration;
12. perform API cleanup and release stabilization.

No later phase should retroactively force concurrency, persistence, or journaling concerns into `hdf-core`.

## 14. Detailed Phased Roadmap

The roadmap includes phases 0-6 and later roadmap work already accepted in the architecture.

### Phase 0 - Specification Freeze

**Objective**

Lock the architecture so implementation work is about execution details, not semantic redesign.

**Scope**

- fault model;
- volatile memory-access stance;
- core read/check/write/repair semantics;
- DMR and TMR truth rules;
- single-owner `!Sync` stance for core;
- store backend split from scheme logic;
- crate boundaries.

**Exact implementation targets**

- final engineering spec;
- code-facing type and module outline;
- normative operation semantics section;
- accepted non-goals and boundary statements.

**Required public API/types or module outcomes**

- stable definitions for `ReadReport`, `TrustedStatus`, `SuspectReason`, `CheckReport`;
- declared existence of `Hardened<T, Scheme, Store>`, `Dmr`, `Tmr`, and `InlineStore`.
- declared responsibilities of `Scheme` versus `Store`, including fixed replica-count agreement.

**Tests/verification required**

- design review against fault model and accepted architecture;
- no unresolved contradictions in read/repair/concurrency semantics.

**Examples/docs required**

- at least one DMR and one TMR behavior example in the spec.

**Explicit deliverables**

- approved specification document;
- implementation sequence;
- roadmap with exit criteria.

**Exit criteria**

- no ambiguity about what the framework guarantees;
- no ambiguity about what the framework refuses to do.

**Dependency on prior phases**

- none.

### Phase 1 - `hdf-core`

**Objective**

Deliver a usable `no_std`, allocation-free RAM core for selectively hardened values.

**Scope**

- `T: Copy + Eq` only;
- DMR and TMR only;
- `InlineStore` only;
- public `RepairOutcome` enum;
- `read_checked`, `check`, `write`, and explicit `repair`;
- volatile replica access;
- exclusive-access semantics only.

**Exact implementation targets**

- `hdf-core` crate;
- opaque `Hardened<T, Scheme, Store>` implementation;
- scheme logic for DMR/TMR;
- `InlineStore` backend;
- report enums and repair outcome type;
- internal `UnsafeCell` + volatile slot access utilities;
- `no_std` configuration.

**Required public API/types or module outcomes**

- `ReadReport`, `TrustedStatus`, `SuspectReason`, `CheckReport`;
- `RepairOutcome`;
- `Dmr`, `Tmr`, `InlineStore`, public `Scheme`/store contracts or an equivalent public boundary;
- constructors and core operations;
- public docs that `read_checked` is non-mutating and DMR is detect-only.

**Tests/verification required**

- full unit coverage for DMR/TMR truth cases;
- repair behavior tests;
- non-mutation tests for `read_checked` and `check`;
- `no_std` build validation;
- compile-fail or type-level assurance for unsupported float-like phase-1 types where relevant.

**Examples/docs required**

- basic DMR example;
- basic TMR example;
- example branching on `ReadReport`;
- docs explaining why `repair()` is explicit.

**Explicit deliverables**

- working `hdf-core` crate;
- tests;
- rustdoc-level API docs;
- example programs.

**Exit criteria**

- all core truth rules are implemented and tested;
- public API reflects accepted report framing;
- no concurrency or persistence semantics leaked into the core API.

**Dependency on prior phases**

- phase 0.

### Phase 2 - `hdf-sync`

**Objective**

Support shared access patterns without corrupting core semantics.

**Scope**

- critical-section wrappers for MCU-style systems;
- external lock/mutex integration points;
- guarded shared access patterns;
- optional narrow metadata/version guard helpers if needed.

**Exact implementation targets**

- separate `hdf-sync` crate;
- wrapper types over `hdf-core` values;
- wrapper/adaptor types and/or serialized operation APIs over `hdf-core` values;
- critical-section integration path;
- lock-backed integration examples.

**Required public API/types or module outcomes**

- wrapper types or traits making synchronized access explicit;
- documentation of guarantees and non-guarantees;
- no change to `hdf-core` report semantics.

**Tests/verification required**

- wrapper tests showing safe serialized access in representative scenarios;
- behavioral parity tests showing wrapped reads produce same trusted/suspect results as core;
- tests/docs proving the crate does not claim lock-free arbitrary-`T` updates.

**Examples/docs required**

- ISR/main-loop protected access example;
- RTOS/mutex integration example;
- doc section explaining torn-update risk if sync wrappers are not used.

**Explicit deliverables**

- `hdf-sync` crate;
- synchronization examples;
- documentation of wrapper semantics.

**Exit criteria**

- users have a sanctioned path for shared access;
- core API remains unchanged in meaning;
- synchronization guarantees are explicit and narrow.

**Dependency on prior phases**

- phase 1.

### Phase 3 - `hdf-layout`

**Objective**

Provide a path beyond naive contiguous replica placement.

**Scope**

- layout-aware store backends;
- split-region/section placement hooks;
- optional encoded/complemented policies for bitwise-compatible types only.

**Exact implementation targets**

- `hdf-layout` crate;
- `SplitStore` or equivalent backend;
- placement descriptors/configuration and integration guidance;
- clearly delimited encoded-policy extension points.

**Required public API/types or module outcomes**

- store backends conforming to the core store contract;
- optional type gates for encoded policies;
- docs separating logical redundancy from physical placement.

**Tests/verification required**

- behavior parity between `InlineStore` and `SplitStore`;
- placement mapping tests;
- encoded-policy round-trip tests where implemented.

**Examples/docs required**

- memory-region split example;
- linker/placement guidance where platform-appropriate;
- clear note that layout improves physical plausibility but does not change truth-table semantics.

**Explicit deliverables**

- `hdf-layout` crate;
- layout examples;
- backend conformance tests.

**Exit criteria**

- users can choose between baseline and stronger placement backends;
- no change to DMR/TMR logical semantics;
- physical-layout claims remain honest and bounded.

**Dependency on prior phases**

- phase 1.

### Phase 4 - `hdf-fault`

**Objective**

Make the framework demonstrably verifiable under injected corruption.

**Scope**

- feature-gated fault injection;
- per-replica corruption utilities;
- deterministic multi-case scenario support;
- torn-state simulation for sync/documentation validation;
- benchmark harnesses.

**Exact implementation targets**

- `hdf-fault` crate or gated module set;
- corruption APIs for tests and examples;
- benchmark setup for read/check/write/repair overhead.

**Required public API/types or module outcomes**

- test-only replica mutation interfaces;
- scenario helpers for DMR conflict, TMR outlier, and no-majority cases.

**Tests/verification required**

- full corruption matrix across DMR and TMR;
- repair verification after injected corruption;
- benchmark data captured for representative target configurations.

**Examples/docs required**

- example showing clean, corrected, and suspect outcomes via fault injection;
- docs on how to validate integration code against expected corruption paths.

**Explicit deliverables**

- `hdf-fault` tooling;
- verification suite;
- baseline overhead report.

**Exit criteria**

- core claims are backed by deterministic tests and injection scenarios;
- users can reproduce major correctness behaviors.

**Dependency on prior phases**

- phase 1, with optional integration points from phases 2 and 3.

### Phase 5 - Selective-Hardening Ergonomics

**Objective**

Make targeted adoption practical without broadening the semantic model.

**Scope**

- clearer aliases or wrappers for common scheme choices;
- policy-oriented helpers for detect-only versus recoverable values;
- guidance for deciding what to harden;
- intentionally narrow ergonomic improvements.

**Exact implementation targets**

- aliases such as common DMR/TMR wrapper names if justified;
- helper constructors;
- documentation patterns for selective hardening decisions.

**Required public API/types or module outcomes**

- optional ergonomic aliases that do not obscure underlying reports;
- docs that keep `ReadReport`-based handling explicit.

**Tests/verification required**

- examples compile and preserve the same semantics as direct core use;
- no helper may hide suspect-state handling.

**Examples/docs required**

- state machine, mode flag, threshold, and counter examples;
- "what to harden and why" guide;
- "what not to harden" guide.

**Explicit deliverables**

- ergonomics additions;
- selective-hardening guidance;
- expanded examples.

**Exit criteria**

- integration is simpler for real firmware users;
- ergonomics do not compromise explicit trust semantics.

**Dependency on prior phases**

- phase 1, ideally informed by phases 2-4.

### Phase 6 - Structured Data Support

**Objective**

Extend protection to small composite values while preserving inspectability and simple equality-based semantics.

**Scope**

- small structured `Copy + Eq` types;
- support patterns for protected composite data;
- optional derive or helper crate only if it materially improves adoption without hiding semantics.

**Exact implementation targets**

- docs and examples for composite protected types;
- optional helper macro crate if justified;
- validation of layout/comparison expectations for small structs.

**Required public API/types or module outcomes**

- no change to the base trust/report model;
- explicit rules for which composite types are supported;
- optional derive output must still map to the same core abstractions.

**Tests/verification required**

- composite struct DMR/TMR tests;
- docs or tests confirming equality-based voting remains the only phase semantics;
- reject unsupported ambiguous types.

**Examples/docs required**

- composite config/state example;
- docs explaining why large, heap-backed, or semantically ambiguous types remain out of scope.

**Explicit deliverables**

- structured-data support patterns;
- examples;
- optional helper crate if justified by clear ergonomic benefit.

**Exit criteria**

- realistic small application data can be protected without semantic ambiguity;
- the framework remains explicit and inspectable.

**Dependency on prior phases**

- phase 1, with strong benefit from phase 5 guidance.

### Phase 7 - `hdf-storage`

**Objective**

Extend trust/recovery concepts to persistent critical state through a separate API family.

**Scope**

- A/B or redundant record patterns;
- versioning and integrity metadata;
- rollback to last valid record;
- stale versus corrupted distinction;
- Flash/EEPROM-aware semantics.

**Exact implementation targets**

- `hdf-storage` crate;
- record and metadata formats;
- read-selection and rollback logic;
- integration path from persisted configuration to protected in-memory values through an explicit load/recover flow.

**Required public API/types or module outcomes**

- storage-specific report types if needed;
- explicit distinction between invalid, stale, and trustworthy records;
- APIs that do not reuse RAM-core signatures when NVM semantics differ.

**Tests/verification required**

- interrupted-update scenarios;
- record-selection tests;
- rollback verification;
- stale/corrupted classification coverage.

**Examples/docs required**

- persisted config example;
- load-into-core example;
- docs explaining why storage is a separate crate from `hdf-core`.

**Explicit deliverables**

- `hdf-storage` crate;
- persistence examples;
- recovery-flow documentation.

**Exit criteria**

- persistent critical state has a credible, bounded API;
- storage semantics do not pollute the RAM core API.

**Dependency on prior phases**

- phase 1; informed by phases 3 and 4.

### Phase 8 - `hdf-journal`

**Objective**

Add observability for integrity failures and recovery actions.

**Scope**

- compact event schema;
- record corruption and correction events;
- record reboot/recovery cause where available;
- host-side inspection helpers.

**Exact implementation targets**

- `hdf-journal` crate;
- constrained-footprint event encoding;
- decode/inspection utilities with structured and human-readable outputs.

**Required public API/types or module outcomes**

- event type definitions;
- append/write API with documented footprint expectations;
- host-side decoding interface.

**Tests/verification required**

- encode/decode round-trip tests;
- event ordering and truncation policy tests;
- examples showing corruption and repair history.

**Examples/docs required**

- recovery trace example;
- host-side inspection example;
- docs on minimal-footprint deployment.

**Explicit deliverables**

- `hdf-journal` crate;
- traceability examples;
- decoding helpers.

**Exit criteria**

- integrity-related events are diagnosable in realistic deployments;
- observability remains modular and does not bloat core.

**Dependency on prior phases**

- phase 1; stronger value after phases 4 and 7.

### Phase 9 - Reference Embedded Integration

**Objective**

Validate the framework inside a realistic embedded workflow rather than as isolated crates only.

**Scope**

- small firmware demo or reference project;
- protected state transitions;
- synchronized access where needed;
- corruption injection during scenario testing;
- practical overhead measurement.

**Exact implementation targets**

- reference application project;
- representative MCU or embedded target support;
- integration guide covering core, sync, and layout choices.

**Required public API/types or module outcomes**

- no new semantic API required, but integration wrappers and examples must reflect the accepted boundaries.

**Tests/verification required**

- end-to-end scenario tests;
- measured RAM/flash/code-size overhead summary;
- verification that suspect states drive safe application behavior.

**Examples/docs required**

- full reference integration;
- step-by-step integration notes;
- fault-injection walkthrough in a firmware context.

**Explicit deliverables**

- reference demo;
- integration documentation;
- test and measurement report.

**Exit criteria**

- the framework is demonstrated in realistic embedded usage;
- claims about selective adoption are backed by an end-to-end example.

**Dependency on prior phases**

- phase 1 minimum; best after phases 2-4 and optionally 8.

### Phase 10 - OSS Stabilization and Release

**Objective**

Turn the project from an internal engineering effort into a stable external release.

**Scope**

- API cleanup and naming normalization;
- documentation pass;
- semver baseline;
- packaging and release process;
- onboarding material;
- roadmap and issue hygiene.

**Exact implementation targets**

- crate versioning policy;
- README and crate-level docs;
- changelog/migration notes if needed;
- workspace release automation where appropriate.

**Required public API/types or module outcomes**

- public API reviewed for coherence across crates;
- unstable/internal items clearly hidden or feature-gated;
- documentation reflects exact guarantees and non-goals.

**Tests/verification required**

- CI for builds, tests, docs, and examples;
- supported-target validation matrix;
- release-candidate review against readiness criteria.

**Examples/docs required**

- polished getting-started example;
- crate-overview docs;
- contribution and roadmap docs.

**Explicit deliverables**

- first stable OSS release;
- public docs set;
- release notes and roadmap.

**Exit criteria**

- APIs are documented and semver-ready;
- core and companion crates have coherent boundaries;
- examples, tests, and docs are sufficient for external adopters.

**Dependency on prior phases**

- at least phases 1, 4, and 9; optionally later crates may remain pre-1.0 if not stabilized.

## 15. Release Readiness and Stabilization Criteria

Before public stabilization, the project must satisfy all of the following for the crates being released as stable:

- semantic stability: DMR/TMR truth rules and explicit repair semantics are unchanged and documented;
- API clarity: `ReadReport`, `TrustedStatus`, `SuspectReason`, and `CheckReport` are documented with examples;
- boundary clarity: docs clearly separate core, sync, layout, storage, and journal responsibilities;
- test adequacy: required matrix sections for the released crates are implemented in CI;
- `no_std` validation: released core crates build without `std`;
- example quality: examples compile and cover trusted, recoverable mismatch, and suspect flows;
- integration proof: at least one realistic reference integration exists for the stable workflow being promoted;
- performance transparency: footprint and overhead are measured and published for representative targets;
- honesty of claims: documentation does not imply atomicity, adversarial protection, or strong physical-fault guarantees beyond the accepted model;
- release hygiene: semver policy, changelog/release notes, README, crate docs, and roadmap are present.

If some later crates are not mature enough, they may remain experimental while `hdf-core` and selected companions stabilize first. Stabilization should be crate-by-crate if necessary, not blocked by unfinished long-range roadmap items.

## 16. Final One-Sentence Definition

Hardened Data Framework is a `no_std` Rust framework for selectively protecting critical embedded data through redundant storage, explicit integrity reporting, and scheme-aware recovery while keeping redundancy logic, synchronization, layout, and persistence as separate architectural concerns.
