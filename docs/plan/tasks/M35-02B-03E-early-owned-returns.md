# M35-02B-03E — Authenticate early return and continuation

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03D
- Specification: [early owned returns](../../specification/typed-generation/languages/c/rust-owned-early-returns.md)

## Contract

Extend one-condition ownership correspondence to a canonical source statement
`if flag { return *first; }` followed by `*second` or `return *second` in the
containing block. All constructors and whole-owner moves precede the condition.
Both exits use the established parameter-rooted owner chains and complete MIR
path relation. The false path is a continuation of the enclosing lexical scope,
not a fabricated else block.

## Definition of done and tests

- A separate safe evidence entry admits this closed source shape and retains
  the actual condition, early-return arm and continuation exit kind/references.
  Existing explicit-if/else and linear APIs retain their exact restrictions.
- True selects the source arm; false continues in the actual enclosing scope.
  Each path authenticates its selected owner, read and all required ordered drops.
- Tail-value and explicit-return continuations, semicolon/no-semicolon early
  returns, renamed/reordered parameters, moved and unused owners pass.
- Guard/arm/continuation substitutions, wrong lexical scope, wrong return owner,
  missing or duplicate cleanup and omitted/shared-exit mistakes reject.
- Statements between the condition and final exit, branch-local ownership
  operations, additional conditions, else arms and partial moves remain explicit
  exclusions. Invalid Rust fails analysis before proof output.
- Keep path evidence distinct from whole-function evidence, exact privacy and
  non-erasure checks, nonempty inventories and all historical target/lint gates.
- Fresh independent review, full isolated exact-tree Linux/Bazel gates,
  documentation, commit and push close this increment. No target heap output.

Conditional owner moves, compiler drop-flag interpretation, owned records and
function-boundary transfers remain separate required parent work.

## Current evidence

- Focused gate `e5f20c15-3749-43a3-b496-ff4ba38ee36f` passed 4/4 tests
  in 15.440 seconds: runtime, format, exact E0451 private construction and
  E0308 early-body-to-if/else-body rejection.
- Eight positive fixtures and nine rejected/compatibility fixtures retain
  canonical exit references, actual root/arm scopes, both MIR successors and
  ordered cleanup. Invalid Rust fails with E0382/E0502 before proof output.
- Nineteen whole-MIR substitutions, eight lexical-exit claims and four isolated
  residual corruptions reject. The residual tests require a real unit write,
  duplicated cleanup bookkeeping and a shared Return. They independently reject
  an other-arm reader and an other-arm nonconstant definition without relying on
  unrelated ownership checks. Successful accounting includes only current-path
  assignments. Nonempty inventories and changed-source controls are mandatory.
- Isolated tree `456570512a700a3134677b995986867281158aa8` passed full gate
  `36f3a674-72a0-4963-ac39-ec06bbc4afb2`: 408/408 tests across 530 targets,
  37.982 seconds, fourteen tests executed and valid cached results retained.
  Historical C/Java/native, frontend and Rust/Bazel lint gates passed.
- Fresh independent Sol Extra High review found no actionable core correctness,
  contract, privacy or test findings. It confirmed canonical root/arm identities,
  path association, complete ownership/cleanup accounting, global residual
  validation with path-local charging, historical restrictions and non-vacuous
  tests. Deferred capabilities remain required parent work, not removed scope.
- Exact archive blob bytes/executable modes were verified. All twenty preserved
  unrelated ownership files matched baseline. The checkpoint commit records
  the final documentation-inclusive tree and repeat full gate. Target heap
  output remains disabled.
