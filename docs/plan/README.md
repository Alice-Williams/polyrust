# Engineering plan

This plan turns the portable code-generation feasibility study into reviewable,
dependency-ordered work. Each milestone has a separate file under
`milestones/`; its exit evidence becomes a permanent gate for later work.

## How to use this plan

1. Work on the earliest unblocked milestone, except for explicitly independent
   research or test preparation.
2. Set its status to `in-progress` before implementation.
3. Keep outcome, checklist, required evidence, and scope boundary visible in the
   milestone file.
4. Cite the milestone ID in commits.
5. Commit and push after each completed milestone.
6. Mark a milestone `complete` only after every exit criterion has evidence.

Valid statuses are `planned`, `in-progress`, `blocked`, and `complete`.

## Architecture and sequencing

- [Feasibility and risk analysis](analysis.md)
- [Phases, dependency graph, and release cuts](compatibility.md)
- [Product charter](../charter.md)
- [Portable language map](../portable-language.md)
- [Technical architecture](../architecture.md)
- [Decision record template](../adr/0000-template.md)

The critical path is:

`Linux/Bazel baseline -> unchecked IR -> checker -> evaluator/builder -> backend contract -> Rust/Go proof -> remaining targets -> conformance`

Existing portable-input frontends lower to the same unchecked IR. All safe backends accept
only checked programs. Rust output is required and follows the same backend
contract as every other target.

## Milestones

### Current priority — Rust compiler frontend experiment

- [M35 — Rust compiler frontend proof](milestones/M35-rustc-frontend-proof.md) — in progress
- [M35-03A — Runtime-free C/Java parity and cleanup](tasks/M35-03A-runtime-free-parity.md) — in progress.
  Completed bounded steps include Boolean/i64/bitwise operations, authenticated
  bool/i32/i64 constants and aliases, unit results, wrapping negation, binary64
  values/comparisons, negation, NaN classification, absolute value and truncation.
  Binary64 arithmetic (02N), including checked source integration, is complete
  with all 879 release/lint tests passing and two clean independent reviews.
  Truncating remainder (02O) is complete through checked compiler integration,
  with an independent oracle, measured Rust/target traces, mutation-sensitive
  API/privacy/docs proof and all 900 release/lint targets passing.
  Remaining parity gaps must be implemented before
  legacy runtime removal.
  [Negative-zero composition (02P)](tasks/M35-03A-02P-negative-zero-composition.md)
  is complete: 86,017 native/upstream/bit-oracle cases, 903 passing release/lint
  targets and clean independent review. It reuses ordinary Rust operations
  without a new compiler capability or custom runtime; wider scalar parity
  remains the active workstream.
  [Wrapping addition (02Q)](tasks/M35-03A-02Q-wrapping-addition.md) is complete
  through checked compiler integration: 15,790 native Rust/oracle cases, 31,580
  target observations per run, measured operand traces, typed dataflow/atomic
  controls and actual exported source-owned packages. All 924 release/lint targets
  pass and two whole-scope source reviews are clean. Remaining scalar arithmetic
  and the broader parity inventory remain active work, not legacy-removal approval.
  [Wrapping subtraction (02R)](tasks/M35-03A-02R-wrapping-subtraction.md) is
  complete through its independent oracle, C/Java foundations and checked source
  integration. All 945 release/lint targets pass and whole-scope review is clean.
  Native proof covers 15,790 Rust/oracle inputs, 31,580 target observations per
  run, original operand traces and seven compiling value/order faults. Typed
  dataflow, atomic rejection and original API/docs/privacy controls pass; actual
  multi-crate packages are exported. Wider scalar/runtime parity remains open.
  [Wrapping multiplication (02S)](tasks/M35-03A-02S-wrapping-multiplication.md)
  has a completed independent oracle: 34,546 exact products match native Rust
  with checks on/off; all 948 release/lint targets pass and review is clean.
  Its C target foundation is complete too: certified unsigned products and
  guarded signed reconstruction, six new tests, all 948 release/lint targets
  passing and clean review. Java target certification is complete: 405 Java unit
  tests, five compiling fault controls at both widths, all 948 release/lint
  targets passing and clean independent review. Checked source integration is
  complete too: all 966 release/lint targets pass, whole-scope review is clean,
  34,546 Rust/oracle inputs and 69,092 target observations per run agree, and six
  compiling value/order faults are detected. Typed dataflow, atomic boundaries,
  original APIs/docs/privacy and actual exported packages accompany the proof.
  Existing output/WIP is unchanged; wider migration remains incomplete.
  [Signed widening (02T)](tasks/M35-03A-02T-signed-widening.md) next addresses
  exact i32-to-i64 conversion, with independent oracle, C/Java foundations and
  compiler integration kept as separately gated/reviewed checkpoints.
  Its independent oracle is complete: 73,890 exact signed inputs, native Rust
  as/From at both optimization/check settings, three faulty conversion controls,
  all 969 release/lint targets passing and clean whole-scope review. Target
  foundations and checked source admission remain separate work. The C foundation
  is complete: exact I32-to-I64 certification, six focused cases, 73,890 native
  inputs with compiling fault controls, all 969 release/lint targets passing and
  clean independent review. Java foundation is complete too: seven focused tests,
  14,000 cast annotation combinations, strict native direct/materialized packages,
  all 969 release/lint targets passing and clean review. Both foundations preserve
  old output bytes. Checked source integration is complete: all 988 release/lint
  targets pass and whole-scope review is clean. Native proof covers 73,890
  original Rust/oracle inputs, measured operand calls, five compiling faults,
  seven compositions and exact typed/atomic/API/privacy checks. Actual packages
  are exported, prior output/WIP is unchanged, and wider migration remains open.
  [Finite f64 constants (02U)](tasks/M35-03A-02U-finite-f64-constants.md) is the
  next bounded increment: independent bit truth, separate C/Java constant
  foundations, then checked source declarations/reads/imports. Nonfinite
  constants and type-alias uses remain excluded; no broader parity is claimed.
  The independent oracle is complete: 24,566 finite patterns plus ten computed
  constants, real native fault controls, all 991 release/lint targets passing
  and clean independent review. The C constant foundation is complete too:
  exact finite storage/import certification, 24,576 native bit observations
  per compiler configuration, mutation-sensitive alias/read tests and all
  992 release/lint targets passing with clean review. Old output/WIP is
  unchanged. The Java foundation is complete: exact primitive-double constant
  certification, 24,576 native field observations in normal/interpreted Java21,
  inlining-sensitive mutation proof, all 994 release/lint targets passing and
  two clean independent reviews. Checked source admission is now complete:
  1,001 release/lint targets pass, fresh whole-scope review is clean, and native
  exact-bit, typed/atomic/API/privacy and actual cache-invalidation proof pass.
  Four actual C/Java packages are exported; 423 prior output hashes and 38
  unrelated WIP hashes are unchanged. Wider scalar/runtime parity remains open.

  [Signed-infinity constants (02V)](tasks/M35-03A-02V-infinite-f64-constants.md)
  is the next bounded increment. Its independent oracle, typed standard C/Java
  constant foundations and checked compiler integration remain separate gates.
  NaN constants stay unsupported; finite witnesses are not widened.
  The independent oracle is complete: 20 native computed constants, 24,934
  classification patterns, actual value faults and independently observed
  predicates pass at both Rust settings. All 1,004 release/lint targets pass,
  whole-scope/hardening reviews are clean and existing output/WIP is unchanged.
  Its C foundation is now complete: typed standard constants, exact signed
  inventory/import facts, inferred math.h without libm, native compiling-fault
  proof and all 1,005 release/lint targets passing. Broad independent review is
  clean; all 462 previous output hashes and 38 unrelated WIP hashes are unchanged.
  Java foundation is complete too: exact standard-field inventory and aliases,
  separate native compilation with inlining-sensitive faults, all 1,006
  release/lint targets passing and clean broad review. Checked Rust-source
  integration is complete too: 39 original reads, 66 target observations per
  configuration, typed/atomic/API/privacy proof, three compiling faults and
  actual exported packages. All 1,012 release/lint targets pass and broad review
  is clean. Producer-sign changes invalidate seven affected actions; restoring
  source recovers original hashes and a cached native pass. Existing output/WIP
  is unchanged; NaN constants and wider migration remain open.

  [Unicode scalar values (02W)](tasks/M35-03A-02W-character-values.md) is the next
  bounded increment: independent full-domain Rust truth, separate C/Java typed
  foundations, then checked character literal/transport/comparison integration.
  Source admission remains disabled until those prerequisites are proved.
  The oracle is complete: every scalar/surrogate, 80 out-of-range inputs,
  4,453 comparison pairs and seven actual fault families match the independent
  model at both Rust profiles. All 1,015 release/lint targets pass and broad
  review is clean. All 501 prior output files and 38 WIP files are unchanged.
  Its C target foundation is complete too: full-domain certified U32 transport,
  six comparisons, original dependency forwarding and private record/helper
  coverage pass under GCC14/Zig O0/O2 and GCC UBSan. Actual compiling width/order
  faults are detected. All 1,016 release/lint targets pass, fresh broad review
  is clean and all prior output/WIP hashes remain unchanged. Java certification
  and checked Rust-source character admission remain separate pending steps.
  Java target certification is now complete too: primitive Int full-domain
  storage, original dependency forwarding, strict separate Java21 compilation
  and normal/interpreted runs pass with four compiling fault controls. All
  1,017 release/lint targets and 427 Java unit cases pass, and fresh broad
  review is clean. Comparison annotations are checked consistently; old output
  and unrelated WIP remain unchanged. Checked source integration is now complete:
  all 1,024 release/lint targets pass, fresh broad reviews are clean and actual
  producer-value changes invalidate all seven affected actions. Restoring source
  recovers original hashes and cached native success. Native proof covers
  1,116,517 rows and 19 literal boundaries, actual call traces and compiling
  faults. Typed source identities distinguish Char from I32 and authenticate
  field owners and dependency signatures; atomic boundaries and original
  API/docs/privacy checks pass. Actual three-owner examples are exported.
  All 501 prior generated hashes and 38 unrelated WIP hashes remain unchanged.
  Character constants, text and the wider runtime migration remain open.

  [Unicode scalar constants (02X)](tasks/M35-03A-02X-character-constants.md)
  is the next bounded increment: independent named/computed constant truth,
  separate C/Java target foundations, then checked declarations/reads/imports.
  The oracle is complete: 4,127 compile-time values at both Rust profiles,
  four native fault columns, invalid const conversions and strict protocol
  controls pass. All 1,027 release/lint tests pass and fresh broad review is
  clean. No character constant source admission is enabled. Target integer
  inventories must not infer original Rust Char identity.
  The C foundation is complete: exact U32 constant storage/import authority,
  4,133 native objects/readers through original-owner aliases, five compiler
  profiles and four compiling faults pass. All 1,028 release/lint tests pass
  and fresh whole-scope review is clean. All 530 prior generated hashes and
  38 unrelated WIP hashes remain unchanged; Java/source steps remain separate.
  Java target proof is complete too: all 4,133 fields/local/imported readers
  agree with independent truth under normal/interpreted Java21; recompiled
  mutations, original aliases, readonly fields and exact capacity controls pass.
  All 1,029 release/lint tests and 436 Java unit cases pass; two fresh broad
  reviews are clean. Existing output/WIP is unchanged. Checked source integration
  is complete too: original Char/value/owner joins, constant-only producers,
  aliases and private/local/inherent reads pass native and atomic controls.
  All 1,033 release/lint targets pass and fresh GPT-6-Sol extra-high reviews are
  clean. Producer-value invalidation and restored cached success are proven;
  real packages are exported, with all 530 preceding output hashes and 45
  unrelated WIP hashes unchanged. Conversions/text and wider parity remain open.

M35 takes priority over unfinished M34A-11 ownership analysis. Existing C work
is preserved pending the integration decision. See the
[experiment specification](../specification/rustc-frontend-proof.md).

[Checked narrowing (02Y)](tasks/M35-03A-02Y-checked-narrowing.md) is the next
bounded scalar increment. Its independent oracle starts first; target/source
admission waits for [no-heap result support (05A)](tasks/M35-03A-05A-scalar-results.md).
This removes the circular dependency between completing all scalar operations
and later implementing the result values those operations require.
The independent narrowing oracle is complete: 74,389 exact inputs at both Rust
profiles, six executable faulty conversions, all 1,036 release/lint tests and
clean GPT-6-Sol extra-high review. No target/source admission follows from it.

No-heap result work has completed the compiler identity probe, private C
transport and [owned C public headers/ABI](tasks/M35-03A-05A-02B-01-c-result-headers.md).
The latter includes typed field-owner scopes, standalone headers, mixed GCC/Zig
objects, compiling fault controls and measured resource bounds. All 1,039
release/lint targets pass and fresh GPT-6-SOL review is clean. Existing generated
outputs and unrelated WIP are preserved.
[Certified C nominal imports](tasks/M35-03A-05A-02B-02-c-result-imports.md) are
complete too: original type/member authority, exact public signature selection,
whole-header closure, foreign read versus owned declaration permissions and
separate native producer/relay/consumer proof pass all 1,040 release/lint targets
and independent review. Existing outputs and 45 unrelated WIP files are unchanged.
[Selected-arm execution](tasks/M35-03A-05A-02C-c-result-branches.md) is the final
C transport checkpoint and is complete: native values, measured call traces,
12 value-preserving faulty runs, frame accounting, independent review and all
1,041 release/lint targets pass. Java result transport and checked
Rust Result admission remain open. This does not complete general result parity
or authorize legacy removal.

[Java synthesized components](tasks/M35-03A-05A-03A-01-java-components.md) are
complete: exact owner/role/type field and accessor references, final assignment,
privacy, closed dependency publication and JVM accounting pass all 1,041
release/lint targets with clean independent review. The Java unit suite passes
446 cases; existing outputs and 45 unrelated WIP files are unchanged. This is
the component foundation, not a complete Java Result family or Rust admission.

[Closed local Java Result families](tasks/M35-03A-05A-03A-02-java-local-families.md)
are now complete too: exact sealed variants, canonical immutable payloads,
authenticated generated upcasts, guarded observations and strict native boundary
proof pass all 1,041 release/lint targets and independent review. The Java unit
suite passes 459 cases; prior outputs and 45 unrelated WIP files are unchanged.
This closes local target transport only.

[Original Java nominal imports](tasks/M35-03A-05A-03B-java-result-imports.md)
are complete: authenticated family/type/member authority, distinct producer,
exported and consumer signatures, atomic bounded registration and strict native
producer/relay/consumer proof pass all 1,041 release/lint targets and independent
GPT-6-SOL review. The Java unit suite passes 476 cases; existing outputs and 45
unrelated WIP files are unchanged. Measured selected-arm fault proof and Rust
source admission remain separate open work, not completed migration parity.

[Java selected-arm proof](tasks/M35-03A-05A-03C-java-result-proof.md) is complete:
24,864 native observations, compiling value-preserving fault controls and actual
resource boundaries pass all 1,042 release/lint targets with clean independent
review. The ordinary Java suite passes 483 cases and five native partitions.
This closes Java target Result transport. Checked Rust Result source admission,
including canonical ownership across independent producers, remains the next
integration step; broader capability parity and legacy removal remain open.

[Canonical instance ownership](tasks/M35-03A-05A-04A-instance-ownership.md)
separates real source roots, descriptive instance facts and generated type owners.
Its [identity foundation](tasks/M35-03A-05A-04A-01-instance-model.md) is complete:
all 1,043 release/lint targets pass, two independent reviews are clean, and 530
recorded generated files are unchanged. The independently gated
[compiler graph proof](tasks/M35-03A-05A-04A-02-instance-probe.md) is complete:
all 1,047 release/lint targets pass with clean exact-tree review.
The pinned compiler revealed that TryFromIntError now carries state. A separate
[original-error-state proof](tasks/M35-03A-05A-04A-03-error-state.md) therefore
is complete: fifteen original identities, six explicit transport codes, native
reachable-state and mutation proof, all 1,051 release/lint targets and two clean
independent reviews. All 530 prior output files and 45 protected WIP files are
unchanged. Payload-free target proofs are not lossless Rust error transport.
No source admission has been opened on that assumption.
C and Java type-only certificates, atomic graph integration and checked
Result HIR bindings follow as 04B–04E; this is not yet source Result admission.

[Strict C type-owner certificates](tasks/M35-03A-05A-04B-01-c-owner-certificate.md)
are complete: immutable full-fact descriptors, exact role/file/member inventory,
structural self-header imports and measured zero executable contribution. Seven
dedicated cases include standalone strict GCC/Zig O0/O2 compilation; all 886
selected C unit cases and all 1,052 release/lint targets pass, with two clean
independent reviews. All 530 recorded outputs and 45 protected files are unchanged.
Type dependency publication remains explicitly closed until 04B-02; six-state
transport, Java type owners and compiler Result lowering remain separate gates.

The Rust-source path now has a normative [C HIR lowering specification](../specification/typed-generation/languages/c/rust-hir-lowering.md).
It reuses checked compiler HIR, documentation attributes and the existing C
AST/certification types; it does not force Rust semantics through the legacy
portable IR. M35-01A closes prototype review gaps, M35-01B integrates the
existing C types, and M35-01C preserves doc attributes before heap-owner work.

### Phase 0 — Reproducible foundation

- [M00 — Linux/Bazel development environment](milestones/M00-development-environment.md) — complete
- [M00A — Local development storage budget](tasks/M00A-development-storage-budget.md) — in progress

### Phase 1 — Semantic spine

- [M01 — Workspace boundaries](milestones/M01-workspace.md) — complete
- [M02 — Portable IR](milestones/M02-portable-ir.md) — complete
- [M03 — Diagnostics](milestones/M03-diagnostics.md) — complete
- [M04 — Resolver and checker](milestones/M04-checker.md) — complete
- [M05 — Reference evaluator](milestones/M05-evaluator.md) — complete
- [M06 — Rust builder frontend](milestones/M06-rust-builder.md) — complete

### Phase 2 — Backend platform

- [M07 — Structured document writer](milestones/M07-document-writer.md) — complete
- [M08 — Backend API and manifests](milestones/M08-backend-api.md) — complete
- [M09 — CLI and safe output](milestones/M09-cli-output.md) — complete

### Phase 3 — Required target backends

- [M10 — Rust backend](milestones/M10-rust-backend.md) — complete
- [M11 — TypeScript backend](milestones/M11-typescript-backend.md) — complete
- [M12 — Python backend](milestones/M12-python-backend.md) — complete
- [M13 — Go backend](milestones/M13-go-backend.md) — complete

### Phase 4 — Cross-language proof

- [M14 — Differential conformance](milestones/M14-conformance.md) — complete

### Phase 5 — Usability and release

- [M15 — Examples and extension proof](milestones/M15-examples-extensions.md) — complete
- [M16 — CI and release gate](milestones/M16-ci-release.md) — complete
- [M16A — Persistent GitHub Actions Bazel cache](milestones/M16A-persistent-ci-cache.md) — complete
  - [M16A-02 — Remove redundant cold release gate](tasks/M16A-02-remove-cold-gate.md) — complete
- [M16B — Native Linux CI](milestones/M16B-native-linux-ci.md) — complete

### Phase 6 — Real-world compatibility

- [M17 — escape-string-regexp equivalence port](milestones/M17-escape-string-regexp.md) — complete
- [M18 — trim-newlines equivalence port](milestones/M18-trim-newlines.md) — complete
- [M19 — slash equivalence port](milestones/M19-slash.md) — complete
- [M20 — strip-bom equivalence port](milestones/M20-strip-bom.md) — complete

### Phase 7 — Target expansion

- [M21 — JavaScript derivative and Java backend](milestones/M21-javascript-java.md) — complete
- [M22 — C++ and C backends](milestones/M22-cpp-c.md) — in progress
  - [M22A — C++20 backend checkpoint](milestones/M22A-cpp-backend.md) — complete
  - [M22B — C17 backend](milestones/M22B-c-backend.md) — in progress

### Phase 6 continuation — Real-world compatibility

- [M23 — html-escaper equivalence port](milestones/M23-html-escaper.md) — complete

### Phase 8 — Language translation architecture

- [M24 — Language package IR and dynamic imports](milestones/M24-language-package-ir.md) — complete
- [M26 — Dependency-bearing flat language IR](milestones/M26-flat-language-ir.md) — complete
- [M30 — Compositional target-language IR](milestones/M30-compositional-language-ir.md) — complete
- [M34A — Typed target-AST architecture migration](milestones/M34A-typed-target-ast.md) — in progress
  - [M34A-10 — Java typed generation](tasks/M34A-10-java.md) — complete, including the reviewed/CI-green M34A-10AB portable-expectation repair
  - [M34A-11 — C17 typed generation](tasks/M34A-11-c.md) — in progress; Java structure adapted to C-specific proof/ownership rules

### Phase 6 continuation — Real-world compatibility

- [M25 — truncate-utf8-bytes equivalence port](milestones/M25-truncate-utf8-bytes.md) — complete
- [M27 — parse-ms equivalence port](milestones/M27-parse-ms.md) — complete
- [M28 — is-fullwidth-code-point equivalence port](milestones/M28-is-fullwidth-code-point.md) — complete
- [M29 — normalize-newline equivalence port](milestones/M29-normalize-newline.md) — complete
- [M31 — has-flag equivalence port](milestones/M31-has-flag.md) — complete
- [M32 — split-on-first equivalence port](milestones/M32-split-on-first.md) — complete
- [M33 — stdlib is-negative-zero equivalence port](milestones/M33-stdlib-is-negative-zero.md) — complete
- [M34 — stdlib abs equivalence port](milestones/M34-stdlib-abs.md) — blocked by M34A after M34-02
