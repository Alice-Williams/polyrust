# M35-02B-02 — Authenticate straight-line owned places

- Status: complete
- Parent: [M35-02B](M35-02B-structured-owned-mapping.md)
- Depends on: M35-02B-01
- Specification: [closed linear correspondence](../../specification/typed-generation/languages/c/rust-linear-owned-correspondence.md)

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

## Verified checkpoint

- Focused Linux/Bazel gate `a762d87f-130f-4358-ad63-bbf886cfd234`: 3/3 tests
  pass, including Clippy compilation, format, runtime mutations and exact E0451.
- Evidence keeps every source binding/place pair and move location, final scalar
  read, normal-exit drop and canonical root scope. Test consumers inspect each
  retained move against the actual typed compiler assignment.
- Isolated tree `97d13100d16b03b6df2c9b7fa5c62a080c02ab45`: gate
  `20ced5ae-1b3b-46ae-93e9-a4b16701ec0d` passed 387/387 tests across 504
  targets in 51.308 seconds. Eleven tests executed; valid cache hits remained
  enabled. This includes release/native proofs, compiler experiments, C/Java
  and shared codegen contracts, Rust/Bazel lint/format and documentation.
- Fresh Sol Extra High review found no core correctness, contract or required-
  testing findings. It independently checked canonical identities, complete
  move/local/assignment inventories, ordering, typed read projection, privacy,
  non-vacuous mutations and Bazel wiring.
- Optional follow-up: mutate the pointer cast kind/target and recorded field
  types directly. Existing wrong-owner/read-producer mutations satisfy this
  checkpoint's required categories, but these additional mutations would make
  deletion of individual projection checks easier to detect. This is accepted
  as future hardening, not treated as a missing core requirement.
- All 20 preserved ownership files match their original byte hashes. The exact
  archive's Git blob bytes/executable modes were verified. The final documented
  tree and full gate are recorded in the commit message before push.
