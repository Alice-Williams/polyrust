# M34A-11-04 — Certify C packages and render structurally

- Status: planned
- Depends on: M34A-11-03

## Goal

Connect the C post-link checker to the shared opaque certificate and total renderer.

## Definition of done

- Implement final C translation-unit checking and the sealed certified renderer adapter.
- Render declarator nesting, expressions, statements, definitions, guards, includes and literals through exhaustive C-owned enums.
- No production source strings/templates, renderer validation or helper discovery; runtime and user items share certification.
- Add linked target resource/platform validation and explicit CResourceError without imposing portable API arity caps.
- Implement c/call-stack-resources.md: actual native call graph, conservative
  automatic-frame/path accounting, explicit entry/callback preconditions and
  measured supported-build bounds before admitting any resource certificate.
- Create a certified target-AST compiler corpus before admitting portable capability mappings.

## Tests and proof

- Add c_structural_format_test, c_grammar_inventory_test, c_ast_compiler_oracle_test and
  c_resource_probe_test with the exact command/negative-shape contracts in
  c/platform-and-proof.md. Record measured numeric policy limits and both
  compiler boundary probes before exposing the capacity adapter.

- Mechanically inventory every variant in the C closed grammar specification
  against exhaustive Rust matches and constructor/mutation/native-oracle cases.

- Compile-fail all wrong-phase/certificate construction/mutation paths.
- Deterministic structured mutation corpus: every admitted package compiles with pinned C17 and strict diagnostics; independent native negatives genuinely fail.
- Declarator precedence matrix, control/zero/hex literal escaping, header standalone/repeated/include-order and separately linked consumers.
- Counted-while exact-count/Continue/nested-switch compiler cases from
  c/counted-loops.md; formatting cannot insert absent initialization or updates.
- Platform assertions and positive/negative resource-boundary probes; rendering/source policies; tracked/release gates.
- Long finite call DAGs, large automatic frames, worst closed dispatch targets,
  exact stack budget/one-over controls and missing-edge/underestimated-frame
  mutations; native controlled-stack evidence for both compilers/opts/sanitizers.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-04 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
