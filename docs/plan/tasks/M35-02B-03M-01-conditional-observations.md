# M35-02B-03M-01 — Observe conditional record ownership

- Status: complete
- Parent: [M35-02B-03M](M35-02B-03M-conditional-owned-records.md)
- Depends on: M35-02B-03L-03
- Specification: [conditional observations](../../specification/typed-generation/languages/c/rust-conditional-record-observations.md)

## Contract

Observe successfully checked Rust before defining the conditional body reader.
Use a local named Pair of two standard scalar Boxes and an independently live
scalar Box for the final read. Distinguish conditional initialization of an
entire previously uninitialized record from moving one field of an initialized
record: these have different compiler cleanup obligations.

The initial fixture candidates are conditional record initialization without
an else, one-arm partial field extraction with scope-end cleanup, and partial
field extraction followed by early return versus false-path continuation.
All constructors use immutable distinct scalar parameter anchors and the
condition is an immutable Boolean parameter. Observe reversed initializer and
opposite-field variants separately. Do not assume exact normal-MIR shapes
before the pinned compiler has produced them.

## Definition of done and tests

- Both source outcomes have independently asserted complete normal traces,
  constructor/aggregate/field moves, selected read, Drop places and full Return.
- Actual Boolean source producer and compiler drop-flag assignments/decisions
  are recorded with typed locals and locations; flag identities are not names.
- True conditional initialization drops the initialized record; false leaves
  the uninitialized record untouched and cleans the still-live source owners.
- Conditional partial movement drops the extracted field once in its arm and
  remaining live fields afterward; the other path cleans both intact fields
  individually, as observed in the pinned compiler's drop elaboration.
- Early-return and continuation fixtures assert their distinct canonical exits.
- Invalid reuse, unconditional read of a maybe-uninitialized record and
  duplicate field movement fail Rust checking without publishing observations.
- Valid source mutations break an explicit observation assertion; an empty
  inventory cannot pass. Frozen compiler budgets and later admission limits
  are documented from these results, not guessed from counts.
- Full isolated Bazel/native/lint gate, fresh independent broad review and
  final exact-tree gate pass before a separate commit/push.

## Scope

Observation only: no new safe body certificate, C allocation or Java output.
Individual assignment to fields of an entirely uninitialized safe Rust record
is not an intended fixture; whole-record conditional assignment is.

## Progress

The six initial fixtures cover ordinary/reversed conditional initialization,
first/second field extraction into a short-lived arm binding, and first/second
field extraction with early return followed by the opposite-field continuation.
The assertion suite now checks both normal paths using typed source identities,
standard constructor identity/full Box type, source-ordered aggregate staging,
actual conditional whole-record movement, field paths, Boolean producer/cleanup
flags, selected read and full Return. Together the paths cover all normal blocks.
Three invalid Rust and fifteen valid-source controls include guard/constructor/
parameter/field/read/order/nominal substitutions and an empty inventory.

Focused runtime/format gate `ec37468d-205a-43ba-ae67-47cc3c87d017` passed
in 14.105 seconds. The initial isolated full gate
`6d8bff23-da2e-456a-b1da-392767b78728` passed 510/510 tests across 647
targets in 34.284 seconds, with 2,370 exact Git blobs/modes verified.

Independent review identified missing ordering edges in the observation oracle:
conditional whole-record movement before the final pointer/read, branch field
movement before its scope-end Drop, and the source guard before conditional
aggregate staging. All were accepted and explicit assertions added; checking
each event's presence alone did not establish these ordering relations.
The first repaired full gate `481cb2ee-e6fe-45f0-a251-e3c02695ad3a`
passed 510/510 tests across 647 targets in 27.588 seconds. The completed review
also identified missing explicit assertions for pairwise-distinct simultaneously
live constructor Places and the early/continuation shared MIR Return Location.
Both were accepted and added. All five findings concern the observation oracle;
no production verifier was admitted by the earlier passing tests.
The subsequent parameter repair and final review evidence are recorded below.
No complete-body evidence or target heap support is enabled.

The independent second review identified one further contract issue: raw MIR
local numbers selected formal parameters. The oracle now retains the canonical
HIR parameter identities, derives MIR arguments through `Body::args_iter`,
checks their typed positional correspondence and passes those actual locals
to the constructor and condition checks. Formal-parameter order is intentional;
hardcoded MIR-local numbering is not part of this contract.

An additional suggested scheduling restriction is not treated as a core error:
requiring the pure Copy of the immutable Boolean parameter to occur after a
pre-if record aggregate. The actual SwitchInt must follow that aggregate, as
already checked. Moving only that authenticated, non-mutating Boolean Copy
earlier changes neither ownership nor behavior, and the specification does not
promise its exact position relative to aggregate construction. Source-ordered
aggregate staging means the field-owner moves retain initializer order, not
that all compiler-generated scalar copies match source statement boundaries.
The reviewer accepted this rationale and withdrew the concern as a core finding.

## Closure evidence

Repaired tree `f98916b3aeb197442375fdea1131d6fb9478254e` passed the full
isolated Linux/Bazel historical/native/lint gate: invocation
`43aee78a-fc0b-4425-9288-8855dcc1a14e`, 510/510 tests across 647 targets,
40.039 seconds, and 2,370 exact Git blobs/modes verified before extraction.
A separate Sol Extra High reviewer inspected that immutable tree across the
whole observation scope and found no actionable core defects. The six accepted
findings were fixed; the pure Boolean-copy scheduling restriction was explicitly
evaluated and withdrawn as non-core, for the rationale above.

The final documentation checkpoint repeats the full exact-tree gate before
commit/push, with its tree and invocation recorded in the commit. M-02 source
frames, M-03 complete ownership correspondence, and target heap output remain
separate unfinished work; this observation checkpoint does not enable them.
