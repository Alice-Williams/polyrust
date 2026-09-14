# M35-01D-03C — Rust public API package mapping

- Status: complete
- Depends on: [M35-01D-03B](M35-01D-03B-c-package-projection.md)
- Parent: [M35-01D-03](M35-01D-03-c-public-packages.md)
- Contract: [public packages](../../specification/typed-generation/languages/c/rust-hir-public-packages.md)

## Goal

Map the admitted compiler public-function graph to checked C files and a typed
API manifest without losing aliases, private ancestry or documentation owners.

## Definition of done

- Package mode maps every admitted public function binding, not only score;
  diagnose every unimplemented public API category instead of omitting it.
- FunctionSignatures handles public roots and reachable private helpers; retain
  the existing selected-entry mode for the single-unit experiment.
- Exact public header/private source placement and bidirectionally verified
  compiler-binding/function/file/linkage manifest.
- Preserve finite module/alias graphs, primary item docs and private/public
  module-documentation placement without duplicated function documentation.
- Produce header/source/manifest together after all checks; reject input before
  creating or replacing output. Save a tested, ignored local example.

## Implementation order

1. Add one explicit source-selection mode to the existing compiler bridge:
   legacy selected entry or complete public API. Read the cached compiler export
   graph and resolve each Value binding to an ordinary local function identity.
   Reject unsupported namespaces/kinds and foreign edges before body lowering.
   Inventory the union of public roots and transitive callees under one budget.
2. Reuse the current capability bindings, Reader and body lowering for both
   modes. Register all functions first. In package mode, exported primaries go
   in the crate-derived reserved header; private primaries and every definition
   go in the implementation. Preserve the existing selected-entry ABI and tests.
3. Expose a read-only typed definition/name view from the certified C package.
   Build the API manifest from the exact compiler binding graph and authenticated
   C function/file/linkage identities, with linker-derived names. Reconstruct
   membership both ways before serialization; never parse rendered source or
   serialize ephemeral registry authentication as a stable identity.
4. Add explicit package CLI output. Finish analysis, target checks, certification,
   manifest validation and rendering before touching destination paths. Publish
   the complete output set; unsupported source must neither create partial files
   nor overwrite an existing package. Keep this separate from C syntax rendering.
5. Add dedicated positive compiler/provenance probes, coordinated manifest
   mutations, negative publication tests and independent native consumers. Run
   the old selected-entry compiler gates alongside the new package proofs.
   Record a fresh independent review and a visible ignored generated example.

## Tests and proof

- Multiple public functions with zero/many/mixed scalar parameters, aliases,
  private-parent exports, all restricted helper visibility forms and modules.
- Independently inspect compiler origins, generated files and manifest; mutate
  missing aliases, widened exports, wrong crate owners and documentation leaks.
- Unsupported public types/macros/foreign APIs and malformed source fail with
  no-create/no-overwrite proof; existing single-unit tests stay enabled.
- Native Rust/C differential and separate-consumer symbol tests; compiler
  contracts, format/lint/docs gates and fresh independent review.

## Implementation evidence

- Public-root selection and shared body assembly reuse all existing executable
  capability bindings. Alias edges deduplicate by exact compiler function ID;
  unsupported public namespaces, types and foreign APIs diagnose before output.
- A certificate-bound C definition view exposes exact function references,
  linker names, defining files and linkage. The API manifest checks complete
  compiler/target membership in both directions and retains no serialized
  registry brand. Its separate 8 MiB metadata policy runs before formatting.
- The compiler graph now retains shared ancestry metadata for public modules,
  including alias-only modules. Paired C docs route those owners through the
  existing typed attachment inventory; legacy selected-entry placement stays
  unchanged. The new fixture proves alias-module docs are not silently lost.
- Package publication stages a new three-file directory after all checks and
  refuses existing destinations. Tests cover deterministic rerenders and 21
  invalid/unsupported cases, each with both missing and sentinel destinations.
- `6680e11d-eb69-4ba6-b2c4-f21fee86cd16`: first real-Rust package/native
  smoke and compiler rustfmt gates pass (2/2).
- `092a09f9-2e06-4f79-a24f-ccdbe2650d1a`: typed manifest mutations,
  negative publication, native package smoke and rustfmt pass (4/4).
- `9107aa59-22c3-4da5-9d1f-c6423cf328b5`: expanded 8,204-input Rust/C
  differential and metadata/negative gates pass. Clippy required a scoped
  expectation for intentional `pub(self)` fixture syntax, and buildifier
  required an argument docstring; both repaired. Full regression/review remains.
- `7394c0b7-fe4d-4923-bb67-dceda06c2d7d`: 61 tests passed; the package
  artifact exposed Bazel's pre-created tree directory. The action now invokes
  the no-overwrite CLI in a fresh staging path and transfers the complete files
  into the empty Bazel-owned artifact. No CLI publication protection was relaxed.
- Independent review identified incomplete module-ancestry membership checking.
  Paired projection now requires exactly the export graph's module keys. Positive
  alias-only docs and coordinated missing-root/missing-alias/extra/wrong-owner
  mutations cover the repair. The reviewer's follow-up found both repairs sound.
- `7d2c1a3b-f9d4-4507-9be4-f19ec8e35943`: the full 99-target frontend,
  C, shared and Java regression gate passes, 62/62 test targets, including all
  676 ordinary C tests and all five capacity partitions. No tests were disabled.
- Tested declared output was copied to the ignored
  `experiments/rustc-frontend/output/public-package` directory and matched by
  recursive byte comparison. SHA-256: api.json
  `3c8fbd072adc9fe6325e86dfcb3d786b8bcbfc2454c8bbca6cfe2e07f50fc451`;
  C source `61b79d3e8fc723689bbb827936a76b3365a3067dd1e4f0f6c10b2b58746ff889`;
  public header `d1f797e525985a8e5fe49d3a49bab788858703289673547d1183116a3c91e2bf`.
  Fresh blind broad Sol Extra High review then examined the complete compiler,
  target, metadata, publication, tests and visible artifact and found no core
  defects. Documentation gate `c0862a11-ff78-43fd-8a7e-4ddda84dc253` passes.
  Continue with the integrated release proof in M35-01D-03D; pushes remain held.
