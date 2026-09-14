# M35-01E-03B — Executable Java source mappings

- Status: complete
- Parent: [M35-01E-03](M35-01E-03-java-hir-mappings.md)
- Depends on: completed M35-01E-03A

## Implementation contract

- Keep one file per admitted source capability in java_lower/capabilities.
  A consuming builder requires all ten executable mappings, with exact Java
  context/output types. No support flags, raw target strings or CoreIR stand-ins.
- Java-owned representation types distinguish i32, bool, nominal immutable
  records and shared references. A shared reference retains its referent plan
  even when its Java expression is the same as the referent. Place mappings are
  distinct from general values; built-in dereference must consume a shared plan.
- Read types, fields and adjustments from TypeckResults. Reject mutable borrows,
  overloaded operations, custom representations, Drop, non-scalar fields and
  unsupported HIR. Keep all source admission/traversal limits explicit.
- Register actual source functions, nominal types and fields through the shared
  origin reader. The facade is a synthesized PackageEntryPoint. Public exposure
  follows resolved exports; same spelling is not source identity.
- Map ordinary scalar signatures to JavaMethodSignature and static callable
  registrations. The selected-entry harness remains fn(i32) -> i32; ordinary
  function signatures have parameter lists, not a zero/one/two arity API.
- Evaluate calls and record fields once, in source order, using typed final
  temporaries before arranging constructor arguments by compiler field index.
- Map all six scalar comparisons. Boolean ordering uses typed conditional int
  operands (false = 0, true = 1); Boolean equality stays Boolean equality.
- Preserve lexical bindings by compiler HirId and fresh target names. Terminal
  nested blocks may flatten into the enclosing Java terminal statement sequence
  only while preserving evaluation order and preventing name/scope leakage.
- Assemble the existing TargetAstPackage<JavaDialect>, then require ordinary
  verification, linking, post-link checks and certification before rendering.

## Definition of done and tests

- Compiler-backed target AST probes cover every mapping and inspect exact types,
  source identities, borrow plans, field order, target names, scopes and exports.
- All ten slots have positive registration and missing/duplicate/wrong-capability,
  wrong-context/output compile-negative coverage; failures are checked by class
  and count, not just any failed compiler invocation.
- Same-source Rust/C/Java native tests include boundary scalars, every Boolean
  pair for every comparison, source-order calls and reordered record fields.
- Invalid Rust and valid unsupported Rust reject without changing artifacts.
- Full Linux Bazel gate and fresh broad independent review pass. No historical
  tests are disabled and no generated files are committed.

## Scope boundary

This proves single-crate lowering. Certified foreign calls and whole-crate bundle
publication remain M35-01E-04; the complete native migration closure is M35-01E-05.

## Verification history and completion

- The production adapter builds with pinned rustc/Clippy and uses the shared
  closed Configuration and declared-input checks before HIR lowering.
- Seven fixtures (model, alternate, scopes, mapping inventory, documentation,
  direct calls, Boolean ordering) each agree with native Rust and C O0/O2 over
  8,204 inputs and compile with Java 21 `-Xlint:all -Werror`. Boolean ordering
  additionally checks all 24 operator/pair cases against an independent truth table.
- Sixty compile-negative cases cover all ten slots for absent, duplicate,
  wrong-capability, wrong-context, wrong-output and wrong-input registrations/use.
- AST probe `a399f2f2-00f0-4370-a700-6f2601a2c722` passed, comparing unchanged
  production/probe output for eight fixtures. Read-only observations check actual
  compiler docs/identity/visibility, nested shared plans, nominal fields, field
  evaluation order and distinct scoped locals. Separate Java consumers execute
  public scalar exports (including four-parameter calls) and reject private helpers
  and record construction.
- Full gate `1ec24e0d-4ce9-496d-bf5d-b55f29f8c4b3` passed all 361 tests,
  including the ten additional call-admission negatives. Independent review then
  identified an output basename defect, an unbounded alias scan and structural
  proof gaps not observable through pure-call result comparison. Repairs require
  another focused/full gate and fresh review before completion (recorded below).
- Subsequent review closed the Java facade's implicit public constructor with
  an explicit private constructor and structural/native access negatives.
- Final resource review identified uncharged nested module/trait/impl/foreign
  item-reference lists. All four hooks now charge the same shared visitor budget
  without enabling duplicate recursive item visits. One-over production tests
  require the specific nested-reference failure for module/trait/impl lists;
  foreign extern blocks instead prove the earlier pinned rustc unsafe-code
  rejection. The production compiler configuration is not weakened for tests.

### Final exit evidence

- Full gate `ca26ff64-0fc0-4665-8f3e-e2094a0b8947` passed all 363 tests after
  the nested-reference repair. Fresh Sol Extra High reviewer
  `java_hir_postbudget_review` returned no findings on that fixed snapshot.
- Full gate `6d9d75bc-7588-4b40-8a5a-5a149c2e4f9b` also passed 363/363 after
  independent E04A fixture preparation and plan/spec updates. The Java suite
  passed 250/250, with zero ignored or filtered tests.
- An actual Bazel model artifact is copied to ignored
  `generated/m35-java-preview/Generated.java`; SHA-256 matches the tested output:
  `583b0bb35e47492bce380e2b17f9261d715a6b7791fe092c0105b7eb0163981d`.
- No historical gate was disabled, no generated output is staged for commit,
  and pushes remain held for the complete C/Java migration/committed-tree gate.
