# M34A-14 — Enforce compiler-derived JavaScript

- Status: planned
- Depends on: M34A-13

## Goal

Make every executable JavaScript byte a deterministic product of the pinned
TypeScript compiler, with no second semantic lowerer or renderer.

## Definition of done

- JavaScript owns no semantic CoreIR lowerer, executable AST, independent
  capability registry, runtime catalogue, or executable Handlebars template.
- The derivation stage accepts only the verified rendered TypeScript package
  plus pinned compiler identity/options.
- Every executable TypeScript input has exactly one expected compiler output;
  no extra/missing executable JavaScript is accepted.
- Runtime/helper topology and runtime-bearing dependencies are the exact
  post-erasure TypeScript subset.
- Package metadata/entry points are typed and point only at derived files.
- Clean compiler output hashes match the packaged JavaScript byte-for-byte and
  deliberate edits fail.
- Standalone JavaScript packages contain no TypeScript source/compiler runtime
  dependency and pass native tests.
- Legacy paired JavaScript strings, checked-in editable runtime output, and
  post-compile semantic rewrites are deleted.
- The JavaScript compliance row moves to **Pass** with exact evidence.

## Tests

- `bazel test //crates/backend-typescript:javascript_all --nocache_test_results --test_output=errors`
- Pinned compiler/options, missing/extra output, tamper/hash, helper/module
  parity, entry-point, and three-clean-compilation determinism tests.
- Standalone npm package tests, interface dispatch after erasure, and all
  M17-M33 JavaScript historical port targets.

## Commit gate

Commit and push `M34A-14: derive JavaScript from TypeScript` only after clean
derivation is byte-identical and standalone JavaScript gates pass.
