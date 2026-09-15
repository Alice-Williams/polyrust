# M35-03A-01 — Parity inventory and runtime-free artifact guards

- Status: complete
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-01D and M35-01E
- Inventory: [coverage and limits](../../specification/typed-generation/runtime-parity-inventory.md)

## Contract

Freeze the feature-preserving migration boundary and catalogue existing Java
capability registrations, C admitted intrinsic operations and runtime helper
families. Distinguish full, partial and missing new-system coverage. Keep
source-operation evidence separate from executable target output.

## Definition of done and tests

- Inventory every legacy Java capability and C admitted intrinsic without
  claiming that capability names prove complete shape support.
- Assign every gap to a concrete parity task and document per-target limits.
- Add drift tests so new/removed legacy registrations require explicit review.
- Check actual C/Java Rust-source bundles contain exactly source-owned artifacts
  and no additional custom runtime/support file; injected extras must reject.
- Keep historical, typed target, compiler and native tests enabled.
- Full isolated Bazel/native/lint gate, independent review and tested commit/push.

This step does not implement all missing capabilities or remove runtime files.

## Completion evidence

- Inventoried 42 Java capabilities, 38 C admitted intrinsics, nine Java helper
  families and 17 C runtime sections, each assigned a primary parity task.
- Twelve inventory corruptions and two helper-catalogue registration mutations
  reject. Actual C/Java four-crate bundles pass exact artifact/metadata checks;
  fourteen injected artifact/dependency faults per target reject.
- Two independent whole-scope Sol Extra High reviews completed. Accepted repairs
  close schema/import dependency checks and enum-versus-executable helper
  registration drift. Both reviewers found no remaining core defects in tree
  `9b7ee186eecd0d3e604cc26e7f22f604c27b77db`.
- Isolated Linux Bazel/native/lint gate on that reviewed tree:
  `62eddec1-9dd9-4b4a-b0b1-2483fe79ce75`; all 512 tests passed across 649 targets
  in 25.041 seconds. Archive verification checked 2,383 Git blobs and modes.
- No legacy path, test, runtime implementation or policy exception was removed.
  The final documentation-only closure is gated again before commit/push.
