# M34A-10Z — Decompose Java implementation modules

- Status: in-progress
- Depends on: M34A-10U
- Supports: M34A-10W, M34A-10X, and M34A-10Y

## Goal

Replace oversized implementation files with responsibility-focused modules,
without changing generated behavior or widening safe public AST construction.
Perform this cleanup incrementally before and alongside the remaining Java
correctness work; it does not supersede the totality or strategy blockers.

## Implementation order

1. Split runtime AST construction by runtime family and separate shared typed
   construction utilities. Move runtime tests outside production source sets.
2. Split structural rendering into declarations, statements, expressions,
   names/types, and syntax. Apply renderer policy to every child module.
3. Split target AST models and verification by responsibility, then dialect
   metadata/linking and CoreIR lowering orchestration. Keep private invariants
   private and isolate substantial test fixtures from production sources.
4. Assess actual Bazel action boundaries separately: Rust module files within
   one crate improve navigation but do not create independent compiler caches.
   Do not claim otherwise or introduce cyclic crates just to shrink files.

## Definition of done

- Production Java files aim below 500 lines and none exceeds 1,000 lines.
- File names describe the owned responsibility; no numbered dumping grounds.
- No safe public API widening, raw source escape hatch, or changed generation.
- Structural-renderer policy scans all renderer modules and mutation tests
  prove that moving forbidden constructs into a child cannot evade it.
- Substantive test modules are excluded from production Bazel source sets.

## Tests and checkpoint evidence

- Focused Java tests, Java 21 compilation with strict linting, Rustfmt,
  Clippy, Buildifier, and source-policy tests after each coherent extraction.
- Full tracked Bazel graph, release gate, and deterministic eight-target
  conformance before a commit/push checkpoint.
- Fresh uncapped Sol Extra High review of visibility, source/test boundaries,
  behavior preservation, and policy coverage.
- Record partial checkpoints here; only mark complete after all large modules
  and the final integration/review gates meet the definition of done.

### Runtime and renderer extraction checkpoint

- Runtime construction is split into 17 responsibility-focused modules, at
  most 405 lines each; the runtime entry point is 84 lines.
- Rendering has five structural child modules and a 111-line entry point.
- Runtime and renderer test modules now live under `src/tests/`, excluded by
  the production Rust library's Bazel source set.
- Renderer policy recursively covers children; typed-source policy recursively
  covers all production Java modules. Policy self-tests exercise child paths,
  including Windows paths and nested modules.
- Local evidence: 29 focused Java/policy targets, all 307 tracked-graph tests,
  and all 244 release-gate tests pass. The release gate was run again after
  extending the general typed-source policy to nested modules.
- Deterministic conformance: 50 cases and one portable test agree between the
  evaluator and all eight targets; repeat generation is byte-identical.
- AST, dialect, and lowerer decomposition remains outstanding. This checkpoint
  does not complete M34A-10Z or resolve M34A-10X/M34A-10Y.
