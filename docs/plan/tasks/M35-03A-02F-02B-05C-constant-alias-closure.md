# M35-03A-02F-02B-05C — Preserve cross-crate constant exports and close proof

- Status: planned
- Parent: [multi-crate constants](M35-03A-02F-02B-05-constant-bundles.md)
- Depends on: M35-03A-02F-02B-05B

## Contract

Extend the checked public inventory with a distinct foreign-constant binding
case authorized by the exact producer certificate. A public re-export preserves
the defining Rust identity and ordinary producer C/Java symbol; it does not mint
a second owned declaration, duplicate storage or create a synthetic accessor.

Retain complete finite module/alias graphs and original module/doc ownership.
A facade consisting only of foreign constant exports is a valid selected API;
do not invent a function or falsely claim ownership of the producer field/object.
Specify and check this mapping in both target source inventories and versioned
bundle metadata before enabling the compiler path. Unsupported foreign modules
or other declaration kinds remain diagnosed, not silently omitted.

## Definition of done and tests

- Direct, renamed and transitive constant re-exports retain one producer identity
  across C/Java metadata and actual imported read nodes. Re-export-only roots and
  mixed roots work without dummy functions or duplicate constant definitions.
- Native independent Rust/C/Java consumers agree for exact values and aliases;
  target files remain separated by defining crate and docs stay inspectable.
- Stale/replaced owner, wrong alias binding, missing export, foreign private value,
  wrong declaration kind, duplicate identity, cyclic/unbounded expansion and
  unsupported schema controls reject before atomic publication.
- Complete whole-graph inventory and producer mutation/rebuild proof pass.
  Existing function, local/private constant and owned-source tests stay enabled.
- Fresh independent review loops, full Linux Bazel/release/lint tests, exported
  generated examples and scoped push complete this checkpoint.
- Close 05/02B/02 only after all required evidence is recorded. Do not claim wider
  constant families, all Rust syntax or legacy runtime retirement is complete.
