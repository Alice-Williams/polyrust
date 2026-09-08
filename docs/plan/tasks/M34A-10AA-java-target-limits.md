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

## Compiler-capacity implementation and fresh exact-boundary review

Checkpoint `c77249a12ee594942cf98d1585d8fb725f6e42f9` was pushed and the
remote main ref verified. Its fresh uncapped Sol Extra High review found three
accepted accounting defects and one accepted proof gap: a missing package
prefix for cross-file generated types, unchecked implicit inner constructors,
an artificial Signature limit for nongeneric heritage, and missing native
encoding/synthesis controls. All four are addressed in the next working tree;
the two suggested optional classifier/literal matrices are not core defects.

The separate `resources/budget/` expression, statement and class modules now
implement the documented conservative pinned-javac admission policy. It covers
individual method code, pool/member capacity, initializers, records, enums,
enclosing captures and combined enum-switch helper initialization. Every switch
reserves possible helper metadata, because a default-only enum switch still
creates a javac map. The shared generic AST remains uncapped.

Permanent tests include a typed 20,000-byte-array resource failure, native
oversized-method and aggregate-class counterexamples, independent class
reservations, and a dependency-free class-file reader. The reader compares the
same certified/rendered AST's calculated pool, member, code, locals, stack,
exception, frame and bootstrap reservations against native output from the
mutation corpus and full capability/runtime/interface packages. Per-method
native comparisons use named-method envelopes (shared only by overloads);
production admission checks every reserved method individually.

The new in-memory compiler matrix covers 255/256 array dimensions, inner and
enum hidden slots, nested binary names, implicit constructor descriptors,
ordinary descriptor aggregation, and present/absent generic Signatures. It
avoids filesystem filename limits. The first run revealed that javac accepts
the over-slot enum constructor; requiring actual JVM class loading catches it.
The corrected focused matrix passes invocation
`936fa26f-5563-4e12-bebc-866821246333`.

The earlier combined gate `c7d16f40-fd3f-457e-aaba-aad563dc0aba` passed 186 of
187 Java tests plus Rustfmt, Clippy, Buildifier and documentation; its one
failure was the compiler-only oracle assumption just described. This is not
recorded as a fully green checkpoint. Full final gates and a fresh review of
the new compiler-admission policy remain required before Java completion.

The first full replay (`1ad1826f-ee23-42fc-b901-1059df58acd6`) exposed
over-rejection of two historical test harnesses, not a native compiler defect.
Instruction-specific reservations now distinguish literal/load operations,
simple statements and control flow. Capacity maxima are unchanged. Invocation
`8d8d302f-0eac-4e30-b6b6-574d18b492db` passes all 187 Java unit/native tests,
both historical model regressions, Rustfmt and Clippy. The class-file audit also
compiles the generated native/conformance harnesses and matches method names,
so an unrelated large method cannot hide an underestimated implicit member.

Hosted CI for the preceding `c77249a` checkpoint completed successfully in
[run 34188422621](https://github.com/Alice-Williams/polyrust/actions/runs/34188422621),
with all eight jobs green. The compiler-budget working tree still needs its
own complete gates, commit, push and immutable review.

## Verified compiler-budget checkpoint

- `86c88fd2-4514-43b6-8ed0-e8f06fd71e76`: all 435 tracked rules build and
  all 310 test targets pass, including historical ports and Rust/Bazel linters.
- `6a7bb6f5-f021-4b00-b37a-278574b0b58d`: all 247 release tests pass.
- `c16f6ec6-ae13-4cd2-9bb9-9cec02c34506`: 50 cases and one portable test
  agree across the evaluator and eight targets; repeated manifests are identical.
- Linux Cargo 1.98.0 compatibility passes 187 Java unit/native tests and eight
  doctests. The mutation/compiler corpus remains specifically Bazel-owned;
  Cargo runs its other native consumer and in-memory encoding controls.

These gates retain normal Bazel action/test caching. Compiler admission is now
implemented and locally integrated; Java completion still awaits independent
evaluation of the budget review and the pushed checkpoint's hosted CI.

## Final synthesis-boundary review repairs

The fresh full review and an independent root audit found two owner-bearing
implicit descriptors not covered by the class Signature limit: enum
`valueOf(String)` needs binary-name length plus 22, and record ObjectMethods
`equals` needs length plus 23. Red invocation
`3c8dfbf6-7142-42cf-ace4-236d25e61309` reproduced both missed rejections while
the paired in-memory native controls passed. Both exact checks are repaired.
The root audit also proved javac's synthetic enum-map class needs the outer
binary name plus `$1`; a typed class-budget kind now checks that reservation.

Permanent native pairs accept/reject enum names at 65,513/65,514 bytes,
record names at 65,512/65,513, and helper owners at 65,533/65,534. The oracle
compiles in memory and loads emitted classes, avoiding filesystem limits.
The suggested record-recipe off-by-one is explicitly rejected as a finding:
the native matrix accepts both 65,534 and 65,535 recipe bytes and rejects
65,536. Source literal limits must not be incorrectly applied to synthesized
metadata.

The earlier review's claim that the native corpus lacked enums was withdrawn:
the corpus already had a structural enum and switch. That alone did not prove
helper emission, because javac can optimize same-compilation enum switches.
The strengthened oracle separately recompiles a typed consumer against emitted
provider classes, requires its synthetic `$1.class`, and compares its actual
metrics with the same AST budget. The new fixture initially failed our verifier
for missing variant coverage; completing the fixture fixed that test setup,
without weakening enum verification.

Invocation `68b8d282-90af-42d6-95c7-e629a34f45d9` passes all 190 Java tests,
including the actual helper class-file check and synthesis boundary matrix.
Full gates and fresh immutable-checkpoint review remain required.

The final full review of immutable `c584aee` reports exactly these three core
synthesis defects and the helper-emission proof gap; all four are accepted and
addressed above. It found no other concrete error across the 42 mappings,
registration/certificates, AST/linker/renderer, interfaces, budgets, dependency
hygiene, module layout, snapshots or CI wiring. Its recipe and pattern-switch
descriptor hypotheses were investigated and rejected with native/compiler
evidence, not deferred as unresolved findings. The separate focused budget
review found no additional core defect. A fresh review of the repaired
checkpoint is still required.

Final local proof for this repair:

- `e3e27b4e-806e-480c-861e-f9fba4828890`: 435 tracked rules and all 310 tests
  pass, including strict Java, Rust/Bazel linters and historical ports.
- `c07929b0-7620-4ca0-ba2b-837fdda5f8ee`: all 247 release tests pass.
- `0ec22c1f-3294-4e50-86b1-9223ba828bdc`: 50 cases and one portable test
  agree across evaluator/eight targets, with deterministic repeated manifests.
- Linux Cargo 1.98 passes 190 Java tests and eight doctests; the mutation
  compiler corpus is Bazel-owned, with other native consumers also run in Cargo.
- Every Java production Rust source is now below 500 lines (largest: 499).

The preceding checkpoint `c584aeeaff0349a91462dbd7c0c6c791ea6cdab9` has
all eight hosted jobs green in
[run 34191179416](https://github.com/Alice-Williams/polyrust/actions/runs/34191179416).
This repair still needs its own push, hosted CI and fresh review before closure.
