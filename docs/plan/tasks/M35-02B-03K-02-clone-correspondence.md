# M35-02B-03K-02 — Borrowed clone and independent cleanup correspondence

- Status: planned
- Parent: [M35-02B-03K](M35-02B-03K-owned-clone.md)
- Depends on: M35-02B-03K-01

## Contract

Certify one scalar parameter, one standard Box construction and one standard
clone borrowing that owner. Admit only a specified immutable, straight-line
root-scope grammar, with whole-owner moves and a final scalar dereference/tail
or explicit return. Refine that closed grammar from pinned observations before
implementation; do not silently broaden historical readers.

## Definition of done and tests

- Authenticate original producer, shared-borrow place, concrete clone call,
  fresh destination, both owner chains and ordered drops against the complete
  PostCleanup normal trace; source identity cannot be inferred from counts.
- Expose source bindings/scopes, canonical exit, clone operation and full MIR
  locations to ordinary typed lowering consumers through private certificates.
- Read-original and read-clone variants prove the source survives cloning.
- Wrong borrow kind/place, alias destination, missing/extra writes or calls,
  moved-source reuse and missing/duplicate/swapped cleanup fail mutation oracles.
- Reject extra clones, arbitrary borrows, branches, custom Drop/allocators and
  other payloads until separately specified. Keep abort-mode compilation.
- Query-only/private-field/erasure controls, fresh broad review and the full
  isolated gate pass before commit/push. C allocation failure and native cleanup
  equivalence remain separate M35-02C/D obligations.
