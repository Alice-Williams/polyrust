# M35-02B-03I-01 — Authenticate direct local owned-call identities

- Status: complete
- Parent: [M35-02B-03I](M35-02B-03I-owned-function-boundaries.md)
- Depends on: M35-02B-03H-02
- Specification: [owned calls](../../specification/typed-generation/languages/c/rust-owned-functions.md)

## Contract

Introduce a separate executable capability for canonical direct calls to local,
safe, nongeneric Rust free functions with Producer, Consumer or Relay signatures.
These are enum variants with fixed typed parameter/result contracts. The input
retains caller LocalDefId, canonical call/argument, callee DefId/GenericArgsRef,
actual instantiated signature and full Box<i32> identity including allocator.

This is an operation-identity proof only. A role name or signature cannot
authenticate allocation, ownership transfer or cleanup; I-02 supplies that proof.
Do not broaden the historical standard-Box constructor capabilities.

## Definition of done and tests

- Only the actual canonical node in the declared caller can create the private
  input. Resolve the direct path through TypeckResults and authenticate its FnDef,
  local free-function kind, empty arguments and fully instantiated safe Rust ABI.
- Require one unadjusted argument and unadjusted call result. Match one of the
  three enum signatures using compiler type identity, not printed type names.
  Standard Box<i32> derives from authenticated compiler library declarations.
- Reject indirect/computed calls, methods, closures, external/generic/unsafe/
  foreign-ABI functions, incompatible arity/types and implicit adjustments.
- A consuming builder slot stores and invokes the typed mapping. Missing,
  duplicate and wrong-capability/context/input/output registrations fail exact
  compile-negative contracts. Existing builder consumers retain inferred types.
- Fixtures include producer/consumer/relay roles, alias imports, same-spelled and
  same-signature distinct functions, canonical-node/owner substitutions, invalid
  Rust, unsupported calls and a nonempty inventory with negative controls.
- Fresh broad review, all historical/native/lint gates and exact-tree evidence
  precede a dedicated commit/push. No whole-body or target-heap claim is made.

## Current evidence

- Initial identity/registration gate `026cebf6-8d0e-427c-bc69-8a37ee3ed12c`
  passed all ten targets in 20.895 seconds: runtime, format and eight exact
  compile-negative contracts. Historical constructor/record gates separately
  passed after introducing the fourth slot.
- Expanded runtime/format gate `b5b35690-ecef-4108-8463-0c48ff9048a9`
  passed in 13.775 seconds. The closed inventory contains 35 free-function
  owners, eight accepted local calls and 24 rejected calls (including standard
  Box constructors, which retain their separate capability). Both registration
  orders invoke the local mapping; nine historical i32 Box mappings execute.
- Cases cover aliases, qualification, distinct same-spelled functions, all three
  role signatures, computed/indirect/pointer/closure/method/associated calls,
  argument/result adjustments, foreign ABI, generics, arity and payload/result
  mismatches. Every accepted input rejects copied-node and wrong-owner controls.
- Four invalid source controls reject with E0308/E0382/E0502/E0133 before proof
  markers. Unsafe calls are rejected by the pinned Rust/frontend boundary, not
  claimed as an admitted source form. Six valid-source mutations reject through
  the independent identity/inventory oracle, including empty source.
- Isolated tree `8c3623f065239fe2fbf36d5ec8b05896d61af279` passed full
  historical/native/lint gate `2119250f-c573-4b19-b94e-6ec033298a73`:
  462/462 tests across 591 targets in 53.956 seconds. All 2,259 Git blobs and
  executable modes were verified before testing.
- Independent Sol Extra High review `local_call_identity_review` found no core
  defects. It checked the complete identity reader, consuming registrations,
  inventory, mutation controls and scope boundary. Caller-body admission remains
  I-02's responsibility; the observation probe does not certify ownership.
- Closure documentation receives a final isolated full gate before commit;
  its exact tree and invocation are recorded in the checkpoint commit. Owned-call
  effects, body/graph admission and target output remain disabled.
