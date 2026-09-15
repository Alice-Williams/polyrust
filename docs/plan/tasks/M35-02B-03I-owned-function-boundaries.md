# M35-02B-03I — Authenticate owned transfers across local functions

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03H-02
- Specification: [owned function boundaries](../../specification/typed-generation/languages/c/rust-owned-functions.md)

## Contract

Extend compiler-backed ownership correspondence across resolved local function
calls. Begin with Box<i32> producers, consumers and relays; do not infer a
callee's behavior from its name or signature alone. Authenticate the callee body
and the caller transfer together before making any cleanup claim.

Keep HIR blocks and calls as the eventual structured output. Retained MIR call,
argument, return and drop places are evidence, not a control-flow renderer.

## Implementation sequence

The implementation checkpoints are [I-01 — Call identities](M35-02B-03I-01-owned-call-identities.md)
and [I-02 — Caller/callee correspondence](M35-02B-03I-02-owned-call-correspondence.md).

I-01 is complete: authenticated call identities and executable registrations
passed 462 tests and independent review. I-02 is complete for the specified
two-function entry/leaf graph, with 469 tests and a clean fresh review after
repairing scalar staging. Longer chains, recursion and target heap output remain
unsupported; the role signature alone still grants no ownership effect.

1. Observe pinned HIR/PostCleanup bodies for a local producer returning a newly
   allocated Box, a consumer dropping an owned parameter, a relay returning its
   owned parameter, and callers using each. Include aliases, local moves and
   explicit returns. Record compiler argument staging and return-place behavior.
2. Specify and implement private typed direct-call inputs using canonical HIR,
   LocalDefId/DefId, instantiated function signatures and full standard Box Ty.
   A closed role enum distinguishes the admitted signatures without claiming
   their effects. Register an executable mapping through the consuming builder.
3. Authenticate each role's complete callee body and the caller's corresponding
   transfers. Assemble an acyclic bounded local call graph, requiring the exact
   callee evidence for each resolved call. A same-signature replacement cannot
   inherit another function's certificate.
4. Add positive and corruption/compile-negative fixtures, fresh independent
   review, exact-tree historical/native/lint gates and a dedicated checkpoint.

## Definition of done and tests

- A producer's return transfers the authentic allocation obligation to its
  caller; a relay passes the incoming obligation unchanged; a consumer performs
  the one authenticated cleanup. No transferred owner is also dropped by the
  transferring frame. Normal scalar returns retain their exact producer.
- Compiler staging locals are not invented source bindings. Every Box move,
  argument, call destination, return and drop belongs to a complete typed path.
- Tests include same-signature/same-spelled functions, aliases, local moves,
  early rejection of unproved callees and invalid Rust before proof output.
- Substituting a callee, argument, result, cleanup owner or signature, omitting
  or duplicating a transfer, and introducing recursion fail closed.
- Missing/wrong capability registrations, fabricated inputs/certificates and
  unchecked MIR construction fail compilation with exact expected diagnostics.
- Existing single-body proofs and no-heap C/Java/native/lint gates remain green.
  Document restrictions and proof evidence before commit/push.

This stage does not enable target heap output. Borrowed parameters/results,
closures/dynamic dispatch, generics, external calls, recursion, custom Drop,
unwind, record payload transfers and clone require separate closed contracts.

## Preparatory compiler observations

Gate `9e187fe1-7657-4ad2-9e79-1b39fb88ba9a` passed both observation
and format targets in 15.051 seconds. Eight canonical HIR/PostCleanup bodies
were inspected; this is preparation, not an owned-call certificate or closure.

- A direct producer tail calls Box::new into RETURN_PLACE. The explicit-return
  producer first receives a local Box, then moves it into RETURN_PLACE. Neither
  producer has a final Drop: that obligation leaves the frame.
- The consumer moves its Box parameter through a local, reads through the
  authenticated pointer path, drops the final local and returns its scalar.
- The relay moves its parameter through a local into RETURN_PLACE, with no
  constructor and no Drop.
- Callers stage owned arguments in distinct Box locals before moving them into
  the call. A consumer caller has no local Drop; a relay caller drops the
  returned owner; a producer caller drops the newly received owner.
- Renaming the imported producer preserves the actual callee DefId. Every
  observed call/drop has unreachable unwind under the pinned abort policy.

The implementation must model both producer forms without inventing a source
binding for RETURN_PLACE or argument staging. Before target mapping, audit all
multi-form body certificates (including G-02) for consumer-visible canonical
exit projections; H-02 review established that private retention is insufficient.
