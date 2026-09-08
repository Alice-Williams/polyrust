# M34A-11-02A — C registries and nominal identities

- Status: in-progress
- Depends on: M34A-11-01, M34A-11-01R

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

This registry-only foundation may proceed while the amended C ABI is reviewed
and Java's hosted CI finishes. It introduces no context-verified AST package,
ABI lowering or certificate. Those boundaries remain gated on M34A-11-00R and
M34A-10AB at the next slice; review findings still apply before integration.

## Definition of done

- Add private registry-scoped, kind-specific identities for struct, union, typedef, enum, enumerator, function, object, member, parameter, local and file registrations. References retain origin, owner and complete structural type/signature.
- Register exact interface adapter/witness/table identities and function-owned loop, switch, cleanup-exit and allocation identities for later AST/proof nodes. Later slices cannot replace them with untyped integers or manufacture proof facts.
- Generated functions and callable members carry a private exact contract
  identity in addition to their prototype. These are body-proof obligations,
  not trusted effect flags; known contracts are catalogue-owned at stage 03.
- Extend the existing CObjectType foundation with the closed nominal categories. Separate known-library origins from generated origins; reject crossed registry/kind/owner references.
- Keep canonical identity/name ordering independent of transient allocation counters. No public source/certificate constructor or string-based symbol lookup.
- Typedefs cannot hide array parameter/return categories or effective const
  qualification. Derive expanded shape from actual registered targets and test
  nested aliases; do not weaken the existing private signature wrappers.

## Tests and proof

- Compile-fail controls for fabricated/cross-kind references and private registry evidence.
- Positive registration/lookup round trips; rejected cross-registry, wrong member owner, duplicate definition and alias-cycle controls; deterministic registration inventory.
- Existing type/declarator tests, Rustfmt/Clippy/Buildifier, full tracked/release/eight-target gates in Linux.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02A; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.

## Implementation checkpoint

The new ast/registry modules provide 18 kind-specific registration categories,
exact source/file/owner/type/signature references, generated callable-contract
identity, immutable alias targets and a consuming read-only registry freeze.
Address-free inventories are computed from the authoritative registrations;
private ephemeral scope identity authenticates references but never names code.
Typedef expansion retains separate declaration provenance, including aliases
erased from prototype compatibility, so a foreign alias cannot disappear before
the registry check. Alias cycles are impossible through the immutable,
already-registered-target constructor; private-target mutation and bare-function
alias attempts are compile-fail controls. Forward nominal pointers remain legal.

Generated references do not masquerade as known-library catalogue references.
The latter remain stage 03 work. Function contracts currently identify the
generated body that must be verified, not a trusted effect summary. Likewise
scope, allocation and interface registrations are not flow, ownership or ABI
certificates; those proofs remain 02C/02D and later native gates.

The initial scope test accidentally used protected keyword switch as its
synthetic identity. Invocation fa8f1225 passed 39/40 Rust tests and correctly
rejected that fixture name; it was replaced by selection. Invocation
f398dd9f then passes all 40 tests, doctests and linters alongside ABI probes.
The additional inventory/contract tests pass in 72ea04b6-9374-4adc-a5dd-f9c2958c2414.
Full tracked/release/eight-target and fresh immutable review remain required.

## Full checkpoint proof

- The first full run 9f69867e-5518-48a5-b382-931f1bb68782 passed 313/314
  tests; only source-policy classification of the new handwritten ABI oracle
  failed. Its exact fixture path is now allowed, with deliberate adjacent-path
  controls that continue rejecting generated templates. No broad exemption.
- d9b876b0-47d0-48c1-84ea-7a941fa3b8c1: all 439 tracked rules and 314 tests pass.
- 71a46a7b-5f5d-475a-a03a-d90ab08f8daf: all 251 release tests pass.
- 6e6b26af-a9b2-4596-bc46-75b6775418e0: evaluator/eight-target agreement
  for 50 cases and one portable test; repeated manifests are byte-identical.
- C Bazel logs confirm 42 unit tests, 14 compile-fail controls and one positive
  doctest. Supplemental Linux cargo +1.98.0 test -p polyrust-backend-c
  --all-features --locked passes the same inventory.
- All new production modules are below 500 lines; the largest registry module
  is below 300. Production Bazel sources still exclude the test modules.

These are registration/type-foundation results, not generated C AST/native
certification. This checkpoint remains in-progress pending its fresh immutable
review and hosted evidence; the rest of M34A-11 remains open.
