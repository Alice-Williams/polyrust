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
   cap is not permitted. The choice between checked target-resource errors and
   a larger argument-packing ABI transformation has been put to the user; do
   not silently introduce the latter while fixing literals.

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
