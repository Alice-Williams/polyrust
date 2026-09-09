# M34A-11-02D-04C-02B — Coupled numeric memory

- Status: complete
- Depends on: M34A-11-02D-04C-02A

## Goal

Compose actual numeric writes and reads with authenticated memory provenance
without laundering arithmetic loss through a heap field or indirect alias.

## Implementation sequence

The [numeric-memory specification](../../specification/typed-generation/languages/c/numeric-memory-proof.md)
defines the composition boundary.

1. Add a private resolved-place interface to the existing numeric engine, keeping
   its standalone path/diagnostic behavior. Reuse its arithmetic, conversions,
   comparisons, loss tracking, loop narrowing and widening rules.
2. Carry storage and numeric states together over the actual contextual graph.
   Resolve indirect locations against the same incoming state used to evaluate
   numeric operands; keep actual call/point and original allocation-byte identity.
3. Compose exact/ambiguous writes, reads, aggregate copies, union changes, joins,
   allocation activation and release with numeric history and guard invalidation.
4. Strictly replay converged reachable states and check every obligation before
   diagnostic success. No provisional state or caller-provided resolver can
   authorize rendering. Preserve the standalone numeric and index gates.
5. Add positive/rejected memory-history and stale-observation controls, run the
   full cached gates and the independent review loop, then commit/push.

## Definition of done

- Share existing numeric transfer rules with the storage analysis; do not
  duplicate arithmetic semantics or rewrite the source AST to obtain evidence.
- Retain written numeric domains and arithmetic/call/global loss history across
  scalar and aggregate copies, aliases, joins, lifetime expiry and selected reads.
- Discharge a memory-read obligation only from its actual initialized storage
  and actual reaching writes. Initialization alone cannot erase numeric history.
- Actual later comparisons refine only current observations; mutation and release
  invalidate dependent relations. No optimistic circular proof is admitted.
- Keep standalone numeric diagnostics and actual immutable site authentication.

## Tests and proof

- Useful checked heap size fields and indirect scalar arithmetic/guards.
- Wrapped size stored/read/copied through heap and aggregate paths still rejects;
  stale guards, changed aliases, ambiguous writes and release cannot retain facts.
- Differential transfer-kernel controls, private boundary checks and full cached
  focused/tracked/release/lint/conformance gates plus uncapped independent review.

## Commit gate

Commit/push only with exact evidence. Dynamic extents remain 02C.

## Implementation checkpoint

The combined worklist now uses the existing numeric transfer/refinement kernel
through a private memory resolver. Focused modules own resolution, paired
actions/edges, convergence and strict replay. Standalone numeric/index entry
points remain separate; neither produces a renderer certificate.

Evidence includes exact heap/automatic aliases, ABI-compatible scalar spellings,
record/union copies, sibling-field writes, ambiguous array reads/writes, guards
and mutations, lifetime/copy behavior, call-site byte snapshots, loop widening,
private actual-site reconstruction and 210 direct-versus-heap arithmetic cases.

## Completion evidence (2026-09-09)

- One product worklist carries memory and numeric state at each actual control
  point. Shared arithmetic/refinement code resolves reads, writes, copies and
  guard terms through the same private memory snapshot. Strict replay checks
  reachable actions and edges before resource-exit obligations.
- Exact writes preserve value/loss lineage and invalidate dependent guards.
  Disjoint fields retain values; overlapping union facts and ambiguous writes
  cannot retain stale exact observations. Interval numeric reads stay unproved.
- Actual allocation requests retain the original operand and authenticated
  graph/call/point. Same-origin joins retain conservative byte ranges; release
  and scope expiry remove dead numeric roots without clearing copied losses.
- Regression development caught and repaired null-branch selection disagreement
  between the two analyses. The existing formerly fail-closed clean heap-size
  test now passes; its wrapped counterpart still rejects. No numeric transfer
  algebra was duplicated or executable source/AST rewritten to obtain evidence.
- Focused invocation 30e8383f-7a64-4ae7-8335-42cdad9c2c5a passed all 440 C
  units plus typed compile-fail, Clippy, documentation and Buildifier.
- Tracked invocation 33abcb1d-03fc-469c-9fc9-f3a0233c9a27 passed 445 rules /
  320 tests. Release invocation 5f9c9c2d-52b2-4f9b-9759-4a421f207fc7 passed
  all 257 tests.
- Conformance invocation ff9ec53c-28dd-4d84-99cb-3b10a1783b42 passed 50
  cases plus one portable test: evaluator and eight targets agree, with
  byte-identical repeated manifests. C conformance still uses legacy emission.
- An independent Sol Extra High reviewer directly inspected every changed/new
  production, test, plan and spec file and required prerequisites. The uncapped
  review returned PASS: no substantive scoped defect or required proof gap.
  It specifically checked poisoning/strict replay, alias and union updates,
  original byte requests, counted-loop composition, release/reactivation and
  standalone gates. No finding was dismissed or left unfixed.
- The production/test/spec tree stayed frozen through the review and final
  gates. Only closure documentation changed afterward; documentation/Buildifier
  are rerun before commit. Dynamic buffers remain 02C, owner/call contracts stay
  later, and neither parent 02 nor the C rendering migration is complete.
