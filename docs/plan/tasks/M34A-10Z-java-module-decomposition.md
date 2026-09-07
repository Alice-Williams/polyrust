# M34A-10Z — Decompose Java implementation modules

- Status: complete
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

### First extraction review and follow-up

The fresh Sol Extra High review found no changed generated behavior or lost
tests. All five architecture/policy findings were accepted: separate float
and list families, group equality semantics under equality, separate statement
builders from expression builders, use explicit imports and file-local helper
privacy, and fail closed when recursive source discovery fails partway through.
The last finding has an injected-failure regression in the release gate.

The follow-up also extracts the AST models, expression/type checks, lexical
flow, constructor assignment analysis, declaration grammar, conformance,
privileged runtime literals, and file verification into focused modules. The
public AST names remain explicitly re-exported; verifier state is not public.
All 39 AST verifier tests are retained in focused test-only modules, including
the compiler oracle. The complete Java Rust test target still runs 86 tests.
The 33 focused Java/policy/lint targets pass after these changes; final full
integration and a fresh review are required before this follow-up is closed.

The second review's remaining coupling findings were accepted: equality-bearing
record assembly now belongs to `equality::record_with_equality`, and member
builders are separate from declaration metadata builders. AST test children
also use explicit imports; two genuinely file-local AST methods became private.

Two suggested privacy changes were rejected after checking test callers:
`validated_result_type` is directly exercised by `tests/runtime.rs`, and
`render_switch_pattern` by `tests/render.rs`. Their `pub(super)` visibility is
needed by those focused sibling tests; it exposes neither helper outside its
private parent module. Making them private fails compilation rather than
strengthening a public boundary. The production/test split is preserved.

### Complete extraction integration evidence

- AST, dialect, and lowering orchestration now have responsibility-focused
  child modules. All Java production files are at most 506 lines; the one
  six-line soft-limit exception is the cohesive AST expression verifier.
- Explicit imports expose sibling dependencies. File-local helper privacy was
  checked against production and direct test callers; no safe public API was
  widened. All 86 Java Rust tests, including the 39 AST tests, remain present.
- Current local proof: all 308 tests in the tracked Bazel graph and all 245
  release-gate tests pass, including native Java, Rustfmt, strict Clippy,
  Buildifier, source policy, and the new partial-discovery failure regression.
- Deterministic conformance: 50 cases and one portable test agree between the
  evaluator and all eight targets; repeated manifests are byte-identical.
- The ownership contract is documented in the Java language specification.
  Independent crate extraction is deferred: these subsystems currently share
  typed AST/catalogue contracts, and module extraction alone is not a separate
  Rust compilation-cache boundary. The test-only source split is effective now.
- Final fresh uncapped Sol Extra High review found no remaining core errors.
  It independently confirmed complete dialect/lowering inventories, unchanged
  test identities, preserved API/privacy, and recursive policy coverage.
  M34A-10X and M34A-10Y remain separate blockers to overall Java completion.
