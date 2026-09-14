# M35-01B-02 — Executable Rust-source C capability bindings

- Status: complete (local migration gate; push held)
- Parent: [M35-01B](M35-01B-c-hir-typed-bridge.md)
- Depends on: M35-01B certified-output checkpoint (266-target gate recorded there)
- Contract: [HIR mappings](../../specification/typed-generation/languages/c/rust-hir-mappings.md)

## Goal

Move the compiler bridge's admitted mappings into focused capability-owned
modules. Registration stores executable typed mappings, not support flags or
string IDs. Keep actual rustc HIR/TypeckResults and the existing C AST.

## Definition of done

- One owner per admitted source capability: object types, literal values,
  resolved places, shared borrows, scalar comparisons, record initializers,
  lexical control and selected entry signatures.
- Each mapping declares its source-input, context and target-output categories.
  Compiler-session references cannot outlive the session; C registry identities
  remain the same authenticated references used by shared certification.
- A consuming builder registers each mapping once and its typed slots determine
  Supports. Missing slots, duplicate registration and wrong-capability mappings
  fail Rust compilation. No Boolean or capability-name string enables lowering.
- Capability modules own exact shape admission and implementation. Traversal
  only assembles/selects mapping inputs; it does not duplicate operation rules.
- Do not register complete portable Functions/Records/Interfaces capabilities:
  these are explicitly narrower Rust-source capabilities. Do not widen admission
  merely to make the refactor pass.
- Source and test files remain focused. No duplicate Rust HIR or C AST, parser,
  borrow checker, raw-code renderer or unchecked certificate route is added.

## Tests and proof

- Compiler-negative tests for incomplete/duplicate/wrong-capability registration
  and mismatched mapping input/output categories, each with a positive control.
- Compiler-backed typed AST assertions cover every admitted capability and its
  exact nominal identities, scopes, qualifiers, field order and output category.
- Existing unsupported/borrow/type/ABI/alias cases remain rejected before output.
- All four Rust/C parity fixtures, determinism, scope/no-goto checks and strict
  native compiler/sanitizer gates continue to pass through certified output.
- Rustfmt, Clippy, Buildifier, source policy, documentation and release gates pass.
- Independent Sol Extra High review; evaluate and record every core finding.

## Work sequence

1. Typed registration and scalar literal/comparison owners.
2. Type/place/borrow/record/control/signature owners and removal of duplicate
   traversal implementations; complete exact mapping inventory tests.
3. Compiler-negative registration tests, independent review and full release gate.

## Intermediate evidence

The first two owners, LiteralValues and ScalarComparisons, now have distinct
session-bound inputs and executable stored mappings. Traversal calls Supports
and has no duplicate literal/comparison implementation. The consuming builder
only exposes registration on missing slots and build when both mappings exist.
These slots are private to the compiler bridge and make no portable-catalogue
support claim. Other planned owners remain to be migrated.

Linux/Bazel invocation `2b4d6dc0-64af-4948-8ed8-174267cd8342` passed all seven
compiler integration targets after this refactor. Invocation
`6d057f28-ff55-42bb-a777-ece782cdf0a5` then passed 13 targets: the same seven,
Buildifier and five isolated compiler-negative contracts. Missing/duplicate
registration fail E0599, wrong-capability registration fails E0271, and wrong
input/output categories fail E0308. The normal adapter/parity target is the
shared positive compilation control.

### Complete owner inventory checkpoint

All eight owners now live in separate modules. Reader traversal dispatches to
their stored mappings; no duplicate operation rules remain. Capability declares
only session-bound source input. Context and target output belong to Mapping,
so the same source contract does not require a future Java plugin to use C types.

Independent Sol Extra High review found one violation of that separation:
ControlInput carried CScopeRef. Accepted and repaired: it now carries optional
HIR parent identity, resolved by the C Reader's source-scope registry. Unknown
parents and duplicate current identities fail closed; registration precedes
child lowering. Re-review found no remaining substantiated core defect across
the eight owners, slot builder and compiler-negative harness.

Linux/Bazel invocation `53108840-f5eb-402e-9465-d929f8ae6465` passed 16 targets,
including all eight negative-contract targets, the compiler-backed mapping
inventory, Rustfmt, Buildifier and four new native matrices. The missing and
duplicate cases each require eight exact E0599 diagnostics. Other tests reject
wrong capability/input/output/context and a C scope passed to generic control
input. The normal adapter is the shared positive compilation control.

The fourth source fixture checks reversed initializer field order, both pointer
const levels, explicit and implicit dereference, all six scalar comparisons,
integer boundaries, moves and nested scopes. AST assertions additionally check
unique block identities, exact parents, owning functions and local scopes.
Its deliberately explicit dereference has a single Clippy expect annotation
with a stated test reason; the expectation itself is checked, not a broad lint
suppression. Each of four certified generated programs runs 8,204 inputs against
native Rust under pinned GCC/Zig and GCC ASan/UBSan, each at O0/O2. No alternate
renderer or hand-built AST supplies these native test inputs.

The earlier invocation `7f815257-fcb4-4357-b789-9a33ef56e606` passed all 16
functional targets and failed only Buildifier list layout; that layout is fixed
in the later passing gate.

Full Linux/Bazel release integration invocation
`defdfe9a-3c06-4ba9-9521-ca1b0ea33871` passed all 280 test targets, including
the existing release gate, every compiler-frontend target, C and shared-codegen
suites. Bazel reused valid cached results; three affected tests executed.
This completes this child task, not the parent C/Java migration. No tests were
disabled and no migration push was made.

## Commit gate

Keep this as part of M35-01B integration; hold pushes until the complete C/Java
migration gate is green. No existing tests are disabled during the refactor.
