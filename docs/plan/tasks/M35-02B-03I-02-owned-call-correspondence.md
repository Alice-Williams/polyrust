# M35-02B-03I-02 — Prove local caller/callee ownership correspondence

- Status: complete
- Parent: [M35-02B-03I](M35-02B-03I-owned-function-boundaries.md)
- Depends on: M35-02B-03I-01
- Specification: [closed call graph](../../specification/typed-generation/languages/c/rust-owned-call-graph.md)

## Contract

Authenticate complete Producer, Consumer and Relay bodies, then pair admitted
call sites with those exact callee certificates. Use only compiler-owned HIR,
types, normal-flow MIR and cleanup evidence after successful Rust analysis.
The source grammar, graph budget and admitted compositions must be written into
the specification before enabling the public query-only graph certificate.

## Definition of done and tests

- Producer proof relates the scalar parameter to a standard Box allocation and
  its return transfer. Cover direct constructor-to-return-place and a chain of
  source local moves ending at the return. No local cleanup may remain.
- Consumer proof traces an owning Box parameter through whole local moves to
  the scalar read and one final Drop. Relay proof traces that parameter to the
  return place without allocation or cleanup. Keep source exits observable.
- Caller proof accounts for scalar/owned argument staging, the exact callee
  DefId/signature, returned owner/scalar, further moves and final cleanup.
  Call graph assembly requires the matching complete callee proof, not a
  signature-only role, with cycles and missing callee evidence rejected.
- Every normal block, call, assignment and relevant owner local belongs to the
  complete relation. A transfer cannot duplicate or silently discharge ownership.
- Typed corruption controls replace callee, argument source, staging edge,
  result/destination, move, return and drop. Same-typed substitutions must fail
  by identity. Include cycles, missing/duplicate evidence and unsupported bodies.
- Consumers exercise executable operation mappings and canonical tail/return
  projections. Private body/graph construction, evidence erasure and unchecked
  MIR inputs fail exact compile-negative contracts; invalid Rust emits no proof.
- Full isolated historical/native/lint gates, fresh independent review and
  documented exact-tree evidence precede the dedicated commit/push.

Native C/Java heap output, clone, borrowed calls, recursive or generic functions
and record-payload call transfers are outside this closed increment.

## Initial graph form and budgets

Begin with a two-function graph: one safe nongeneric i32-to-i32 entry function
and one directly called leaf function with an authenticated I-01 signature.
The entry makes exactly one owned local call. A leaf contains no further local
call; recursion and longer call chains therefore reject before graph admission.
The graph evidence must retain both actual function identities and the exact
call edge. This bounded stage does not claim arbitrary acyclic call chains.

Each body has one root scope, at most 128 immutable let bindings, and the
existing 512-normal-block/1,024-local MIR bounds. Every Box binding belongs to
one whole-owner chain. Producer leaves allow a direct constructor exit or a
constructor binding followed by local moves and a final owner return. Consumer
leaves read/destroy their owning parameter after local moves. Relay leaves
return that parameter after local moves, without allocation or cleanup.

The entry either calls a Producer and later reads/drops its returned Box, builds
a Box and transfers it to a Consumer as its scalar exit, or builds a Box,
passes it through a Relay and reads/drops the returned Box. Whole local moves
may occur between these specified operations. Keep tail/explicit returns and
the direct return place distinct from source-local bindings. Reject arbitrary
scalar expressions, borrows, branches, extra calls and unproved leaf effects.

Before implementation, add explicit-return/move variants to the observation
fixture and pin their actual argument/return staging. Split source admission,
leaf relations, entry relations and graph evidence into focused modules.

## Implementation and evidence

- Source admission is split into `calls/frame.rs` (canonical root/signature),
  `calls/source.rs` (closed enum grammar), `calls/moves.rs` (unique owner edges),
  `calls/relations.rs` (complete normal trace), `calls/evidence.rs` (read-only
  projections) and `calls/mod.rs` (query-only graph assembly). Entry and leaf
  relations share the same ordered operation checker; neither infers effects
  from a role signature. No file in this increment exceeds 500 lines.
- `OwnedCallGraph::read` obtains both bodies from the same compiler context.
  Private assembly requires the exact callee identity, signature and full Box
  type. Consumers can inspect both canonical exits, source-binding/MIR pairs,
  argument staging, call destinations, moves, read/drop and return locations.
  Consuming the bodies supplies the existing executable operation mappings.
- Focused seven-target gate `7635f801-7a36-4d77-9dc6-d83205809c77` passed
  in 19.711 seconds, including five exact compile-negative privacy/raw-input/
  erasure/assembly contracts. That initial suite had 439 body/graph corruptions.
- Expanded runtime/format gate `23338ca5-c113-40ee-9d92-412b7c6d8216`
  passed in 17.887 seconds. The inventory has ten admitted graphs and 28 rejected
  entry bodies; ten constructor and ten local-call mappings execute. Four invalid
  Rust controls, six valid-source oracle mutations, and the accepted 128/rejected
  129 binding boundary run before the success marker of the Python harness.
- Final review and gate evidence is recorded below. Target heap output stays disabled.

## Review repair

The initial isolated tree `f0992b3e6436b682ebca2ed843c3b4bc01f09fb3`
passed gate `999240a4-9bab-4efb-aaef-5eac1e34dd06`: 469/469 tests,
599 targets, 50.405 seconds, after verifying 2,274 exact Git blobs/modes.
Independent Sol Extra High reviewer `owned_call_graph_review` found one core
contract defect and no others: scalar argument provenance did not require the
specified direct Copy staging edge.

We agree with the finding. Moving an i32 or adding a copy can be semantically
equivalent, but this closed certificate promises the pinned representation,
not an arbitrary equivalent rewrite. The public query-only boundary was not
bypassed; nevertheless the relation must reject those inconsistent claims.

Three coherent counterexamples reproduced rejection-oracle failures before the
repair: removed staging/direct argument (`36ddaa8e-422d-4282-a474-2f614e77b6a0`),
Move staging (`d2738272-8671-4d07-a28f-5753411b8033`) and an extra transitive
Copy stage (`bead6695-772c-44f6-a9ef-ccbd32637e63`). The calls-only relation now
requires one distinct stage defined by Copy of the exact parameter, without
narrowing the shared historical scalar-provenance helper.

Repair gate `3ee9222e-68a9-442a-bc16-5091ab25c8f1` passed runtime/format in
16.398 seconds. All 475 body/graph corruptions pass, including 36 new coherent
staging mutations across the twelve scalar call sites exercised by the mutation
suite.

Repaired tree `7b158e2cb21c7c2624114b5903cf79094b886072` passed full isolated
gate `84209986-5ed8-4cd1-aef2-117fea8c1e87`: 469/469 tests, 599 targets,
42.076 seconds, with all 2,274 Git blobs/modes verified. Fresh independent Sol
Extra High review `owned_call_graph_recheck` found no further core defects.
Its broad review covered the repaired staging contract, complete body/graph
accounting, public projections, privacy, limits and test scope.

Closure documentation receives a final isolated full gate before commit; the
checkpoint commit records its exact tree and invocation. Only the bounded
two-function compiler correspondence is complete, not C/Java heap support.
