# Layer 9: manifest assembly and verification

- Status: normative
- Input: `RenderedPackage`
- Output: validated `OutputManifest` and permanent compliance evidence

## Manifest assembly

Only the shared compiler adapter constructs `OutputManifest`. It validates:

- unique normalized relative paths;
- source/text/byte role agreement;
- deterministic file ordering;
- per-file and package size limits;
- declared external dependencies;
- injected helper reports;
- target/plugin/IR versions;
- generation options;
- no partial files after failure; and
- stable content hashes for diagnostics and cross-host comparison.

The manifest does not write to disk. CLI materialization remains a separate
atomic operation.

## Architecture evidence

Every generation can produce test-only canonical phase evidence:

- CoreIR dump;
- unresolved target AST dump;
- support-decision report;
- resolved symbol/import/helper/file report;
- resolved target AST dump;
- syntax-certificate and structural-renderer coverage report;
- rendered file hashes; and
- final manifest dump.

These artifacts contain no timestamps, absolute paths, addresses, random IDs,
or unordered-map output.

## Shared test pyramid

### Static and compile-fail

- Phase types cannot be crossed.
- Grammar categories cannot be mixed.
- Phantom-typed known operations reject wrong types.
- Incomplete feature/grammar enum matches fail compilation.
- Later-phase constructors are inaccessible.

### Verifier and fault injection

- Every CoreIR, unresolved AST, resolved AST, helper, file graph, renderer, and
  manifest invariant has a deliberate invalid fixture.
- Each dialect receives a mandatory post-link whole-file verification hook.
  It validates constraints which become visible only after runtime helpers,
  split declarations, or other structural items have been assembled into their
  final file; validating the items separately is not sufficient.
- No invalid fixture reaches a later phase.
- No failure writes output.

### Golden and property tests

- Canonical dumps cover every node variant.
- Identifier/literal/path generators cover boundary and hostile values.
- Expression precedence matrices cover every operator pair.
- Repeated phase transformations are byte-identical.

### Native language tests

Every language specification names exact format, lint, compile, negative-type,
native behavior, public-consumer, and sanitizer commands.

### Semantic conformance

Every portable vector passes:

- the reference CoreIR evaluator;
- generated native tests in every requested target; and
- cross-target canonical result comparison.

Binary64 values use exact raw bits. Strings use scalar-aware encodings. Owned
values use deep structural comparisons.

### Real-world differential proof

Each compatibility port:

- pins an MIT or Apache upstream revision and every retained blob;
- defines the exact typed API scope;
- retains or builds an independent upstream oracle;
- uses deterministic boundary and seeded corpora;
- compares behavior losslessly;
- generates every required output;
- documents every CoreIR/language gap filled; and
- completes all local/hosted gates before another repository begins.

## Language migration proof

A backend moves to `Pass` only when:

1. its complete accepted feature surface lowers through its typed AST;
2. no legacy executable fragment/string path remains;
3. imports/includes come only from typed symbol resolution;
4. runtime helpers are structural AST;
5. its opaque render-ready certificate and total structural renderer meet the
   rendering contract, with no executable template path;
6. interface/composition and any target-only adapter inheritance pass;
7. all historical ports pass native and differential evidence unchanged; and
8. the hosted workflow is green on the pushed commit.

Temporary dual paths may exist only inside an in-progress task and cannot be
committed as a completed language migration.

## Historical replay

The final migration gate enumerates every previously completed port rather than
depending on a broad target whose membership could drift. For each port it
records:

- pinned upstream commit/license;
- portable model hash;
- targets generated;
- vector count;
- differential corpus count;
- native targets;
- public consumer/sanitizer targets;
- three-generation hash;
- local gate run; and
- hosted CI run and commit.

## Required repository gates

The final task runs uncached:

- `bazel test //... --nocache_test_results --test_output=errors`;
- `bazel test //:release_gate --nocache_test_results --test_output=errors`;
- Buildifier;
- Rustfmt;
- Clippy with warnings denied;
- dependency-boundary tests;
- source-policy and fault-injection tests;
- every language-native toolchain;
- determinism and cross-host manifest comparison; and
- hosted CI.

The exact test counts are evidence recorded at execution time, not normative
constants.

## Release rule

M34 remains the sole active compatibility port while M34A replaces the
generation architecture. No later repository is selected until:

- all language migrations pass;
- every historical port is replayed;
- M34 is rebuilt and proven on the new architecture;
- documentation is complete;
- changes are committed and pushed; and
- hosted CI is green.
