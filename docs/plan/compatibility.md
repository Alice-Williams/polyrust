# PolyRust implementation plan

This roadmap implements the v0.1 proof of concept described by the charter. Every
milestone has its own file in `milestones/` with an outcome, implementation
checklist, required exit evidence, and scope boundary.

## Delivery principles

- Semantics precede syntax lowering.
- Rust output is required from the first backend milestone.
- Rust, TypeScript, Python, and Go form the completed initial proof; M21 adds
  JavaScript and Java as release gates.
- Unsupported behavior fails before output is written.
- Portable tests are declarations in the base program and become native tests in
  every output.
- A feature is complete only when checker, evaluator, all required backends,
  and conformance tests agree.
- Backends share infrastructure and contracts, not target-language policy.

## Milestones

### Step 0 — Linux/Bazel development baseline

M00 establishes the bind-mounted Linux Dev Container, Bazel/Bazelisk,
hermetic Rust and Go SDKs, caches, and cross-language smoke tests. Every later
milestone runs through this environment on non-Linux hosts.

Exit gate:

- a clean image build succeeds;
- `bazel test //...` passes one Rust and one Go test;
- toolchain versions match the checked-in pins; and
- no host compiler or build tool is needed.

### M0 — Workspace and semantic spine

M01–M06 establish the workspace, IR, diagnostics, checker,
evaluator, and Rust authoring API. At this point a generator program can build
and evaluate a checked module but cannot emit source.

Exit gate:

- the demonstration module includes constants, a restricted contract and
  implementation, pure functions/methods, and portable tests;
- it can be built through Rust and round-trip through JSON;
- invalid programs yield stable diagnostics;
- at least 20 semantic vectors and ten portable tests pass in the evaluator; and
- no target-specific types appear in core IR.

### M1 — Backend platform and CLI

M07–M09 implement structured documents, the backend contract, manifests,
atomic writing, and CLI wiring.

Exit gate:

- a test backend can generate a deterministic multi-file manifest;
- capability preflight reports all missing features;
- unsafe output paths are rejected; and
- CLI check/emit/targets/explain behavior is integration-tested.

### M2 — Required source backends

M10–M13 implement Rust, TypeScript, Python, and Go. Work may overlap only
after M08 freezes the first backend contract. Rust is completed first and is
the reference for API review, not a special core path.

Exit gate:

- every generated package formats and passes its native static/build tests;
- package snapshots are deterministic;
- difficult numeric, Unicode, enum, option/result, and list behavior is present;
- every backend emits and passes the demonstration's native tests;
- restricted contract dispatch compiles and behaves consistently; and
- Go's pointer/slice choices preserve PolyRust value semantics.

### M3 — Cross-language proof

M14 builds the differential harness and expands the conformance corpus.

Exit gate:

- at least 50 vectors agree across evaluator plus four targets;
- portable test outcomes agree in all native test frameworks;
- injected helpers and dependencies are reported;
- repeated generation is byte-identical; and
- seeded semantic mutations are caught by tests.

### M4 — Usability and release readiness

M15–M16 deliver examples, extension documentation, an out-of-tree test
backend, CI, benchmarks, and release gates.

Exit gate:

- the complete example is generated with one command and no hand edits;
- an external toy backend compiles without core target-name changes;
- CI exercises supported toolchain versions on Windows and Linux; and
- all PRD success measures have recorded evidence.

## Task dependency graph

```text
000 Linux/Bazel environment
 └─001 Workspace
 ├─002 IR ─────┬─004 Validator ─┬─005 Interpreter
 │             │                └─006 Rust builder
 ├─003 Diagnostics ──────────────┘
 └─007 Document writer

002 + 003 + 004 + 007
 └─008 Backend API/manifests
    └─009 CLI/atomic output
       ├─010 Rust backend ───────┐
       ├─011 TypeScript backend ┤
       ├─012 Python backend ────┼─014 Conformance harness
       └─013 Go backend ────────┘

006 + 010–014 ──015 Examples and extension proof
001 + 009–015 ──016 CI, benchmarks, release gate
```

## Ordered task list

| ID | Task | Depends on | Milestone |
| --- | --- | --- | --- |
| 000 | Establish Linux/Bazel development baseline | — | Step 0 |
| 001 | Scaffold Rust workspace and quality baseline | 000 | M0 |
| 002 | Define versioned PolyIR and canonical serialization | 001 | M0 |
| 003 | Implement structured diagnostics and source references | 001 | M0 |
| 004 | Implement resolver, type checker, and capability analysis | 002, 003 | M0 |
| 005 | Implement reference evaluator, portable tests, and canonical values | 002, 004 | M0 |
| 006 | Implement typed Rust builder API | 002, 003, 004 | M0 |
| 007 | Implement structured document writer | 001 | M1 |
| 008 | Implement backend API, registry, preflight, and manifests | 002, 003, 004, 007 | M1 |
| 009 | Implement CLI and safe atomic output | 006, 008 | M1 |
| 010 | Implement required Rust backend | 008, 009 | M2 |
| 011 | Implement required TypeScript backend | 008, 009 | M2 |
| 012 | Implement required Python backend | 008, 009 | M2 |
| 013 | Implement required Go backend | 008, 009 | M2 |
| 014 | Build four-target differential conformance harness | 005, 010, 011, 012, 013 | M3 |
| 015 | Deliver examples, author guide, and external backend proof | 006, 010, 011, 012, 013, 014 | M4 |
| 021 | Add derived JavaScript and independent Java output gates | 014, 016, 020 | M5 |
| 022 | Add independent C++20 and C17 output gates | 021 | M6 |
| 016 | Add CI matrix, determinism benchmark, and release gate | 001, 009–015 | M4 |

## Suggested sequencing

The shortest correctness-first critical path is:

`001 → 002/003 → 004 → 005/006/007 → 008 → 009 → 010 → 011/012/013 → 014 → 015 → 016`

After M08, backend milestones are structurally parallel, but completing Rust first
is recommended because it exposes IR-to-strong-type mapping issues early. Do not
freeze public generated APIs based only on Rust; review all four target mapping
documents before declaring an IR feature stable.

After M21, review all six target mappings; after M22, review all eight. The
original four-target rows above remain the dependency history for M10–M14.

## Estimated effort bands

These are engineering estimates for planning, not commitments:

| Area | Estimate |
| --- | ---: |
| M0 semantic spine | 4–7 weeks |
| M1 backend platform and CLI | 2–3 weeks |
| M2 four backends | 6–10 weeks |
| M3 conformance | 2–4 weeks |
| M4 usability/release readiness | 2–4 weeks |
| Total proof-of-concept to documented alpha | 16–28 weeks |

The ranges assume one experienced engineer, normal review, and the v0 feature
freeze. Adding platform I/O, mutable aliasing, generic/default contracts, or async
invalidates the estimate.

## Risk-driven spikes before feature expansion

1. Confirm one `i64` checked/wrapping program across all four targets.
2. Confirm scalar-string behavior for an astral character and combining sequence.
3. Confirm nested tagged enums and `Result<Option<T>, E>` public APIs.
4. Confirm one contract parameter dispatches to a record implementation in all
   four generated targets.
5. Confirm the same ten portable tests run through the evaluator and every native
   target test framework.
6. Confirm Go slice/pointer lowering cannot violate immutable value semantics.
7. Confirm an out-of-tree backend can consume checked IR without depending on a
   concrete backend crate.

Failure in a spike triggers an IR/spec revision, not a target-specific semantic
exception.

## Post-v0 candidate backlog

These are intentionally not mixed into the proof-of-concept tasks:

1. Add Rust-like macro/attribute syntax that lowers into unchecked PolyIR.
2. Investigate a restricted ordinary-Rust parser only after the common model is
   stable; it must reject constructs without mapped PolyRust semantics and use
   the same checker.
3. Add generic records, functions, and contracts with portable bounds.
4. Add declared host capability adapters, beginning with clock and logging.
5. Specify insertion-ordered maps/sets and deterministic iteration.
6. Specify mutable collections and observable aliasing.
7. Investigate a portable reference/pointer capability using Rust and Go as the
   first design pair, then prove its behavior in TypeScript and Python.
8. Add one independently maintained external target backend.

## Toolchain note

The current machine reports Rust 1.86.0, Node 23.11.0, and Python 3.7.0; Go is not
currently installed. Before implementation, M01 must select supported
versions, provision/pin them in CI, and avoid mistaking the present local versions
for the product compatibility policy.
