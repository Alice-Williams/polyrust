# M35-01D-01 — Resolved crate API inventory

- Status: complete (local migration gate; push held)
- Depends on: [M35-01C](M35-01C-c-hir-documentation.md)
- Parent: [M35-01D](M35-01D-c-files-and-visibility.md)
- Contract: [crate boundaries](../../specification/typed-generation/languages/c/rust-hir-files-and-visibility.md)

## Goal

Preserve compiler-resolved public bindings, including aliases, without treating
declared pub spelling as proof of external reachability.

## Definition of done

- Add target-independent typed export metadata using compiler declaration and
  crate identities. An export graph distinguishes module edges from item edges
  and records the Rust namespace of each binding.
- Read resolved compiler module children after full analysis; do not scan source
  text for use declarations or reconstruct name resolution.
- Keep public aliases as bindings to the original declaration, not duplicate
  declarations. Preserve private logical ancestry separately from public paths.
- Share one immutable inventory per compiler crate. Traverse each module once;
  cyclic aliases remain finite graph edges, not infinitely expanded path lists.
- Keep dependency-module edges separate rather than flattening foreign crates.
- Bound extraction work and copied names before allocating unbounded metadata.
- Carry the inventory through existing origins/checked C projection; it is
  provenance, not a caller-constructible validity certificate.

## Tests and proof

- Compiler-backed fixtures assert exact root/module/alias/namespace/target
  identities, same-spelled distinct items, raw identifiers and restricted scopes.
- Pub items behind private parents stay hidden unless reachable through an
  actual public re-export; private fields retain their own visibility.
- Public module aliases and cycles do not lose edges or loop during extraction.
- Ordinary source includes/path-selected modules retain declared-input checks.
- Repeat extraction and certification deterministically; mutations cannot alter
  retained shared origins after projection without failing verification.
- Rustfmt, Clippy, Buildifier, documentation checks, independent review and
  full release gate pass. This does not close native API/privacy proof.

## Implementation and proof evidence

- RustCrateExports/RustExportName/RustExportTarget retain a shared finite graph
  in existing RustSourceOrigin. Compiler extraction uses resolved module children,
  not source text or target names; a visited set handles self and two-module
  alias cycles without path expansion. Foreign module/function IDs stay foreign.
- The compiler fixture has six reachable local modules, a private definition
  module, raw-identifier aliases, all three Rust namespaces, public/restricted
  records and fields, and a declared path-selected source module. Assertions
  check exact binding inventories, original ownership and shared Arc identity.
  An alias-only pair beneath the private parent forms its own cycle: neither
  module is a direct public root child, so the test cannot pass by relying on
  ordinary public module traversal. This strengthens a review-identified
  coverage omission; no production traversal fix was needed.
- export_budget_test covers exact binding/name-byte limits, cumulative counts,
  one-over rejection and checked-arithmetic overflow. These are extraction
  limits, not claims about rustc's own memory or target output resources.
- export_graph_negative_test checks E0603, E0616 and E0365 privacy diagnostics
  and undeclared public-module inputs, including no-create/no-overwrite controls.
- Four shared graph metadata mutations reject changed root ownership, missing
  modules, retargeted bindings and changed namespaces. Review requested an
  unchanged reconstruction control; that control now requires exact package
  equality and successful verification before the negative cases count as proof.
- Linux/Bazel `cfc4abb4-afa1-4813-b8e5-fff91e228715` passed five existing
  provenance/documentation/lint targets with the new inventory. Initial graph
  invocation `94ff3a49-8413-4013-b0fd-eb8803a1eb3b` passed six targets;
  expanded `6ce9f776-91bb-49ba-b44a-9f46843efbaf` and
  `75cfec44-bc2b-4d39-8282-55247d122cfa` each passed seven. The native graph
  matrix compares 8,204 inputs with Rust using GCC/Zig and ASan/UBSan at O0/O2.
- `1fdde665-2760-4014-8a51-17a94f4efc7d` passed all seven targets with the
  unchanged-rebuild positive control. Independent review found no core
  production defects.
- Fresh Sol Extra High independent review found no further core errors after
  the alias-only strengthening. Primitive-type re-export targets are outside
  the declaration-ID inventory and diagnose explicitly; supporting them later
  requires a typed target variant, not a fabricated declaration identity.
- Full Linux/Bazel gate `f7aa81b6-c9bb-4f99-a036-f57ba18f94f4` passed all
  296 test targets (69 executed; remaining results cached). This includes every
  C capacity partition, Rustfmt, Clippy, Buildifier and the existing release
  gate. No tests were disabled. The generated graph fixture was copied to the
  ignored local output/export_graph.c and compared byte-for-byte with Bazel's
  artifact. Push remains held until the C/Java migration gate is green.
