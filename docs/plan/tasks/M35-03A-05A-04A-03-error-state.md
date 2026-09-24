# M35-03A-05A-04A-03 — Preserve the original opaque error state

- Status: planned
- Parent: [instance ownership](M35-03A-05A-04A-instance-ownership.md)
- Depends on: [graph observation](M35-03A-05A-04A-02-instance-probe.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md)

## Contract

The pinned compiler reports a one-byte standard TryFromIntError, not a unit
value. Preserve its original IntErrorKind value across public transport even
though translated source initially cannot inspect it. Do not narrow arbitrary
incoming errors to the two kinds produced by i64-to-i32 narrowing. The source
instance key is unchanged; target representation profiles become version 2.

## Implementation and definition of done

1. Authenticate the original private wrapper field, its normalized enum type,
   every fieldless variant and compiler discriminant under the same core root.
   Establish the full pinned inventory, not a spelling-only or layout-only
   nominal match. Preserve typed original IDs and reject changed inventories,
   unknown values, added payloads, drop or incompatible layouts.
2. Define a closed typed error-kind mapping for Empty, InvalidDigit,
   PosOverflow, NegOverflow, Zero and NotAPowerOfTwo. Stable transport codes
   are explicit mappings, not implicit Rust enum-memory discriminants. Bind
   the map to the original compiler witness and reconcile it across the graph.
3. Specify concrete native oracle creation/inspection for every reachable kind
   without admitting unstable source operations, private-field access or unsafe
   transmutation in translated input. A public incoming signature has no
   narrowing-producer provenance restriction. Cover all six target codes with
   independent typed fixtures; distinguish structural coverage from safe Rust
   construction of each kind. Do not claim a Rust ABI bridge or reification.
4. Prove at least positive overflow, negative overflow and NonZero conversion's
   zero error remain distinct through forwarding and Err reconstruction.
   Compiling collapse/swap faults must fail the oracle. If the native oracle
   cannot distinguish a promised state safely, document and resolve the gap
   before opening target or source admission, rather than erase the state.
5. Specify the version-2 C Bool/I32 carrier (I32 stores success or error-kind
   code according to the tag) and Java six-constant Error enum. Closed
   error witnesses are not ordinary success integers; invalid foreign codes
   follow an explicit checked boundary policy.
   The compiler's original error field/kind/variant table must be retained and
   reconciled on every encounter; 04A-02's seven nominal IDs do not certify it.
6. Complete resource, identity substitution, exact/one-over, fail-closed and
   independent GPT-6-SOL reviews plus the full Linux Bazel release/lint gate.
   Commit/push separately. No target or source admission in this checkpoint.

The prior empty-error target tests remain useful transport/evaluation evidence,
but are not proof of lossless Rust 1.98 error-state transport.
