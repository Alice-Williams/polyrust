# M35-02B-02 — Authenticate straight-line owned places

- Status: planned
- Parent: [M35-02B](M35-02B-structured-owned-mapping.md)
- Depends on: M35-02B-01

## Contract

Define a closed first body shape before implementation: one authenticated
Box<i32> construction, immutable whole-value moves, a scalar read and a normal
root-scope exit. Use canonical HIR binding identities and compiler MIR places.
Associate a unique construction through complete typed operation inventories,
then validate the complete move/drop relationships. Do not pair variables by
names, debug entries, source-span coincidences or arbitrary traversal position.

This is correspondence validation after rustc analysis, not a replacement borrow
checker. Ambiguous, additional or unmatched owned operations must diagnose.
Keep structured HIR as the later rendering input; do not generate MIR gotos.

## Definition of done and tests

- Specify exact permitted body/parameter/initializer/return shapes and limits.
- Private compiler-session evidence binds each accepted source binding to its
  authenticated place, every move to its source/destination, and the final owned
  place to the relevant normal-exit drop obligation.
- Demonstrate scalar argument identity and construction-result identity, not
  merely matching constructor/type counts. Account for all relevant owned
  operations and drop edges; diagnose ambiguity, extras and unsupported shapes.
- Positive tests cover renamed/shadowed bindings and multiple move-chain lengths.
  Negative/mutation tests swap owners/places, omit or duplicate correspondence,
  change argument identities and insert extra ownership/control-flow operations.
- Prove no fabricated evidence through safe exposed constructors; retain exact
  typed registration and context/input/output compile-negative contracts.
- Full isolated Linux/Bazel gates, fresh review, documented limitations and a
  verified commit/push pass. No C/Java heap output is enabled at this checkpoint.
