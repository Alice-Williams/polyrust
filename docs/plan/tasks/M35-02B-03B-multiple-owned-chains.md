# M35-02B-03B — Distinguish multiple owned producer chains

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03A
- Specification: [multiple producer chains](../../specification/typed-generation/languages/c/rust-multiple-owned-chains.md)

## Contract

Extend the closed normal-path relation to multiple same-type Box<i32> owners.
Authenticate each construction through its actual scalar parameter producer,
and every later binding through its own complete move chain. Ambiguous anchors
diagnose; equal constructor/type counts cannot pair owners. Retain canonical
tail-nested scopes and associate each remaining owner with its own drop.

The first increment allows a finite list of immutable i32 parameters and requires
each construction to consume a distinct parameter identity. Reusing one parameter
for two constructions is a documented exclusion until a separately proven
evaluation-order relation authenticates those otherwise identical producers.

## Definition of done and tests

- Specify exact multi-owner source shapes and budgets before implementation.
- Parameter identities, not spelling or scalar type, anchor each constructor.
  Every Box local and move edge belongs to exactly one complete source chain.
- The final dereference identifies one live owner without losing cleanup for
  the others. Every normal-exit drop has an authenticated owner and lexical
  declaration scope; actual cleanup order is checked for the admitted path.
- Test distinct parameters with equal types, interleaved moves, shadowing,
  nested owner lifetimes, selection of different final owners and unused owners.
- Swapped constructor arguments/owners, cross-chain moves, duplicate/missing
  drops, wrong cleanup order/scope and ambiguous repeated anchors reject.
- Keep prior root-only and one-chain scope tests and privacy contracts enabled.
  Full isolated Linux/Bazel gates, fresh review, documented proof, commit and
  push are required before closure. This does not enable target heap output.

## Current evidence

- Focused Linux/Bazel gate `8ddc857c-3e47-4c80-bc5c-d41c15f8ef9e` passed
  10/10 tests in 18.518 seconds, including all linear/scope regressions,
  multi-owner fixture/mutation controls, privacy failures, Clippy and format.
- Complete parameter-anchored chains identify every Box local and move; actual
  constructor order, final read and canonical reverse-declaration cleanup order
  are checked. Canonical containment facts remain separate from each owner's
  drop obligation.
- Pinned interleaved MIR retains four unread Boolean writes. A complete typed
  visitor proves these residual values have no runtime readers before they are
  accounted for; an isolated read mutation rejects and a distinct unread
  constant substitution remains valid. Conditional cleanup stays unsupported.
- Initial isolated tree `76985cd3de5db7d6ab286fd1046c6dc6a1262c24` passed
  gate `33e9f9f8-a3af-4dbc-89e4-37576a051d12`: 394/394 tests, 513 targets.
- The first Sol Extra High review found an ignored async-drop cleanup successor.
  The complete-path matcher now requires `drop: None`; a mutation adds a
  successor to an already visited normal block and must reject. The focused
  gate `4d3178be-b6e2-4ae5-872c-aeca7af94e48` passed 4/4 tests.
- The reviewer's initial owner-local permutation concern was withdrawn after
  evaluation: consistently renaming definitions, uses, cleanup and declaration
  entries is alpha-renaming, not an ownership substitution. Parameter producers
  still authenticate the same source chains. A positive executable control
  establishes this; partial argument/move/read/drop substitutions still reject.
- Reviewed isolated tree `5ac7213bc92543a3eac6cb1ac0523ff950ee4086` passed
  gate `3a357a58-d729-491c-a5f9-83cbab5fa238`: 394/394 tests across 513
  targets in 28.669 seconds. Five tests executed; valid cached results stayed
  enabled. Historical C/Java/native proofs and Rust/Bazel lint gates passed.
- A new independent Sol Extra High reviewer found no further core issues or
  optional-extension concerns. It confirmed the async-successor regression,
  complete parameter-anchored chains, cleanup/scope correspondence, residual
  visitor, alpha-renaming control, privacy and historical target coverage.
- Exact Git archive blob bytes and executable modes were verified, and all
  twenty preserved ownership files matched their baseline. The final
  documentation-inclusive tree and gate are recorded in the checkpoint commit.
  No C/Java heap output is enabled by this completed compiler-only increment.
