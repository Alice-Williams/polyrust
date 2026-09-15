# M35-02B-03I-02 — Prove local caller/callee ownership correspondence

- Status: planned
- Parent: [M35-02B-03I](M35-02B-03I-owned-function-boundaries.md)
- Depends on: M35-02B-03I-01
- Specification: [owned calls](../../specification/typed-generation/languages/c/rust-owned-functions.md)

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
