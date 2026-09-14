# M35-01E-01 — Shared compiler capability contracts

- Status: complete
- Parent: [M35-01E](M35-01E-java-rustc-retrofit.md)
- Depends on: passing M35-01D implementation/native gate; final read-only
  native-proof review may run alongside this representation-neutral extraction

## Goal

Give C and Java the same compiler-owned input categories without making Java
depend on C mappings or introducing a replacement Rust AST.

## Implementation

- Extract Capability, Mapping and Supports plus the ten current HIR input
  contracts into source_capabilities, one file per capability.
- Inputs retain rustc lifetimes and actual HIR/Ty/DefId values. Context and
  output remain associated types of the executable target mapping.
- Keep C's consuming builder and exact context/output bounds in c_lower.
- Declare shared source files in every affected compiler/probe Bazel action.
- Reuse source_check and compiler configuration unchanged; extraction alone
  does not change admission, source authority or target certification.

## Definition of done and tests

- No C/backend target type or C reader occurs in the shared contracts.
- Every existing C capability compiles against the shared identity, not copied
  marker types; missing/duplicate/wrong input/output/context compile-fail tests
  still reject for their intended reasons.
- All compiler probes, native C matrices, rustfmt, Clippy, Buildifier, docs and
  full release/migration gates pass. Generated C remains unchanged by extraction.
- Independent review finds no unresolved core issue. This is a reusable input
  boundary, not a claim that Java mappings have been implemented.

## Implementation evidence

The ten shared input contracts and Capability/Mapping/Supports now live in
src/source_capabilities. C's original consuming builder retains exact Reader,
unit-context and C output constraints. All eight compiler/probe roots and their
declared Bazel source sets use the same definitions. The independent
source_capabilities_probe has no C, Java or codegen dependency; it type-checks
all ten inputs and two distinct executable mapping signatures over one input.

Focused gate 0e92484a-49f1-4854-98f2-d5526539e3a0 passed all 13 targets,
including existing C missing/duplicate/wrong-input/output/context negatives.
The first full gate exposed a missing rustc_driver linkage declaration in the
new standalone probe. Adding the same pinned driver linkage used by the other
compiler probes fixed it; no new dependency or suppressed warning was needed.
Final gate 5630aeea-7116-4b63-bfcf-5a07969cb6a2 passed 343/343 tests
across 413 targets. Both the ten-file chain bundle and thirteen-file native
diamond bundle remain byte-identical to the pre-extraction workspace examples.
Fresh independent Sol Extra High extraction review is clean after the linkage
repair. No unresolved core issue remains. This closes M35-01E-01.
