# Checked Rust public scalar constants

- Status: normative design; implementation pending M35-03A-02F-02B
- Prerequisite: [local constants](rust-local-constants.md)
- Layers: [shared dependency values](certified-dependency-values.md),
  [C17](languages/c/rust-public-constants.md),
  [Java 21](languages/java/rust-public-constants.md)

## Scope and authority

Add public nongeneric module bool/i32/i64 constants to Rust-source package
generation. Retain compiler DefId, source declaration identity, normalized type,
exact evaluated value, declared/effective visibility, module ownership, export
bindings and doc attributes. Use the existing shared compiler evaluator; no
source-string interpreter or backend evaluation of Rust initializers.

Public API selection becomes a closed declaration-kind inventory: ordinary
scalar functions and admitted scalar constants, plus finite module edges.
Every public non-module export must resolve to one admitted declaration or
produce a diagnostic. Constants-only packages are valid; empty function lists
must never be indexed and no dummy function may be synthesized.

Read checked HIR only after successful compiler analysis and declared-input
checks. Existing alias-use, generic-owner, source-budget and atomic-publication
restrictions remain. Constant initializers may use the compiler's admitted
evaluation, without enabling equivalent runtime arithmetic. Wider values,
borrowed constant storage, mutable statics, public associated-constant APIs and
generic constants remain separate work.

## Executable capability mappings

Introduce PublicConstants with a private ConstantDeclarationInput constructed
from a compiler constant definition and authenticated public export selection.
Its declaration mapping operates on package registration state, not an invented
function body Reader. Each backend returns an opaque registered constant
reference carrying its type/owner, then assembles the exact definition through
existing typed target AST nodes.

Introduce PublicConstantReads with private checked use input retaining the
resolved constant DefId/HIR and type. Its executable mapping uses the exact owned
or imported constant witness; it cannot select a target symbol by name. Register
both mappings through the typed consuming capability builder with independent
missing/duplicate/wrong capability/context/output/input controls. Source inputs
contain no target AST references.

ScalarConstants continues to fold private/local constants into typed literals.
A read of an externally reachable constant must use PublicConstantReads and
its registered value reference. Register all admitted public declarations before
body lowering. A foreign constant read requires a separately certified producer
witness; rustc metadata or matching text alone cannot mint it.

Split package state from per-function analysis state. Constants-only assembly
uses the crate's checked export/origin context, never tcx.typeck(roots[0]).
Function Readers exist only while lowering actual functions. Preserve existing
function-only, selected-entry and local-constant behavior.

## Public exports, dependencies and files

Preserve one declaration identity through multiple aliases/re-exports. Existing
crate-flattened output can map multiple Rust public paths to one target symbol;
the manifest must record every finite binding exactly, not duplicate runtime
storage or expand module cycles into infinitely many paths. Cross-crate aliases
retain producer identity. Do not merge crate output boundaries.

Producer certificates prove declaration completeness, scalar type/value and
readonlyness. Consumer witnesses retain that exact producer authority. Target
registries, resolved syntax, source descriptions and manifests must agree in
both directions. Stale or mismatched compiler/target metadata fails before any
output is published. Metadata is descriptive, never a replacement certificate.

Extend versioned C/Java manifests explicitly with typed constant descriptions
and value references. Encode signed values losslessly (including JavaScript's
unsafe integer range) under an explicit schema; readers reject old/incomplete or
unsupported schemas rather than guessing. Add fixtures proving boundary values
round-trip exactly and producer-value changes invalidate dependent generation.

## Completion proof

Separate target-AST certificate/native tests from compiler integration tests.
Final proof covers constants-only and mixed packages, both bool values, i32/i64
limits, exact wide values, computed values, forward references, private same-name
constants, doc routing, aliases and a real multi-crate consumer.

Inspect actual mapping outputs and compare native Rust/C/Java behavior against
independent expected values. Mutate type, value, owner, visibility, readonlyness,
initializer, import and declaration membership. Structural inconsistencies must
reject certification; coordinated semantic changes must fail independent native
truth. C/Java native writes to exported constants must fail compilation.

Retain all local/private constant gates and legacy functionality until full
replacement evidence exists. Each ordered child task needs its own scoped
reviewed commit after the isolated full Linux/Bazel gate, including Rust/Bazel
linters, passes. No failing test may be disabled to advance this migration.
