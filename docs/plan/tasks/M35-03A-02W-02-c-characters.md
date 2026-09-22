# M35-03A-02W-02 — C Unicode scalar target foundation

- Status: complete
- Depends on: [oracle](M35-03A-02W-01-character-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-character-values.md)

## Contract

Represent checked scalar values as typed U32 expressions, parameters/results
and ordinary declarations. Reuse the existing uint32_t catalogue and inferred
stdint dependency; no C character token, wchar_t, raw fragment or runtime.

## Definition of done and tests

Before compiler admission, certify actual target packages for exact literal/
identity transport, immutable locals, conditional selection and six U32
comparisons. Run the full scalar corpus and comparison pairs through GCC14/Zig
O0/O2 and GCC UBSan with standalone headers and strict warnings. Detect actual
byte/UTF-16 narrowing and wrong-order faults. Authenticate graph, numeric and
call-effect evidence; adding U32 transport must not admit missing/indirect/
foreign callees or pointer storage. Prove correct inferred dependencies, resource
bounds, syntax/compile-negative controls and unchanged old output. Full Linux
release/lint and fresh broad review must pass before separate commit/push.

## Implementation and proof inventory

The implementation changes only the closed target profile, public dependency
signature inventory and scalar-call signature classification. U32 now passes
through ordinary parameters/results, record fields and exact same-type
conditional nodes. The renderer, numeric verifier and dependency catalogue
are unchanged. Source Char admission is still disabled.

Focused Rust fixtures construct 19 exact boundary literals, identity,
immutable-local transport, Boolean selection and all six comparisons. Two
separately certified consumers forward the original identity through immutable
locals. Another package exercises a private record and private helper call.
The native consumer drives the complete 1,112,064-scalar domain plus 4,453
comparison pairs through these actual rendered packages; no million-node
literal fixture or handwritten replacement implementation is used.
The native matrix has its own independently cached Bazel action,
//crates/backend-c:c_characters_native_test; the existing partition-contract
test ensures it is not silently dropped from the complete suite.

The native matrix compiles separate objects and standalone public headers with
GCC14 and Zig at O0/O2, plus GCC UBSan. Fault cases change the actual producer
to truncate to eight/sixteen bits or reverse comparison operands, rebuild all
owners and compare exact binary observations with independent faulty models
and correct truth. Missing-header and malformed-source controls must fail
compilation. The handwritten consumer is an exact-path source-policy exception;
adjacent production/fixture paths remain negative controls.

Typed checks cover foreign registry identities, missing and indirect/foreign
callees, conditional call edges, pointer parameters, unsupported public U64,
mixed comparisons, mismatched conditional types, unadmitted narrowing/division,
excessive depth and numeric representability. Positive certificates are also
required: a negative test must not pass because the whole fixture is invalid.
Source-byte/stack bounds and inferred stdint/no-math dependencies are checked.
Valid C integers outside Unicode remain valid target integers, explicitly
preventing target certification from being mistaken for source char validation.

Full Linux `//... //:release_gate` passes: 1,016/1,016 targets, including Rust
and Bazel lint checks (79 executed, the rest cached). The tested implementation
tree is 88a2ce005b145a12e056d4daa6358f7d3db0109c; invocation
93917223-d60e-4aa3-9ac0-0c9bc6b8ffdc. All 501 prior generated file hashes
and 38 unrelated WIP hashes are unchanged. An initial gate exposed only the
missing exact-path handwritten-test-consumer policy registration; that was
fixed without relaxing production policy. Source admission remains disabled.

## Review disposition

The first broad Sol Extra High review found no core defect, but identified a
stale dependency-signature diagnostic enumerating only i32/i64/bool. This was
accepted because the actual closed predicate also admits F64 and now U32.
The message now describes admitted scalar parameters without a stale duplicate
type list; its Int-negative assertion still proves the same rejection boundary.
A fresh independent Sol Extra High review of the corrected implementation and
native-test partition (tree 88a2ce005b145a12e056d4daa6358f7d3db0109c) found
no actionable issues or optional-feature gaps. No review findings were dismissed.
