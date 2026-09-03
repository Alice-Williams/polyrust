# Typed-generation compliance ledger

- Status: normative baseline for M34A
- Baseline date: 2026-09-02
- Last updated: 2026-09-03
- Supersedes for new work: `docs/language-ir-compliance.md`

This ledger measures the stronger ADR-0004/ADR-0005/ADR-0006 contract. The M30 ledger remains
historical evidence that the existing fragment paths carry dependency metadata;
it is not evidence of typed executable syntax.

## Current audit

| Surface | CoreIR input | Typed executable AST | Typed symbols and derived dependencies | Structural runtime | Render-ready certificate and total renderer | Result |
| --- | --- | --- | --- | --- | --- | --- |
| Shared codegen | `CoreProgram` is the only lowering input | Generic checked target AST | Catalogue/linker-derived bindings, imports, files, helpers, and packages | Typed helper DAG and structural items | Opaque verify/link/certify states, sealed certified adapter, and total structural-renderer extension point | **Pass** |
| Inferred typed AST | `TypedProgram<R>` and consuming builder specified; profile implementation is being replaced | Private `Expr<T, R>` plus invariant body/record handles and recursive typed lists required | Constant-checked names plus inferred requirements and `SupportsAll<R>` | Not applicable at this layer | Admitted target adapters are total; rejection is an implementation defect | **Partial** |
| Rust | Missing | Missing: `RustCode`/raw documents | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| TypeScript | Missing | Missing: paired `EcmaCode` | Manually attached fragment metadata | Raw runtime source | Paired source path | **Fail** |
| JavaScript | Missing | Independently paired source exists | Shares manual ECMA metadata | Checked-in/runtime paired text | Not solely compiler-derived | **Fail** |
| Python | Missing | Missing: generated source fragments | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| Go | Missing | Missing: generated source fragments | Manually attached fragment metadata | Raw runtime source | Documents are directly rendered | **Fail** |
| Java | Exhaustive verified `CoreProgram` lowering | Closed Java 21 AST and pre/post-link verification | Closed catalogues, typed references, and linker-derived imports | Structural helpers recomposed and rechecked with user declarations | Opaque render-ready certificate and direct total structural renderer; local and hosted gates green | **Partial** pending user review |
| C++20 | Missing | Missing: `CppCode`/raw documents | Manually attached fragment metadata | Included/sectioned runtime source | Documents are directly rendered | **Fail** |
| C17 | Missing | Missing: `CCode`/raw documents | Manually attached fragment metadata | Included/sectioned runtime source | Documents are directly rendered | **Fail** |

## Evidence locations

- Shared typed pipeline: `crates/core-ir/src`,
  `crates/codegen/src/target_ast.rs`, `crates/codegen/src/linking.rs`,
  `crates/codegen/src/rendering.rs`, `crates/codegen/src/typed_pipeline.rs`,
  `crates/codegen/src/manifest.rs`, and `crates/codegen/src/compliance.rs`.
- Rust: `crates/backend-rust/src/v0.rs`.
- TypeScript/JavaScript: `crates/backend-typescript/src/lib.rs` and
  `crates/backend-typescript/src/runtime.ts`.
- Python: `crates/backend-python/src/lib.rs` and
  `crates/backend-python/src/runtime.py`.
- Go: `crates/backend-go/src/v0.rs` and
  `crates/backend-go/src/runtime.go`.
- Java: `crates/backend-java/src/ast.rs`,
  `crates/backend-java/src/dialect.rs`, `crates/backend-java/src/lower.rs`,
  `crates/backend-java/src/runtime.rs`, `crates/backend-java/src/render.rs`, and
  `crates/backend-java/src/lib.rs`.
- C++20: `crates/backend-cpp/src/lib.rs` and
  `crates/backend-cpp/src/runtime.hpp`.
- C17: `crates/backend-c/src/lib.rs`,
  `crates/backend-c/src/generator.rs`, `crates/backend-c/src/runtime.h`, and
  `crates/backend-c/src/runtime.c`.

The locations for rows which still fail are migration inputs, not allowlisted
final architecture. Shared codegen implements the ADR-0005 target foundation,
and ADR-0007 supersedes the coarse ADR-0006 profile with inferred per-feature
requirements. Java's previously implemented profile adapter remains migration
evidence, but it does not make the new inferred path compliant.

## Shared M34A-08S evidence

- `StaticProgram<F>` and every expression, local, function, record, field, and
  type witness have private representations. Generative invariant lifetimes
  prevent cross-body locals and cross-record fields from unifying.
- Ten compile-fail cases cover type/operator, return, call, constructor, field,
  protected-name, and proof-forgery failures. Every `StaticV1` constructor
  compile-passes and replays through the checker, CoreIR lowerer, and verifier.
- Implementation checkpoint `389f0cb` and evidence checkpoint `81248be` are
  pushed. The tracked Bazel universe passed 297/297 tests, the release suite
  passed 233/233, and deterministic conformance agreed across eight targets.

## Shared M34A-08R evidence

- The compiler adapter now consumes
  `UnresolvedPackage -> VerifiedPackage -> LinkedPackage ->
  RenderReadyPackage -> RenderedPackage` in order. All proof-wrapper fields and
  constructors are private, none is deserializable, and only immutable AST
  views are public.
- Compile-fail doctests reject unverified linking; unresolved, verified, and
  linked rendering; proof construction; checked-AST mutation; certificate
  deserialization; and direct construction of a certified source-file view.
- `TargetRenderer` is sealed. Third-party crates implement
  `TotalSourceRenderer<D>` and obtain the sealed implementation only through
  `CertifiedStructuralRendererAdapter`; the external-backend example executes
  that complete sequence from a separate crate.
- The legacy Handlebars engine remains only for explicitly unmigrated
  backends. It is not reachable from Java or from the new certified adapter and
  is scheduled for deletion by M34A-18.
- Local Linux-container proof passed 36/36 expanded shared/Java/policy/lint
  targets, 295/295 tracked repository targets, 233/233 independently composed
  release-gate targets, the fresh Cargo workspace/all-features/locked suite
  including 31 codegen compile-fail doctests, and the focused external-plugin
  gate 5/5.

## Java M34A-10 evidence

The evidence below passed at `e03a633`, but a fresh blind review demonstrated
remaining semantic, boundary, preflight, and verifier defects. It is retained as
test evidence rather than a current **Pass** claim while M34A-10R is open.

- The Java backend has no production opaque executable source node, checked-in
  runtime source, manual import API, or direct manifest construction path.
- Exact import-set tests include the former runtime dependencies and prove that
  repeated type/constructor references produce one physical import. Shared
  linker fault injection rejects forged import membership.
- Hermetic Java 21 `-Werror -Xlint:all` positive, public-consumer,
  conformance, interface/composition, and deliberate negative-type tests pass.
- All 39 tracked historical Java targets pass. The uncached tracked-scope
  repository gate passes 288 of 288 tests with Rustfmt, Clippy, Buildifier,
  typed-source policy, template policy, dependency policy, native compilers,
  sanitizers, and differential conformance included.
- Round 2 remediation after that review adds recursive semantic equality,
  `byte[]` value ownership, exact shape preflight, feature/mode/strategy
  selection validation, sealed closed interface values, type-directed deep
  public-boundary reconstruction, and the completed AST verifier checks. Its
  focused Java/codegen/policy gate passes 28 of 28 targets, its complete
  tracked repository replay passes 291 of 291 targets uncached, and its
  independent release-gate replay passes 230 of 230 targets uncached. The row
  remains **Fail** until a pushed immutable checkpoint receives a clean fresh
  blind review and closes M34A-10R.
- Fresh blind review round 3 of pushed checkpoint `a25a15f` confirmed four
  remaining blockers: checked-remainder overflow parity, direct generic tagged
  value aliasing, fail-open Java AST verifier shapes, and an empty generated
  conformance entry point. These accepted findings keep Java at **Fail** until
  their regression evidence and another fresh review are complete.
- Fresh blind review round 4 of pushed checkpoint `4bc84d36` found four further
  Java blockers in scalar-safe empty-needle replacement, inherited `Object`
  method shape preflight, privileged literal placement, and Java statement/
  switch grammar. All are accepted, so Java remains **Fail** pending round 4
  remediation, replayed gates, and another fresh blind review.
- Fresh blind review round 5 of pushed checkpoint `bbf91d2b` validated the
  round 4 repairs but found four further blockers: public implementation helper
  callables bypass scalar-string normalization, non-local lexical bindings may
  redeclare enclosing names, catch clauses lack subtype-dominance checks, and
  constructors lack blank-final definite-assignment proof. All are accepted;
  Java remains **Fail** pending round 5 remediation and a clean fresh review.
- Fresh blind review round 6 of pushed checkpoint `b528489d` validated round
  5, but found two further fail-open verifier paths: reads of unassigned locals
  and blank finals, and `instanceof` binder collisions outside selected
  control-flow conditions. Both are accepted core blockers; Java remains
  **Fail** pending round 6 remediation and another clean fresh review.
- Fresh blind review round 7 of pushed checkpoint `3916e3e8` validated round
  6, but found two further fail-open verifier paths: field initializers bypass
  lexical/blank-final/checked-exception preflight, and statements after an
  unconditional exit remain accepted. Both are accepted core blockers; Java
  remains **Fail** pending round 7 remediation and another clean fresh review.
- Round 7 checkpoint `4a72777afd82eea847c827e32703898efe5d2611`
  was pushed and remotely verified. Hosted CI run `33732935834` passed every
  job for that exact commit.
- Fresh blind review round 8 validated round 7, but found three further core
  blockers: name-sorted Java constant fields can create illegal forward
  references, loop completion ignored Java constant-condition and current-loop
  break rules, and composed expressions could duplicate an operand or let a
  later fallible child overtake an earlier allocation/receiver. All three are
  accepted. Java remains **Fail** pending round 8 regression evidence, complete
  gates, a pushed checkpoint, hosted CI, and another clean fresh review.
- Round 8 regression evidence passes: 36 of 36 uncached focused
  Java/codegen/policy/snapshot/lint tests, 294 of 294 uncached tracked-repository
  tests, 232 of 232 independently composed release-gate tests, and a fresh
  Cargo workspace/all-features/locked target including doctests. Java remains
  **Fail** until the immutable pushed checkpoint receives successful hosted CI
  and a clean fresh blind review.
- Fresh blind review round 9 of pushed checkpoint `e5bfefa` validated the
  round 8 fixes but found three more core gaps: advertised portable `Evaluate`
  could become a verifier-forbidden Java expression statement, an empty
  `while (false) {}` was accepted despite Java 21 rejecting its unreachable
  body statement, and a public top-level type was not required to match its
  source filename. All are accepted. Java remains **Fail** pending round 9
  regression evidence, gates, push, hosted CI, and another fresh blind review.
- Round 9 regression evidence passes: 37 of 37 uncached focused
  Java/codegen/policy/snapshot/lint tests, 294 of 294 uncached tracked-repository
  tests, 232 of 232 independently composed release-gate tests, and a fresh
  Cargo workspace/all-features/locked target including doctests. Java remains
  **Fail** until the immutable pushed checkpoint receives successful hosted CI
  and a clean fresh blind review.
- Round 9 checkpoint `597031d` was pushed and remotely verified; hosted CI run
  `33742877916` passed every job for that exact SHA. Fresh blind review round 10
  validated the round 9 fixes but found one core fail-open path: runtime helper
  fragments were not reverified as their final combined `Runtime` class. The
  accepted round 10 remediation adds a shared post-link whole-file verifier,
  Java declaration recomposition, fragment/shell placement checks, and forged
  AST plus Java 21 compiler proof. Its focused gate passed 37 of 37 tests, the
  complete tracked-repository replay passed 294 of 294 tests, the independent
  release gate passed 232 of 232 tests, and a clean Cargo workspace replay
  passed with all doctests. Java remains **Fail** pending round 10 push, hosted
  CI, and another clean fresh review.
- Round 10 checkpoint `1dbbdbd` was pushed and remotely verified; hosted CI run
  `33746746728` passed every job for that exact SHA. Fresh blind
  review round 11 validated the composition repair but found three more core
  fail-open paths: runtime shell/provenance identity was not exact, method
  annotations lacked verifier rules, and blank static finals were accepted
  without a representable initializer path. All are accepted. Round 11 now
  uses one canonical typed runtime shell, confines fragments to linker-selected
  helpers, validates annotation uniqueness/context exhaustively, rejects blank
  static finals, and carries forged-AST plus Java 21 compiler regressions. Its
  focused gate passed 37 of 37 tests, the complete tracked-repository replay
  passed 294 of 294 tests, the independent release gate passed 232 of 232
  tests, and a clean Cargo workspace replay passed with all doctests. Java
  remains **Fail** pending round 11 push, hosted CI, and another clean fresh
  review.
- Round 11 checkpoint `385565e` was pushed and remotely verified; hosted CI run
  `33750373016` passed every job for that exact SHA. Fresh blind
  review round 12 found three accepted fail-open paths: one-way reserved-runtime
  identity, weak-access interface implementations, and coarse generic callable
  signatures. Round 12 makes runtime identity bidirectional, makes constructed
  Java signature identity exact, and uses one public concrete implementation
  predicate for registration, conformance, and `@Override`. Forged AST and
  hermetic JDK counterexamples are permanent. A deterministic 128-case AST
  mutation/compiler-oracle corpus now requires every verifier-accepted
  declaration to link, strictly render, and compile with all Java 21 lint
  warnings as errors. Its focused gate passed 37 of 37 tests, the complete
  tracked-repository replay passed 294 of 294 tests, the independent release
  gate passed 232 of 232 tests, and a clean Cargo workspace replay passed with
  all doctests. Java remains **Fail** pending round 12 push, hosted CI, and an
  exhaustive clean review.
- Round 12 checkpoint `5cb5a10552f3d1b4f96d974eeee91ec67042c486`
  was pushed and remotely verified; hosted CI run `33755866121` passed all
  eight jobs for that exact SHA. An uncapped nine-category Sol/xhigh audit then
  validated the generated packages but found ten accepted defect families in
  source-policy scanning, recursive boundary ownership, callable-position
  matching, wildcard/cast grammar, record and inherited-`Object` rules, nested
  names, canonical Java paths, and oracle coverage claims.
- Round 13 fixes all ten families and adds exact verifier regressions plus eight
  paired hermetic Java 21 compiler-negative fixtures. Its compiler oracle is
  broadened across structured declaration/expression/file categories and is
  documented as deterministic sampling rather than universal proof. The first
  repository replay also exposed and permanently covered the valid generic
  boxed-result-to-matching-primitive unboxing used by `has-flag`.
- Round 13 passes 53 of 53 focused Java/codegen/policy/snapshot/lint tests, 294
  of 294 uncached tracked-repository tests, 232 of 232 uncached independently
  composed release-gate tests, and a fresh isolated
  `cargo test --workspace --all-features --locked` run including every doctest.
  Java remains **Fail** pending an immutable pushed checkpoint, successful
  hosted CI for that SHA, and a clean fresh exhaustive review.
- Round 13 checkpoint `c1a17ee10faf0eceddc7f63f8430f33b22e711d9`
  was pushed and remotely verified; hosted CI run `33764587519` passed all
  eight jobs for that exact SHA. A different uncapped nine-category Sol/xhigh
  review confirmed the prior fixes but found five accepted blockers in array
  ownership-changing casts, unchecked parameterized casts, source-policy
  marker recognition, registered inherited-`Object` collisions, and generated
  interface method completeness.
- Round 14 remediates those five blockers and the related redundant-cast lint
  case discovered by the compiler oracle. Permanent proof adds forged-AST
  regressions, four exact hermetic Java 21 compiler-negative fixtures, a
  positive array-aliasing witness, and Rust literal/comment policy decoys. Its
  initial verifier/compiler/policy and Rustfmt/Clippy/Buildifier gate passes 7
  of 7 targets. Its complete focused gate passes 54 of 54 tests, the uncached
  tracked-repository replay passes 295 of 295 tests, the independently composed
  release gate passes 233 of 233 tests, and a fresh isolated Cargo workspace
  replay passes with every doctest. Java remains **Fail** pending an immutable
  pushed checkpoint, successful hosted CI, and another clean exhaustive review.
- Round 14 checkpoint `71b5ecd` was pushed and remotely verified. M34A-10V then
  removes all Java executable Handlebars templates and serialized render views,
  renames the file discriminator to `JavaSourceFileKind`, and emits every Java
  construct directly through exhaustive structural Rust matches. Java reaches
  that renderer only through the shared opaque post-link certificate.
  Generated Java snapshots remain byte-identical. Local M34A-10V proof passes
  the 36/36 expanded gate, 295/295 tracked repository replay, 233/233 release
  gate, fresh Cargo workspace/all-features/locked suite, and external-plugin
  adapter proof. Implementation checkpoint
  `48fecbfaecf67b089085f2ea28203d562ca5fe68` was pushed and remotely
  verified; hosted CI run `33790836392` passed all eight jobs for that exact
  SHA, including cache-cold and cache-warm complete release gates. The row
  remains **Partial** pending the user's requested review.

## Pass rule

A row moves to **Pass** only in the same checkpoint which:

1. satisfies all nine shared layer specifications;
2. satisfies every section of the corresponding language specification;
3. deletes the language's legacy raw/paired executable path;
4. adds permanent positive, negative, compile-fail, and source-policy proof;
5. passes its native formatter, linter/static analysis, compiler, and tests;
6. passes three-generation determinism and canonical semantic conformance; and
7. replays every historical real-world port for that language.

JavaScript passes only with TypeScript and only after clean pinned compiler
derivation proves every executable JavaScript byte.

## Release rule

M34A is complete only when the shared row and all eight language rows are
**Pass**, the old M30 ledger is clearly historical, and uncached repository,
release, and hosted CI gates are green.
