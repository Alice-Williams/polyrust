# M35-01E-04B-02 — Java consumer-owned dependency bindings

- Status: complete
- Parent: [M35-01E-04B](M35-01E-04B-java-imported-callables.md)
- Depends on: M35-01E-04B-01

## Implementation contract

- A consuming JavaDependencyScope builder imports only JavaDependencyFunction
  handles and freezes an immutable original-package inventory. Empty legacy
  packages need no consumer authority. Scope identity is opaque and never emitted.
- JavaImportedCallable retains the exact consumer scope and owner function proof.
  Its signature is derived from that proof, not a mutable caller-supplied record.
- The original Java file projection retains frozen bindings. Package verification
  rejects mixed scopes, omitted/foreign-scope registrations, conflicting owner
  certificates for one source crate and overlap with the consumer crate identity.
- Extend JavaCallableRef and all structural verification/reference/rendering
  branches. Calls lower through the shared Qualified spelling policy using the
  owner's typed RustCrate namespace, facade path and member identifier.
- Preserve lexical qualifier protections and resource accounting for dynamic
  qualified-name lengths, binding counts and exact used dependency inventories.
  No raw source text, static import lists, fabricated aliases or copied bodies.
- Extend closed owner admission to certified imported calls. Retain checked call
  heights in owner proofs and include imported call heights in the consumer's
  bounded acyclic call-path calculation; crossing a crate does not reset the
  existing call-height limit. Existing constructor/method budgets remain shared.

## Definition of done and tests

- Compile-negative tests reject forged imported handles/bindings, unchecked owner
  packages and mutation of a frozen scope.
- Wrong scope, substituted owner, altered signature, absent registration and
  source-owner overlap fail before render readiness. Include same source IDs with
  independently certified different bodies and equivalent-looking scope clones.
- Exact used-owner/unused-registration tests and repeat rendering prove no extra
  import directives or duplicate owner bodies.
- Cross-owner height exact/one-over cases cannot reset the admission bound.
- Focused Java/shared/C and lint gates pass; B-03 supplies independent Java 21
  consumers before the parent is declared complete.

## Review and verification evidence

- Final focused cross-backend gate `d6487427-1e59-4e30-92f6-93c773fbae6b`
  passed 15/15 targets. The reviewer completed the current-snapshot audit with
  no unresolved core correctness, authority or safety findings. A fresh reviewer
  follows the independent B-03 native/full-regression proof.

- Focused cross-backend gate `a8bf1121-8119-4869-bb82-066bcc83b2cd` passed
  15/15 targets after the initial review fixes. Later additions remain subject
  to the final B-02/B-03 gates.
- Accepted the Sol Extra High review's per-call package-scan finding: opaque
  signatures now read their certificate directly; scope membership is checked
  once per file and independently rechecked by catalogue derivation.
- Added review-requested exact/one-over transitive-owner bounds, used as well as
  unused qualified-name resource checks, explicit owner ordering, and an entirely
  unused registration whose transitive graph conflicts with the consumer.
- Adopted bounded owner/function Debug as graph-performance hardening, not an
  authority or confidentiality claim. It never recursively formats certificates.
