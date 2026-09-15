# M35-02B-03H-01 — Authenticate scalar-record Box constructors

- Status: complete
- Parent: [M35-02B-03H](M35-02B-03H-boxed-scalar-records.md)
- Depends on: M35-02B-03G-02
- Specification: [construction boundary](../../specification/typed-generation/languages/c/rust-boxed-scalar-records.md#construction-boundary)

## Contract

Introduce a separate capability for the standard Box constructor instantiated
with a closed scalar-record payload. Keep the existing Box<i32> capability
closed; adding a payload type must not silently widen historical callers.

A private shared call-authentication helper may factor canonical node, owner,
resolved direct-call identity and instantiated signature checks. Neither helper
nor capability accepts caller-authored identities or target AST types. Use an
enum for admitted scalar field kinds rather than string/ID dispatch.

## Definition of done and tests

- Input retains canonical call/argument, owner, constructor DefId/GenericArgsRef,
  result Box Ty, payload AdtDef/Ty and each field DefId/FieldIdx/normalized type.
- Admit local nongeneric named records with one to 128 i32/bool fields, default
  representation and no custom Drop. Preserve nominal identity through aliases.
  Reject other payloads, generic/external/tuple/unit/enum/union shapes, custom
  allocators/representation/destructors and adjusted/indirect constructor calls.
- A separate consuming builder slot stores and invokes the correctly typed
  mapping. Missing/duplicate/wrong capability/context/output/input and private
  construction are exact compile-negative contracts; Box-only and G's
  record-construction consumers retain their prior inferred signatures.
- Runtime fixture inventory includes mixed scalar fields, alias/import renaming,
  same-spelled module records and unsupported shapes. Canonical-node/owner/type
  substitutions and invalid Rust fail before a success marker.
- Historical constructor/record/ownership, native C/Java and lint gates pass.
  Fresh broad review and full isolated exact-tree evidence precede commit/push.

This input proves one operation's identity and closed payload shape, not the
payload's producer path, selected field read, allocation/drop correspondence or
target heap semantics. Those remain H parent work.

## Current evidence

- A private shared standard-Box call reader authenticates canonical ownership,
  direct resolved constructor identity and the actual instantiated signature.
  BoxConstructionInput still requires i32; ScalarRecordBoxInput separately
  requires the closed scalar-record metadata and rejects adjusted results.
- Historical gate `75b38172-995d-415c-804a-77711f6d1b17` passed all eleven
  selected constructor/record/partial-record tests in 19.060 seconds after the
  helper and third typed builder slot were introduced.
- New gate `c53fc183-a046-499f-904b-06cfbdd46cac` passed all twelve targets
  in 22.518 seconds: runtime, Rustfmt and ten exact compiler-negative contracts.
  The runtime inventory includes eleven accepted and twenty rejected forms;
  fields retain explicit I32/Bool kinds, normalized types and declaration IDs.
  One/128/129-field cases, same-spelled records, aliases and both registration
  orders are checked. All three stored capability mappings actually execute.
- Wrong-owner and copied canonical-node substitutions reject. Invalid moves,
  borrows and unstable allocator-api source fail with E0382/E0502/E0658 before
  success output. The allocator case is a stable-compiler rejection boundary,
  not a claim to support or type-check allocator-api programs in the adapter.
  Valid-Rust representation, field declaration order, inventory and empty-source
  mutations fail the independent fixture oracle.
- Isolated exact-tree gate `80ec4b11-5a7c-4b91-9e90-eb20196f71c2`
  passed all 444 tests across 570 targets in 48.327 seconds. The staged tree
  `892f80d69c73e09ed12e8f7b1455e089742048bf` was exported with all 2,231
  Git blobs and executable modes verified, excluding unrelated working changes.
- A fresh independent Sol Extra High review found no core defects after
  inspecting the complete staged change, historical boundaries and proof matrix.
  The final closure tree is gated again before its dedicated commit/push; that
  commit records the exact tree and invocation. Payload producer/read/drop
  correspondence and target heap output remain disabled and belong to H-02
  and later H work, not this completed construction boundary.
