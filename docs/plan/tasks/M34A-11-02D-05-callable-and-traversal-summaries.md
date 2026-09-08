# M34A-11-02D-05 — Authenticated callable and traversal summaries

- Status: planned
- Depends on: M34A-11-02D-04

## Goal

Compose safety across actual generated bodies and flat interface dispatch.

## Definition of done

- Derive generated callable preconditions/effects/normal and failure outputs from
  their bodies. Apply standard contracts only through closed known identities
  after checking every operand-specific precondition.
- Authenticate indirect callable origin, exact member/table/witness/adapter,
  borrowed receiver, allocator and outcome type; a prototype match is insufficient.
- Pre-register and analyze legal synthesized lifecycle components without assuming
  a topological body order. Unknown/circular claims cannot bootstrap proof.
- Validate runtime-traversal.md's one-step enqueue contracts: same work engine,
  persistent destination lifetime, no recursive driver, commit only after complete
  work, allocation-free intrusive drop/rollback and actual finite work accounting.
- Reject TLS/global hidden engine state, nested public lifecycle recursion,
  unproved callback effects and user-call recursion excluded by checked Core.
- Keep summary evidence private, immutable and derived; no selectable purity,
  helper-name exception or caller-supplied success/failure transition.

## Tests and proof

- Same-prototype wrong-function/contract/member/table substitution, reordered
  allocator arguments, wrong context and different engine identity.
- Legal recursive specialization identity graphs with finite iterative work,
  versus circular summary assumptions, recursive native drivers and expired
  stack destinations. Perturb registration/body order without changing results.
- All failure/partial-prefix/cleanup paths, hidden allocation in drop, premature
  output commit and temporary comparison allocation failures.
- Focused summary/compile-fail/mutation tests and complete cached tracked/release/
  lint/eight-target gates. Stage 06 still proves actual generated runtime bodies.

## Commit gate

Record evidence and commit/push M34A-11-02D-05; callable registration or a
successful local type check alone must never establish a summary.
