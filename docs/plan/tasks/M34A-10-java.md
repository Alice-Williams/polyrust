# M34A-10 — Migrate Java 21 to typed generation

- Status: complete
- Depends on: M34A-09

The original implementation checkpoint was pushed as
`f4d9e1d539064ed70eb3c012537b99535ed344a0`. Independent review found gaps in
its claimed exit evidence. Those gaps and subsequent review findings are now
resolved in [M34A-10R](M34A-10R-java-review-remediation.md).
Java implementation is complete and ready for the user's design review; this
does not record user approval or complete the other language migrations.

## Goal

Prove the complete architecture first in Java and eliminate the reported
opaque-runtime/manual-import design.

## Definition of done

- Java owns the complete `JavaType`, expression, statement, declaration,
  member, modifier, heritage, compilation-unit, package, and grammar/format
  enums in its language specification.
- Every CoreIR feature has an exhaustive typed Java strategy and every known
  JDK type/method/field/constructor has one authoritative signature catalogue.
- Primitive/boxed generic contexts, evaluation order, sealed/tagged values,
  Unicode, checked arithmetic, exact F64 bits, immutable bytes/lists, and
  failures are verifier checked.
- Flat interfaces, multiple conformance, first-class/nested interface values,
  dynamic dispatch, and explicit final-field delegation pass.
- Portable inheritance and generated chains/reuse are absent. Java does not
  currently expose an external-framework adapter; the shared one-edge exception
  is optional and would require its own typed implementation and evidence.
- The resolver derives every package/import/qualification/helper/file from
  typed references, including all former `Runtime.java` dependencies.
- Runtime declarations are Java AST and render through the ADR-0005
  render-ready certificate and total Java structural renderer.
- `JavaCode`, raw executable documents, `RUNTIME`, `require_java`, hard-coded
  import strings, and the legacy Java pipeline are deleted.
- The Java compliance row moves to **Pass** with exact source/test evidence.

## Tests

- `bazel test //crates/backend-java:all --test_output=errors`
- Full tracked-rule and release gates retain Bazel action/test caching under
  M16A. Query rule targets rather than relying on an ambiguous `:all` suite.
- Java AST/verifier/catalogue/import/helper/certificate/total-renderer positive
  and negative matrices.
- Hermetic Java 21 lint-as-error compile, native/conformance tests, separate
  public consumer, invalid type fixtures, interface corpus, and three-generation
  determinism.
- All M17-M33 Java historical port targets.

## Commit gate

Commit and push `M34A-10: migrate Java to typed AST` only when all Java and
shared typed-generation gates pass in the dev container.

## Current integration checkpoint (2026-09-08)

### Latest repair checkpoint

The current implementation is pushed as
`708c37bcda25c8e518eb241ee89bd13b19075226`. Its local proof passes all 435
tracked rules / 310 test targets, all 247 release tests, deterministic
evaluator/eight-target conformance (50 cases and one portable test), and Linux
Cargo 1.98 compatibility (205 Java tests plus eight doctests), followed by a
complete workspace/all-features/locked Cargo test and doctest replay. Normal action
and test caches remain enabled. All Java production Rust files are below 500
lines; the largest is 477.

The target-resource follow-up now includes compiler-generated enum/record
descriptors and synthetic enum-switch class names. Native evidence loads
in-memory class files at exact encoding boundaries and requires an actual
separately compiled enum-switch helper before comparing its class metrics.
The exact red/green resource evidence is in
[M34A-10AA](M34A-10AA-java-target-limits.md). The final known-pattern,
array-allocation/rendering, negative-proof, catalogue and value-boundary repairs are documented
with full gate identifiers in [M34A-10R](M34A-10R-java-review-remediation.md).

Hosted CI for this SHA is
[run 34204771444](https://github.com/Alice-Williams/polyrust/actions/runs/34204771444).
All eight jobs pass for this exact SHA. A fresh uncapped Sol Extra High review
of the same immutable implementation reports no remaining demonstrated core
correctness or specification errors. Root independently evaluated its report;
the only organization suggestion is optional, with disposition in M34A-10R.
This closes M34A-10 and its Java follow-ups. Earlier checkpoint statuses below
are historical and do not override this completed integration record.

Current exact proof:

| Gate | Result | Invocation |
| --- | --- | --- |
| Focused Java and linters/docs | 205 Java tests; all five targets pass | `63102c96-0faa-4a6f-8332-ab854e0619e2` |
| Complete tracked rule graph | 435 rules build; 310/310 tests pass | `c36492bb-cc90-4383-8139-bc7d68a0cbd4` |
| Explicit release suite | 247/247 tests pass | `f9ed3814-fcdf-4b72-9ae8-7cb498fc1e7a` |
| Deterministic conformance | Evaluator/eight-target agreement; byte-identical repeated manifests | `e5fa64ce-f017-4b2f-88b2-4f03dbba658a` |
| Supplementary Cargo 1.98 | Whole workspace/all features/locked, all doctests pass | Linux development container |

Only the untouched untracked stdlib-abs package is excluded. This does not
change CI's full tracked-checkout graph. Capacity diagnostics retain the
documented exact JVM limits and conservative pinned-javac budgets; this proof
does not claim that finite tests establish a theorem for future compilers.

### Earlier integration checkpoint

Implementation and review repairs are pushed at
`fb3803ee0ac4e224b0caf4000fe3f1dd9d5ad293`. Hosted CI
[run 34178072602](https://github.com/Alice-Williams/polyrust/actions/runs/34178072602)
passes all eight jobs for that exact checkpoint, and the remote main ref matches.
The fresh review subsequently found missing Java literal and target-resource
boundaries, tracked in [M34A-10AA](M34A-10AA-java-target-limits.md). Java remains
open until their repairs, full proofs, and another fresh review complete. The
older evidence below records its exact checkpoint, not the new working tree.

### Architecture delivered

- All 42 capabilities have executable typed Java mappings and capability-owned
  plans. The builder automatically stores checked wrappers; plan selection,
  actual mapping invocation, and owned-output verification cannot be bypassed
  by a safe registered handler. Admission uses those same slots.
- Typed naming handles target collisions by identity. Interfaces with no
  implementations use the documented private uninhabited subtype; ordinary
  interfaces, multiple conformance, dispatch, and composition remain covered.
- AST, catalogue, lowering, runtime, and rendering have responsibility-focused
  modules. `preflight/` is distinct from `capabilities/`; production imports
  are explicit. Test fixtures are excluded from the production Bazel source set.
- Typed references drive linking/imports/helpers. Runtime and user AST share
  verification, linking, certification, and the total structural renderer;
  there are no Java executable templates or new production dependencies.

### Verified repair checkpoint proof

All executions use the Linux development container and pinned toolchains, with
normal Bazel action/test caching enabled:

| Gate | Result | Invocation |
| --- | --- | --- |
| Focused Java, strict lint, and layout | 32/32 tests pass | `88e0739b-b709-41be-a764-2107ab0f2b4e` |
| Complete tracked rule graph | 435 analyzed rules, 310/310 tests pass | `4255b202-5803-4725-b83f-5b414cd6ea5b` |
| Explicit release suite | 247/247 tests pass | `40bed654-9d0f-4e89-ace1-2a161ed7275d` |
| Deterministic conformance | 50 cases + one portable test; evaluator and eight targets agree | `e34767bc-e138-4da4-b7ee-3d3b9a01e194` |
| Supplementary Cargo 1.98 compatibility | 159 Java unit tests + eight doctests pass | Linux, all features, locked dependencies |

The tracked graph includes historical Java ports, strict Java 21 native/public
consumer and negative compilation, mutation/compiler oracles, snapshots,
Rustfmt, Clippy, Buildifier, and architecture policies. The only local graph
exclusion is the untouched, user-owned untracked `examples/real-world/stdlib-abs/`
package. The Bazel compiler oracle actually runs in the full gate; Cargo's
159-test result alone is not claimed as execution of the Bazel-only mutation
oracle. Its separate native consumer checks do run under Cargo.

### Review and proof boundaries

The integration review found and repaired 20 production wildcard imports and
boxed-number casts that incorrectly permitted unboxing followed by narrowing.
The cast matrix has 36 independently expected pairs; all 15 accepted pairs
also pass certified generation and real Java 21 compilation. Native negative
fixtures retain representative rejected casts. See
[M34A-10R](M34A-10R-java-review-remediation.md) and
[M34A-10Y](M34A-10Y-java-strategy-certificates.md) for finding dispositions.

The uncapped immutable `edada37` review concluded with exactly those two core
defects and no additional demonstrated blocker. Its proposed void-return hole
was withdrawn because expression verification already rejects void values.
Standalone void calls, Object-to-array casts, and additional pattern-flow forms
are conservative exclusions outside current lowering, not required extensions.
Generalizing the deliberately-invalid test node remains optional. Its verifier
now enforces the exact `int = String literal` negative shape: accepting other
assignments merely because a narrower invocation relation rejects them was a
core native-proof defect, fixed in M34A-10R. These observations do not excuse any accepted
production AST that fails the promised native validity checks.

The documentation audit also corrects historical naming/heritage claims:
portable `clone` methods receive safe Java names, while raw target `clone`
methods remain rejected; class-extension adapters are not implemented by the
current `None`/`Interfaces` heritage enum. No extra language feature was added.

Rust proves API/category/support constraints and prevents proof-wrapper
forgery. Correct verifier, lowering, and renderer implementations remain trusted
code backed by the permanent tests and review: neither finite compiler samples
nor mapping-owned root certificates prove arbitrary functional equivalence.
This migration is not an arbitrary-Java-program generator. Additional accepted
Java syntax and external-framework adapters require separate typed extensions.

## Historical exit evidence

- Historical pre-ADR-0005 evidence: `crates/backend-java/src/ast.rs` owns the
  closed Java syntax, type-use, modifier, heritage, literal, operator, file,
  and former template model. The dialect
  catalogue and verifier are in `dialect.rs`; exhaustive CoreIR lowering is in
  `lower.rs`; structural helper declarations are in `runtime.rs`; and
  `render.rs` was a resolved-only strict Handlebars renderer; M34A-10V replaces
  that executable path before this task can complete.
- The legacy checked-in `Runtime.java`, raw Java document path, `JavaCode`,
  `RUNTIME`, `require_java`, and `serde_json` interpreter dependency are
  deleted. The typed-generation source policy now scans every production Java
  backend Rust source, while the dependency policy admits directives only in a
  certified typed import template.
- The shared linker coalesces one physical import used by multiple typed
  symbols, retains the exact symbol membership on that import, and rejects
  forged membership. Java tests lock the exact runtime import set and prove
  that generated files receive only reference-derived imports.
- The canonical interface corpus proves two flat interfaces on one immutable
  record, static and dynamic dispatch, first-class interface values in nested
  type positions, explicit final-field composition, and three-generation
  determinism. Invalid heritage, modifier, type-use, literal, operator,
  callable, constructor, and catalogue mutations fail closed.
- Hermetic Java 21 compilation uses `-Werror -Xlint:all`. Separate public
  consumer tests, native semantic tests, canonical conformance tests, and
  deliberate invalid-type compilation tests pass for both the base and
  interface packages.
- All 39 tracked historical Java generation/conformance/negative-type targets
  pass. The final uncached tracked-scope repository graph passes 288 of 288
  tests in the Linux development container; the user-owned untracked
  `examples/real-world/stdlib-abs` package is the only exclusion and was not
  modified.
