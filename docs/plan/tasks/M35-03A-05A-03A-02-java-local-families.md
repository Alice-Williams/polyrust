# M35-03A-05A-03A-02 — Closed local Java scalar-result families

- Status: complete
- Parent: [local Java results](M35-03A-05A-03A-java-local-results.md)
- Depends on: [typed components](M35-03A-05A-03A-01-java-components.md)
- Specification: [Java scalar results](../../specification/typed-generation/languages/java/rust-scalar-results.md)

## Contract

Use the existing sealed-interface and record declarations to represent exactly
two variants: success with the synthesized primitive-int payload and a zero-field
error record. Each family has three distinct registered generated identities;
equal names or equal shapes never establish family membership. No source IDs,
legacy runtime roles, custom source snippets, null sentinel or exception-as-Err.

Keep the ordinary Java syntax certificate distinct from the bounded result-family
proof. A focused family verifier must derive its opaque proof from an immutable
certified package plus exact declaration identities. It validates the complete
permits/implements inventory, canonical constructors, exact immutable storage,
visibility and absence of extra methods/custom accessors. A generally valid Java
record with a customized accessor does not prove the required Result observation.
Retain the original certificate behind the proof; caller-supplied names or IDs
alone confer no authority. This local proof grants no dependency import handle.

Construction needs a checked variant-to-interface conversion. Existing portable
interface coercions carry CoreImplementationId and must not be fabricated for
synthesized declarations. Add a separate typed generated-interface witness path
that the verifier authenticates against actual local declarations and their
synthesized adapter origins. Keep checked-Core conformance rules unchanged. Render
a checked reference upcast, not an unchecked payload cast or helper runtime.
Preserve the expression's declared interface type even in instanceof/pattern
contexts; erasing an explicit upcast can change Java's static type analysis.

Observation uses one materialized interface value and guarded instanceof bindings
for the exact variants. Access the success payload only through its typed bound
record reference. Keep foreign Java null outside the Rust value domain and apply
the explicit public non-null boundary policy; do not silently map null to Err.

## Definition of done and tests

- Strict Java21 compilation, normal/interpreted runs, both tags, signed extrema,
  zero and ordinary values prove constructors, copies, calls, returns and fallback.
- Syntax and family certification separately reject wrong owners, variants,
  components, constructors, permits/implements, extra subtypes, mutable/unsupported
  payloads, noncanonical initialization and customized accessors. Include valid
  Java near-misses that cannot acquire the narrower family proof.
- Checked generated upcasts work in locals, calls and returns; same-layout or
  unrelated targets, changed receiver/source identity and Core-origin substitution
  reject. No weakening of existing Core interface tests.
- Unguarded pattern reads and foreign null cannot yield a successful Rust-domain
  result. Separate external consumers prove the intended private/public boundary.
- Compare emitted classfile budgets against reservations; retain capacity gates,
  output/WIP preservation, all Linux Bazel release/lint and independent broad
  GPT-6-SOL review. Commit/push only the verified checkpoint.

## Deferred

Original foreign nominal/member/constructor import authority is 03B. Exhaustive
selected-arm/evaluation-order fault matrices are 03C. Rust compiler Result
admission and multi-producer physical-owner placement are 05A-04, not this task.

## Verification history

- Focused Linux Bazel tests on tree
  `6955e250176194884e24e61e04cf002bb8f72ee2` pass, including strict separately
  compiled Java21 producer/consumer execution: 80 observations in each of normal
  and interpreted modes, null-boundary controls, five external negative consumers
  and measured budgets for all seven generated classfiles.
- The first review identified two evidence gaps, both accepted: a canonical
  source facade with a public nominal Result signature must fail dependency
  publication for that signature (not an unrelated naming/provenance failure),
  and a real checked Core interface identity must not authorize a generated
  adapter coercion. Both now have explicit regression tests and positive controls.
- Focused testing also exposed missing retained adapter registrations. The
  exact linker-derived type registrations now survive certification; projection
  identity/equality and exact/one-over metadata limits are tested. This is not
  a new public authority or general source-declaration allowlist.
- The first full gate found a forbidden production wildcard import. It was
  replaced by explicit imports, and the known-failed run was interrupted. The
  corrected full gate and preservation checks subsequently passed, as below.
- Independent broad GPT-6-SOL extra-high review of the corrected code tree
  `02cad6465ecc9eb8cd985678b2cb6f6616bd1d85` is clean, with no remaining core
  correctness or authority findings. The reviewer checked production paths,
  ordinary syntax/conformance checks, projection, publication isolation,
  resource walkers and native evidence; it did not run or attest to the full gate.

## Completion evidence

- Linux Bazel `test //... //:release_gate --jobs=2 --keep_going --lockfile_mode=off`
  passes all 1,041 test targets on corrected code tree
  `02cad6465ecc9eb8cd985678b2cb6f6616bd1d85`: 121 executed, the rest cached,
  1,159.641 seconds overall. Rust Clippy/rustfmt, buildifier and all native/source
  policy checks remain enabled; no tests were disabled.
- The Java unit target passes 459 cases in 286.63 seconds. Its four excluded
  large native cases execute in their separate Bazel targets and pass too.
  The focused component/family/inventory invocation passed 28 selected cases.
- Strict Java21 native evidence covers 80 payload/tag/fallback observations in
  both normal and interpreted modes, null boundaries, five external rejection
  cases and seven actual classfiles checked against same-AST reservations.
- All 530 prior generated file hashes, four additional character-constant
  bundles and 45 unrelated WIP file hashes are unchanged. Independent broad
  GPT-6-SOL extra-high review is clean; both earlier findings were fixed and none
  waived. No dependency publication or compiler source admission was widened.
