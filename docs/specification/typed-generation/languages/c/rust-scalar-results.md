# Rust scalar results in C17

- Status: planned
- Contract: [shared](../../rust-scalar-results.md)

Use a source-derived complete struct with a typed Boolean success tag and I32
payload. Initialize both fields on every construction; canonical error
construction sets the unused payload to zero. Any error-tagged payload is
ignored semantically. Avoid unions, uninitialized storage and error sentinels
that collide with valid integers. By-value transport needs no custom allocator.

The result's nominal identity, member identities and complete defining header
must survive independent package certification/import. Extend the currently
scalar-only public signature profile narrowly; arbitrary aggregate, recursive,
pointer-bearing or heap types stay rejected. Update call/frame/metadata/output
budgets before admitting the shape. No unchecked C struct/name reconstruction.

Source match lowering tests the tag before exposing success payload or the
opaque error witness; it evaluates only the selected arm. Foreign C callers
must use the published nominal type. Fully initialized representation does not
authorize observing an inactive payload in the source model. Prove standalone
headers, native ABI/copies/returns and original dependency ownership.
