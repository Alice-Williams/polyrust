# M35-02B-03D — Authenticate guarded owned exits

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03C
- Specification: [guarded owned exits](../../specification/typed-generation/languages/c/rust-owned-guarded-exits.md)

## Contract

Introduce a closed two-path correspondence for a source `if` controlled by a
direct immutable Boolean parameter. Both branches explicitly return a live
Box<i32> dereference. Constructors and whole-owner moves precede the branch;
the existing distinct scalar-parameter anchors still identify every owner.

Preserve canonical HIR condition/branch/return references. Authenticate the MIR
switch discriminator through its actual Boolean parameter producer and map its
false/true successors explicitly. Prove both complete paths and their cleanup,
including shared suffix blocks, without emitting gotos or inventing a borrow
checker. This is not admission of conditional moves or compiler drop flags.

## Definition of done and tests

- Specify the source grammar, typed guard identity, bounded path inventory and
  shared-block accounting before implementation; existing linear APIs stay closed.
- Both Boolean outcomes identify their actual selected source owner and exit.
  Every owner receives exactly its required drop on each path, in lexical order.
- Shared cleanup/return blocks may occur in both path certificates but must not
  be counted as two runtime drops on one execution. No graph edge or instruction
  is left unexplained; cycles, extra edges and async/unwind cleanup reject.
- Test distinct/equal-type owners, swapped branch results, renamed parameters,
  moves before the branch, shared cleanup suffixes and both Boolean outcomes.
- Wrong/constant/negated guard producers, swapped successors, missing branches,
  wrong selected owners and missing/duplicate/reordered cleanup reject in tests.
- Safe evidence construction queries actual analyzed rustc data; private
  fabrication and missing capability binding fail at their stated typed boundary.
- Unsupported nested conditions, branch-local construction/moves, implicit
  return arms, partial moves and unrelated calls diagnose without output.
- Fresh independent review, exact-tree full Linux/Bazel/native/lint gates,
  documentation, commit and push are required. Target heap output stays disabled.

Later increments admit conditional moves/drop flags, an early-return arm with a
continuing sibling path, partial records and function-boundary ownership. This
checkpoint cannot close the parent structured-ownership milestone by itself.

## Current evidence

- Shared refactor gate `8c1c8e04-171c-4cbc-a3c3-461ffb416291` passed all
  four historical ownership runtime targets. Initial guarded gate
  `7069625c-940e-4e8c-8537-ba75ca01e027` passed 6/6 tests, including exact
  E0451 private body/guard/path checks and E0308 path-to-whole-body rejection.
- Final focused corruption gate `3fa4a97c-9f5f-4c96-b64d-a4284fb84b8e`
  passed runtime and format tests in 15.044 seconds. Seven positive fixtures,
  eight guarded-shape exclusions and one tail-only compatibility fixture cover
  both actual successors and canonical source identities.
- Twenty-one private whole-MIR corruptions and removal of the shared final
  cleanup reject. An extra compiler guard-copy producer remains valid. The
  pinned moved fixture already shares its final cleanup block between paths;
  the test requires that real shared location and its single drop on each path,
  without assuming the block's preceding bookkeeping statements are empty.
- Old root/tail/multiple/explicit-return APIs keep their source restrictions.
  GuardedPath is a separate private type, never an exposed single-path body.
- Isolated tree `12485a7d3c6a3f9067cd9d23591e115c2bf51a4d` passed full gate
  `6db15e04-780d-46aa-903e-149f4ab22e93`: 404/404 tests across 525 targets,
  35.668 seconds, eleven tests executed and valid cached results retained.
  Historical C/Java/native, compiler frontend and Rust/Bazel lint gates passed.
- Fresh independent Sol Extra High review found no core correctness, contract,
  test, privacy or Bazel findings. It confirmed false/true guard association,
  complete path inventory, pre-branch ownership, shared suffixes, canonical
  identities, evidence privacy, test non-vacuity and historical restrictions.
  Deferred ownership forms are out of scope here but remain required parent
  work; they are not removed merely because this closed review is clean.
- Exact Git archive bytes/executable modes were verified, and all twenty
  preserved ownership files matched baseline. The final documentation-inclusive
  tree and gate are recorded in the checkpoint commit. Target heap output stays
  disabled.
