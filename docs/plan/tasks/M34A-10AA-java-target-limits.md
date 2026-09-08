# M34A-10AA — Java literal and target-resource boundaries

- Status: in-progress
- Depends on: M34A-10X and M34A-10Y implementation checkpoints
- Blocks: completion of M34A-10, M34A-10R, and M34A-11

## Goal

Resolve concrete Java compiler-limit counterexamples from the fresh immutable
`fb3803e` review without imposing arbitrary caps on the generic authoring API.
The shared static-AST specification already allows explicit checked target and
resource constraints; syntax certification must not overclaim compiler capacity.

## Accepted findings and implementation order

1. Portable text currently becomes one Java string literal. The real Java 21
   oracle accepts 65,534 ASCII bytes and rejects 65,535 and 65,536. Modified
   UTF-8 costs differ from UTF-8: NUL costs two bytes and a supplementary scalar
   costs six. Split on scalar boundaries and assemble a balanced typed tree of
   `String.concat` calls, which cannot be folded into one literal constant.
2. Authenticate every owned chunk, receiver, argument, known member, and exact
   signature in the `TextValues` mapping plan. Raw oversized literal ASTs fail
   verification; the renderer only spells the already-selected AST.
   `StringConcatenation` must also use a native `String.concat` call: even two
   individually valid literals, including final-local constants, can otherwise
   fold into an oversized constant. Generated metadata/runtime string builders
   share the bounded construction helper.
3. Resolve explicit Java target limits: method parameter slots, encoded names
   and descriptors, and oversized method/class resources. A new generic arity
   cap is not permitted. Follow the shared specification's checked-resource
   policy; argument packing is a separate optional ABI extension, not part of
   this repair. The concrete boundary and phased proof obligations are in
   [Java target resources](../../specification/typed-generation/languages/java/target-resources.md).

## Definition of done

- Admitted large portable text generates, compiles, and returns exact contents.
- Raw literal limits use Java's modified-UTF-8 accounting, not Rust byte length.
- Literal chunking is structural mapping work, never renderer text rewriting.
- Every accepted target-limit finding has a documented boundary and permanent
  regression; typed resource failures are distinct from syntax/capability bugs.
- Generic recursive parameter/argument/field lists remain uncapped APIs.
- No accepted unresolved core finding remains after a fresh Sol Extra High
  immutable-checkpoint review and independent evaluation of its report.

## Tests and checkpoint gate

- Exact ASCII 65,534/65,535/65,536 boundary compilation, NUL, supplementary
  scalars, mixed-width boundaries, empty text, and multi-chunk text consumers.
- Independent chunk-size/content mutations and verifier rejection of oversized
  raw literals, including supplementary and UTF-16-unit representations.
- Exact Java parameter-slot and identifier/descriptor counterexamples, with
  positive controls and stable resource diagnostics for the selected policy.
- Focused Java tests, Java 21 strict compiler/consumer oracles, Rustfmt, Clippy,
  Buildifier, full tracked-rule and release gates, deterministic eight-target
  conformance, and hosted CI. Normal Bazel action/test caching stays enabled.
- Commit and push verified remediation checkpoints citing M34A-10AA; keep Java
  open until all required resource-boundary decisions and proofs are complete.

## Reproduction evidence

- `98fd3fea-3cef-4239-af3f-0e880b1ae049`: the new typed text test fails at actual
  `javac` with `constant string too long` for 65,536 ASCII characters.
- `ee726b96-f31e-4317-babf-9bd147feb3fc`: the 65,534-byte consumer compiles and
  executes; the following 65,535-byte case fails with the same diagnostic.
- These are new regressions against the previously green `fb3803e` checkpoint,
  not evidence that the current working tree is fully verified.
- `9ccec998-db94-4d36-95bc-96b92b817b9f` reproduces the related constant-folding
  hole after the initial literal split: concatenating two 60,000-character
  strings fails actual `javac`. The permanent fixture includes direct literals
  and typed final locals. It now uses the native-member mapping rather than `+`.

The final full review identified four additional target-resource families:
parameter slots, encoded names/descriptors, per-method bytecode, and aggregate
class member/constant-pool capacity. These remain explicit open work; earlier
green gates and the literal repair do not close them.

A full wildcard-import scan also found one nested `expressions::*` import that
the earlier parent-glob scan missed. It is replaced with explicit imports, and
production Java now denies `clippy::wildcard_imports`; test-only glob scopes are
not production library dependencies.

## Verified text checkpoint (2026-09-08)

- Focused Java, Rustfmt, Clippy, Buildifier, and documentation gates pass in
  invocation `22647476-c001-4c6c-9c47-25246c013314`.
- The first full replay caught the intended interface snapshot change from
  string `+` to native `concat`. The curated snapshot was refreshed from the
  exact Bazel artifact, then its byte-for-byte regeneration check passed.
- `cd6401f9-e7e4-4522-9822-92835c06cf4e`: all 435 tracked rule targets build
  and all 310 test targets pass, with normal action/test caching enabled.
- `da60dd6e-9ead-4056-8f8b-3ec68c7bfa56`: all 247 release tests pass.
- `470155fd-4002-4116-80a7-dd4f5ac917f5`: 50 cases and one portable test agree
  across the evaluator and all eight targets; repeated manifests are identical.

This closes the reproduced portable-text construction and concatenation holes,
not the four open JVM resource families. Java remains in progress.

## Fresh text-checkpoint review

The immutable `1c0793e` Sol Extra High review confirmed portable chunking,
mapping certificates, imports and helper routing, but found two raw-AST holes:
string `Add` still allowed compiler constant folding, and switch-label literals
bypassed scalar/encoding payload checks. Both are accepted core findings.
Invocation `5c275178-cedf-40fe-b8f2-e7330c5a59d4` reproduces both verifier
failures with permanent regressions. The repair limits target `Add` to numeric
operands, constructs native `concat` calls in runtime helpers too, and shares
literal payload validation with switch labels. Java remains open pending gates.

## Exact resource-boundary implementation checkpoint

The shared compiler now invokes target capacity checking after certification in
`TargetResourceValidation`. Java returns only typed resource errors from that
phase and shared output-size errors from `Rendering`; other failures remain
invariants. This prevents capacity checks from hiding an invalid target AST.

Separate declaration, executable and encoding modules check slots, nested
binary names, descriptors/generic signatures, record recipes, array dimensions,
local bindings and referenced types. Enum/enclosing-instance constructor
parameters are included. Linked Runtime fragments share one class traversal.
Native controls confirm 255 static int slots, 254 instance int slots, 127 long
parameters, boxed widths, record limits, 65,535-byte names, and case literals.

Invocation `dd03d410-8118-4c01-b4a0-f42fcd3de965` passes Java and shared compiler
tests, Rustfmt, Clippy and documentation checks. The full checkpoint gate is
still required. Compiler-generated method/class budgets remain open; these
exact checks alone do not close M34A-10AA or Java.

An architecture review caught the initial early-phase ordering, missing body
walk and hidden constructor parameters; these were corrected before checkpoint
completion. Its test-wiring concern was withdrawn: resource tests deliberately
use an owner-local `#[path = "tests/resources.rs"]` module, and are excluded
from the production Bazel source set while included in the Rust test target.

The final linked-name repair is verified by focused invocation
`c8ff521f-8155-4506-a7d9-f061d524f519`. The exact-tree replay then passes:

- `80e4266f-9302-44e2-8f61-69795e15051e`: 435 tracked rules, 310 tests.
- `6c73aa08-a741-4f59-af80-6a500e807081`: all 247 release tests.
- `73a29a26-6b95-4a64-a268-9fc440a99b6b`: 50 cases and one portable test,
  evaluator plus eight targets agree, with byte-identical repeated manifests.

Normal Bazel caching was enabled. This checkpoint closes the exact structural
limits and raw-literal review findings, not the remaining method/class budgets.
