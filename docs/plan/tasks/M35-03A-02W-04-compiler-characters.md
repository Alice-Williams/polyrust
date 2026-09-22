# M35-03A-02W-04 — Checked Rust character source integration

- Status: complete
- Depends on: [Java foundation](M35-03A-02W-03-java-characters.md)
- Specification: [shared](../../specification/typed-generation/rust-character-values.md)

## Contract

Admit canonical rustc Char literals/types through a private checked input
carrying Rust char. Add an executable character-value mapping and distinct
source TypePlan::Char, preserving source identity even where Java uses Int.
Extend immutable places, scalar signatures, direct calls, conditionals and
same-Char equality/ordering exhaustively. Keep constants, casts and methods
outside this increment.

## Definition of done and tests

Use original multi-crate Rust with public/private APIs, documentation, aliases,
named imports and source-owned target packages. Compare native source and C/
Java across the complete scalar corpus, literal boundaries and comparison
pairs; measure original call order and once-only evaluation. Prove exact typed
U32/Int mappings, original declaration/type/owner joins and manifests that
distinguish char from integer and document the valid-scalar foreign-input
domain. Preserve scalar-field record behavior or diagnose unsupported shapes;
do not silently reinterpret source integers as chars. Atomic negatives cover
casts, constants, methods, references, invalid Rust literals and unsupported
forms. Detect actual value/order/type/owner faults. Export real examples,
update partial inventory, preserve old output/WIP, pass full Linux release/lint
and fresh broad review before separate commit/push.

## Implementation map

Preparation inspection identified source-sensitive reverse mappings in Java.
Do not merely add Char to the object-type match and let these reconstruct I32
from Java Int. Both separately reviewed/gated target foundations are complete.

1. Extend the existing private canonical-HIR LiteralInput with a Char(char)
   enum variant and an executable typed literal mapping for each target.
   Reuse Supports<LiteralValues>; do not duplicate the literal capability or
   introduce an untyped capability flag. The checked value is Rust char, never
   a raw integer admitted by a range assertion in the renderer. Preserve the
   original type-checking context and reject adjustments/unsupported shapes.
2. Retain source Char separately in Java representation plans. Parameter
   binding, direct-call arguments/results, record construction and field reads
   must obtain their source plans from original compiler types or retained
   source field plans, not from target JavaType. Target-only signature rendering
   may still inspect Java primitives; it must not create source-type evidence.
3. Extend C literal/type/signature/record/comparison mappings to exact U32.
   Check original source types at joins and callees even where a target type
   currently happens to be unique. Preserve evaluation order through existing
   materialization. Support existing bounded immutable scalar-field records;
   keep direct character references outside this increment and test their
   atomic rejection explicitly.
4. Carry original source scalar signatures, field identities and their enclosing
   record identities to public
   metadata through typed source-owned facts. Java Int alone cannot identify
   Char. Authenticate facts against original compiler declarations in the
   adapter and check their target representation against certified inventory.
   Distinguish source-domain metadata from a target validity certificate.
   Emit a character-aware schema only for affected packages so existing
   generated files remain byte-identical. Document valid-scalar foreign inputs.
5. Build original producer/middle/root Rust fixtures with docs, private helpers,
   aliases, mixed char/i32 signatures and mixed scalar-field records. Prove
   native values at both Rust profiles and against both target implementations.
   Include measured nested operand calls, both conditional branches and field
   construction order, not only literal snapshots.
6. Add typed probes that deliberately substitute I32 for Char while leaving
   Java Int unchanged; wrong source identity must be detected. Also mutate
   original owner/type/signature facts, actual narrowing/order output and
   manifest character annotations. Require positive controls before negative
   controls so rejection cannot be explained by unrelated malformed fixtures.
7. Exercise source constants/casts/methods/references and invalid literals as
   atomic publication negatives. Run producer-change/restored-input cache proof,
   export actual host-readable packages, audit old output/WIP hashes, complete
   fresh broad review and full release/lint gates before commit/push.

The legacy selected-entry harness remains exactly fn(i32) -> i32. Java must
check the original Rust signature, not only the shared Java Int representation;
fn(char) -> char and mixed char/i32 entry signatures are explicit negatives.

Alias coverage means local declaration re-exports and resolved named imports
from original dependency owners. Foreign function/module public re-exports
are currently rejected by the common public-package inventory and are not
silently enabled by character support; keep an explicit rejection control.
The first bundle attempt correctly diagnosed that unsupported fixture shape.

These are integration requirements, not claims that a Java Int or C U32 target
certificate validates the Unicode scalar domain. Neither target foundation
enables arbitrary integer-to-character conversion or Unicode text operations.

## Verification log

- Initial source integration built both compiler adapters and passed the
  original Rust corpus. An unsupported foreign public re-export in the first
  fixture correctly prevented bundle publication; fixtures now use local
  aliases and named imports, with foreign re-exports retained as negatives.
- Snapshot 00d4b981 built both three-owner target bundles; original corpus,
  fixture Clippy and rustfmt passed. Compiler adapter construction itself
  uses the pinned Clippy driver with warnings denied.
- Snapshot 354ba5b9 passed character_source_native_test: 1,116,517 rows and
  19 literal boundaries agree across original Rust O0/O2, generated C with
  GCC/Zig O0/O2 plus UBSan, and Java21 normal/-Xint. All owners compile
  separately. Exact source/target types, aliases, fields, documentation and
  public/private inventory were checked.
- The first full regression gate finished with 1,000/1,021 passing, seven
  build failures and fourteen test failures. It exposed metadata-only
  probe dependency/dead-code boundaries and stale diagnostic text assertions.
  Scoped fixes preserve the existing rejection and atomic-publication checks;
  they await verification in the next snapshot.
- Additional prepared gates cover original-type/field/owner/declaration
  corruption, actual generated traces and compiling width/order faults,
  external visibility, unsupported source boundaries, exported examples and
  producer-change/restored-input cache behavior. These are not yet completion
  evidence. Broad independent review found a missing field-to-record owner
  join and a missing foreign-module re-export rejection case. Both findings
  are accepted: retain typed field owners, reconcile them against certified
  target owners, and test two same-shaped records with swapped metadata both
  before compiler authentication and after it at target reconciliation. Test
  foreign function and module re-exports separately. A second fresh review
  found C direct calls missing the explicit canonical Rust signature join.
  This is accepted too: retain the DefId, join original argument/result types
  and arity, check target representations, and test corrupt source joins without
  changing target signatures. Every C value expression also reconciles its
  returned target type with the original adjusted compiler type. No source-integration
  commit/push is authorized until the remaining evidence and full gate pass.

- Snapshot ce6ffd63 passed the full-domain native test, compiling value/order
  faults, 48 atomic character-boundary cases, original Rust/lints, codegen tests
  and 431 Java unit cases. The metadata probe passed its 24 corruption cases
  before finding a stale expected diagnostic in the foreign-function rejection
  case; the assertion now distinguishes function and module diagnostics.
- All 501 prior generated output hashes and 38 unrelated WIP hashes are
  unchanged. Actual examples are exported to the ignored host directory
  generated/m35-character-source-examples (24 files including original Rust,
  both generated packages and handwritten clients). These artifacts are not
  committed. The previous pushed Java checkpoint's CI run 35751098268 is green.
- The second full gate finished with 1,022/1,024 passing. Its two failures were
  duplicate Java fixture registration under Clippy and a shifted unsupported-
  operator diagnostic in the bit-shift rejection test. One shared fixture
  registration and operator admission before same-type comparison checks fix
  these without relaxing any rejection or target/source type check.
- Snapshot 88969b07 passed all six focused checks: Rust Clippy, native full-
  domain observations, compiling value/order faults, bitwise boundaries,
  character boundaries and source/target authentication. The latter now proves
  30 atomic metadata/call/import faults and eight foreign re-export failures.
  In particular, a Java producer's Char-to-I32 metadata mutation after source
  authentication preserves Java Int signatures but fails the consumer's
  original Rust signature join. That closes a fresh review's evidence finding.
- Final fresh Sol Extra High whole-scope review of 88969b07 reports no actionable
  core findings; the preceding reviewer also reports no unresolved findings.
  The full release/lint gate on that snapshot and the cache-change/restoration
  proof remain pending. No completion or push is claimed yet.

- The third full gate on 88969b07 finished with 1,023/1,024 passing. Only
  java_finite_constants_native_test timed out at its existing 900-second limit
  under four concurrent jobs; no assertion or compiler failure was reported.
  The preceding two-job gate passed that test in 437 seconds. Its isolated
  retry was deliberately interrupted when the user requested Docker shutdown
  for gaming. On explicit resume, restart that test alone and use two jobs
  for the final full gate. Do not disable tests or raise timeouts to close it.
- Resume audit confirms all 501 prior generated hashes and 38 unrelated WIP
  hashes remain unchanged. The normal Git index is empty; nothing from this
  source-integration checkpoint has been pushed.
- The resumed isolated Java finite-constant native test passed in 222.8 seconds
  without code, assertion or timeout changes. The complete two-job release/lint
  gate then passed all 1,024 tests on 88969b070b6e100c410fc6605503a20f2b11a199
  (five executed, 1,019 cached), invocation
  aec0b548-cb97-441d-af1f-ac7a0a59a6bf. The cache-change/restoration proof and
  final documentation closure remain pending.

## Completion evidence

The archive-isolated cache proof on 88969b07 passed every phase. Warm builds
had no target actions and warm native tests were cached. Changing the producer's
maximum scalar from 0x10FFFF to 0x10FFFE reran all seven affected metadata/package
actions without recompiling the adapters. Old truth failed uncached; updated
original Rust/C/Java observations passed. Restoring source and truth recovered
all baseline output hashes and a cached passing native result. Receipts are in
the ignored generated/m35-character-cache-proof directory, including summary,
hashes, build events and action logs.

The completed implementation has all 1,024 release/lint targets passing, clean
fresh whole-scope reviews, 431 Java unit cases, full scalar-domain native proof,
30 atomic type/owner/call/import faults, eight foreign re-export controls and
48 character-source rejection cases. Actual examples are exported to
generated/m35-character-source-examples. All 501 previous generated hashes and
38 unrelated WIP hashes are unchanged. Final documentation closure receives
the full gate again before its separate commit/push. Character constants and
the wider scalar/runtime migration remain separate work; no legacy test or
runtime has been removed or disabled.
