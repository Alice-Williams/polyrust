# M35-03A-05A-04A-01 — Descriptive instance and owner identity

- Status: complete
- Parent: [instance ownership](M35-03A-05A-04A-instance-ownership.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md)

## Contract

Introduce a closed, fixed-size descriptive identity for the selected standard
Result<I32, TryFromIntError> instance, complete original variant/field facts and
an owner enum separating source crates from canonical instances. Representation
profiles remain backend-owned types. Metadata does not authenticate a compiler
session, grant target handles or admit new source constructs.

## Implementation and definition of done

1. Private fields and fallible constructors enforce same-crate and distinct-ID
   consistency. Keep primitive/nominal argument roles explicit; no strings or
   arbitrary generic-argument lists. Preserve all original definition IDs.
2. Retain the actual core root as a separate anchor, not an owner key. Fixtures
   demonstrate two distinct instance keys with the same root remain distinct.
3. Dedicated Bazel tests cover every fact role, cross-crate substitutions,
   duplicate-ID pairs, ordering, profile distinctions and source/type separation.
   Compile-fail coverage protects construction invariants. Key/fact metadata has
   no caller-controlled allocation; graph resource limits belong to 04A-02.
   The generic owner is allocation-free only for backend-selected fixed-size
   closed profiles, not arbitrary caller-chosen generic parameters.
4. No target name rendering, source admission, certificates, or backend changes.
   Full Linux Bazel release/lint, unchanged old outputs/WIP, independent
   GPT-6-SOL review, then scoped commit/push with evidence recorded here.

## Evidence

The closed key, seven-role facts and source/type owner enum are implemented in
focused shared modules. Constructors validate all cross-crate and duplicate-ID
pairs; metadata remains descriptive, not compiler or target authority.

- Linux Bazel full release/lint gate: 1,043/1,043 targets passed, 178 executed,
  2,432.953 seconds; invocation `39e827a9-7c41-40c3-a249-b0ca830cb9d5`.
  Code tree: `f049ce39239596b32e1b66948fbf5ce5e6112e61`.
- Nine dedicated identity tests and construction compile-fail doctests pass.
  Ordinary Java suite: 483 passed, zero failed/ignored; five native partitions
  are separately included in the full gate.
- Rust formatting, Clippy and Bazel formatting pass. An initial missing root
  visibility declaration for the new test was corrected before this full gate.
- All 530 recorded generated files remain byte-identical; 45 protected WIP
  files remain unchanged and outside the checkpoint.
- Two independent GPT-6-SOL reviews report no remaining core defect. Review
  clarified that the generic owner is allocation-free only with fixed-size
  backend profiles; it does not promise that for arbitrary generic parameters.

Compiler graph observation remains 04A-02. No source Result admission or target
type-owner certificate is enabled by this checkpoint.
