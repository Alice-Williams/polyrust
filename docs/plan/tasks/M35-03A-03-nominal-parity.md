# M35-03A-03 — Nominal types, interfaces and structured behavior

- Status: planned
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-03A-02; M35-02 for owned shapes

## Contract

Cover existing record/enum construction and equality, methods, trait/interface
implementations and dispatch, pattern matching, local bindings, conditionals
and bounded iteration. Preserve composition rather than generic inheritance.
Use actual compiler nominal/callee/field identities and explicitly admitted
generic instances, not string-tag or ID comparisons in rendering.

## Definition of done and tests

- Close the foreign function re-export gap found by M35-03A-02C: preserve the
  dependency-owned definition identity through a public alias, and prove it with
  separately compiled C/Java consumers and missing/wrong-owner rejection tests.
  Local aliases and imported calls do not establish this capability. Split this
  into its own implementation checkpoint before claiming crate-boundary parity.

- Ordinary generated C declarations and Java types implement the admitted
  source behavior without Runtime type/member privileges.
- Native tests exercise enum cases/payloads, empty interfaces, multiple
  implementations, nested values, branch scope and short-circuit/loop exits.
- Wrong receiver/method/field identity, missing match cases, unsupported dynamic
  behavior and unproved ownership combinations reject before publication.
- Source legality and target syntax/ownership certification remain separate.
- Preserve public/private crate behavior and doc attributes in consumers.
- Operation-specific child tasks, full gates, independent reviews and commits.
