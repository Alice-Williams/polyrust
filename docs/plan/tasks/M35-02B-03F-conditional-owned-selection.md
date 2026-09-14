# M35-02B-03F — Authenticate conditional owner selection

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03E
- Specification: [conditional owner selection](../../specification/typed-generation/languages/c/rust-owned-selection.md)

## Contract

Authenticate a root-scope binding `let selected = if flag { first } else {
second };`, followed by a scalar dereference of that selected owner. Constructors
and ordinary whole-owner moves precede the selection. Each source arm moves a
different existing owner into the same destination binding. The nonselected
owner remains live in its original scope. Preserve canonical source identities
and the distinct binding-to-place relation on each path.

This is the first increment that interprets actual compiler drop-flag branches.
Derive their Boolean values from executed typed constant definitions, then check
the resulting cleanup against the selected source owner chains. Do not infer
ownership from the spelling, local number or Boolean type of a flag.

## Definition of done and tests

- Inspect the pinned compiler representation before finalizing admission.
  A typed source selection input retains both arm operands, condition and
  destination binding; a private complete-body certificate retains both paths.
- Resolve the user condition from its actual Boolean parameter; distinguish it
  from subsequent compiler cleanup switches. Constant propagation is closed and
  bounded, with exact definition/use accounting and no unknown-value guesses.
- Both paths move the selected chain into the destination, read it, drop it once
  and drop every remaining live chain once in correct lexical order. The moved
  source is never dropped again. Shared destination locals do not conflate the
  two mutually exclusive parameter-rooted producer chains.
- Include swapped arms, renamed/reordered parameters, prefix moves, unrelated
  live owners, tail/explicit returns and both Boolean outcomes. Unsupported
  source operations, ambiguous anchors and control flow diagnose.
- Wrong condition/arm/destination, flag definition/value/successor, missing or
  duplicate cleanup, cross-path owner substitutions, source scope mistakes and
  unaccounted reads/writes fail dedicated non-vacuous corruption tests.
- Exact privacy and non-erasure compile failures, invalid-source and empty-
  inventory controls, existing C/Java/native/lint targets, fresh independent
  review and an isolated full exact-tree gate precede this checkpoint's push.

Branch-local allocation, partial owned records, arbitrary branch expressions,
loops, custom Drop, unwinding and function transfers remain required separate
work. No C/Java heap output is enabled by this compiler-only checkpoint.

## Current evidence

- Pinned observation run `5bc93433-0316-40f6-aefb-1520e4f684f1` established
  a common destination local and two complementary cleanup decisions. The
  initial relation run `2532c338-289b-422d-90af-227f957c9318` authenticated
  ordinary and prefix-moved selection before the full fixture suite.
- Focused gate `8a733cb6-93f2-4080-9212-f8b5b19bf45e` passed all 7 tests
  in 19.810 seconds: runtime, format, three exact E0451 privacy boundaries and
  two exact E0308 non-erasure/grammar boundaries. Compiler-adapter Clippy builds
  continue to deny warnings.
- Eight positive fixtures, ten valid exclusions and one tail compatibility
  fixture check canonical source identities and independent execution of both
  MIR paths. Thirty-three whole-MIR mutations, two isolated flag-use/definition
  corruptions and four source-identity/scope corruptions reject. Swapped-source,
  changed-guard and nonempty-inventory controls fail as intended; E0382/E0502
  fail before compiler proof output.
- The first isolated gate passed 414/415 tests; Buildifier required the public
  target macro's standard `name` parameter. The macro now takes and uses that
  parameter. No compiler/ownership/native test failed; a corrected full gate
  is required before closure.
- Corrected tree `427edb52cf7ad9e5689713ece90912b8c3198b59` passed gate
  `7ac131c4-45fe-4321-8eef-692c30d1c1b8`: 415/415 tests across 538 targets,
  19.876 seconds, three tests executed with valid cached results retained.
  Historical compiler/C/Java/native and Rust/Bazel lint gates passed.
- Fresh independent Sol Extra High review found no concrete core defects. It
  confirmed canonical HIR/MIR identities, exact complementary flag decisions,
  complete block/edge/instruction accounting, independent ownership cleanup,
  cross-path binding maps, evidence privacy and non-vacuous tests. Its only
  housekeeping note was a pre-existing untracked Python cache; that unrelated
  cache is excluded from this exact tree and intentionally left untouched.
- Exact Git blob bytes/executable modes were verified, and all twenty preserved
  ownership files matched baseline. The checkpoint commit records the final
  documentation-inclusive tree and repeated full gate. Target heap generation
  remains disabled; deferred parent capabilities remain required work.
