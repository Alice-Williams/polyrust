# M34A-11-00P — Seal resource admission into render readiness

- Status: in-progress
- Depends on: M34A-10AB
- Required by: M34A-11-00R before contextual C implementation

## Goal

Repair the shared direct-certification bypass found in C design review round 4.
This is a shared pipeline defect affecting existing Java, not an implemented
C AST defect. Public certify_linked_package/certify_resolved_package currently
produce RenderReadyPackage after syntax verification while only the compiler
adapter later invokes verify_resources. A direct safe caller can skip that gate.

## Definition of done

- Both public certifiers and the compiler adapter use one shared path: language
  verification, then target resource validation, then private construction of
  RenderReadyPackage. Never create a render-ready value before both succeed.
- Preserve distinct RenderReadinessCertification/TargetResourceValidation
  diagnostic stages and TargetResourceLimit codes; syntax failure runs first
  and suppresses resource validation. Resource validation executes exactly once.
- Preserve existing public function signatures where possible; no new mutable
  package access, public certificate constructor or renderer overload.
- Move cohesive certification implementation into a small module rather than
  growing the existing shared pipeline mega-file. Tests are excluded from
  production Bazel sources through the existing tests directory policy.
- Update shared Layer 8 and C/Java resource specifications. Late manifest/output
  assembly limits remain distinct and cannot be mistaken for target admission.

## Tests and proof

- Direct public certifier positive boundary and one-over resource rejection;
  both linked and raw resolved entry points, with exact checker call traces.
- Syntax-plus-resource failure proves syntax precedence, no resource/render
  call, and no render-ready value. Resource failure also prevents rendering.
- Existing compiler-stage tests prove unchanged diagnostics/order and no
  duplicate resource invocation; public wrong-phase/forged-certificate
  compile-fail controls remain passing.
- Real Java linked package at the 255-slot method boundary succeeds; 256 slots
  fails direct certification with TargetResourceLimit, not an invariant panic.
- Linux Bazel shared pipeline/compile-fail/Java resource tests, Rustfmt, Clippy,
  Buildifier, documentation and full cached tracked/release/eight-target gates.
- Fresh uncapped Sol Extra High read-only code review; evaluate every finding.

## Commit gate

Record exact invocation IDs/counts, failures and dispositions. Commit and push
M34A-11-00P independently of C contract amendments; do not close 00R or claim
C generation until its own remaining obligations are met.

## Implementation and regression evidence (2026-09-08)

Certification now lives in a focused shared module with one private constructor
and one syntax-then-resource path. Both public entry points and the compiler
adapter use that path, preserving the two diagnostic stages without a duplicate
resource call. Tests remain outside production Bazel source sets.

- Red control 6c2c83bb-124e-4bc4-ad43-39ad98cba0d4 reproduced the direct Java
  bypass: a linked 256-parameter static method acquired RenderReadyPackage.
  The positive 255-slot boundary remains accepted after the repair.
- Focused 35065894-76e6-4d22-8995-51a86cf5bb5f: all seven shared pipeline,
  compile-fail, Java, Rustfmt, Clippy, Buildifier and documentation targets pass.
- Full tracked 7bb71a7b-bd86-48d5-a4ef-7554c2828f1a: 439 rules / 314 tests pass
  (69 executed, remaining results cached normally).
- Release c2c52272-a14c-416a-b511-acf197cdbb37: all 251 tests pass.
- Conformance 10a6446a-6a30-4811-805b-33ce17bc7ddc: evaluator plus eight targets
  agree on 50 cases and one portable test; repeated manifests are byte-identical.

Fresh immutable code review and hosted confirmation remain open. C has no new
certified renderer yet; this shared repair must not be reported as C cutover.
