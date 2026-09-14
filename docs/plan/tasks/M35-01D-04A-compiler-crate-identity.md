# M35-01D-04A — Explicit compiler crate identity

- Status: complete
- Depends on: [M35-01D-03](M35-01D-03-c-public-packages.md)
- Parent: [M35-01D-04](M35-01D-04-c-crate-linking.md)
- Contract: [crate dependencies](../../specification/typed-generation/languages/c/rust-hir-crate-dependencies.md)

## Goal

Replace the package experiment's implicit shared crate identity with explicit,
validated compiler configuration before admitting foreign calls.

## Definition of done

- One typed compiler configuration owns crate name/disambiguator and fixed
  compiler arguments. Package CLI/Bazel actions supply declared values.
- Default invocations remain compatible; malformed, duplicate or incomplete
  explicit configuration rejects without output mutation.
- Package symbol spelling derives from rustc's stable crate/declaration pair,
  not local declaration hash alone, source path or transient compiler numbering.
- Keep foreign calls diagnostic; this slice is not separate-crate linking proof.

## Tests and proof

- Repeated processes/configurations produce identical files and manifests.
- Same source/item names in differently named or disambiguated crates have
  distinct identities/symbols; relocating source files preserves stable identity.
- CLI negative matrix covers malformed names/keys, missing or duplicate options,
  unsupported option placement and missing/existing destination preservation.
- Bazel declared identity attributes influence generation outputs; old selected
  entry and package native proofs, Clippy/rustfmt/buildifier/docs remain green.
- Record fresh review and visible updated artifact before completing the slice.

## Evidence

- `Configuration` owns a closed Mode/Identity model, validated name/key types,
  declared input pairs and the fixed compiler argument list. The package Bazel
  rule carries identity attributes explicitly. Selected-entry defaults are intact.
- Compiler-derived package names include both crate and declaration identities.
  The native proof generates identical source in three distinct identities,
  repeats each in separate adapter processes and relocates the root source;
  identical configurations remain byte-identical, while all six function symbols
  across the three crates are distinct.
- `2999842e-6cdd-4c52-b746-b9b83651805c`: 7/9 focused tests passed. Fixed
  the required Bazel macro name argument and test handling of normal Bazel
  runfile symlinks; direct CLI outputs still must contain regular files.
- `eb1d7f94-ad49-46f9-a909-676245027015`: full release/frontend/C/shared/
  Java gate passes, 351 targets and 311/311 tests (73 seconds, cached results
  enabled). Sixteen independently compiled native consumers cover 8,204 vectors,
  both header orders, GCC/Zig O0/O2 and GCC ASan/UBSan under a 1 MiB stack.
  Twenty-five negative configurations preserve absent/existing output. Explicit
  adapter-like crate names with spoofed RUSTC_BOOTSTRAP cannot enable unsafe code
  or unstable features. Four pure configuration tests have a separate Bazel target.
- Tested A/B/default packages were copied to ignored
  `experiments/rustc-frontend/output/crate-identity/{a,b,default}` and each
  byte-matched its declared artifact. API JSON SHA-256 values respectively:
  `3f33a3164ddc6c3c9f1c2236a2f69604744d3134378087a7f2636337093ea0eb`,
  `27a67bdea5186c852d351da841aa074d0998b5336fbf9423157ead9ce935e466`,
  `e84f4698e813697e4a74985e04f01c9097fd19164c6791afb535e23aea1e08f9`.
- Fresh independent Sol Extra High review found no issues in the final source,
  corrected tests and saved artifacts. Foreign calls remain
  diagnostic; no dependency certificate or cross-crate call support is claimed.
