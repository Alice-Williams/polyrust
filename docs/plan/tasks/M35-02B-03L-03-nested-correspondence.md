# M35-02B-03L-03 — Complete nested-record ownership correspondence

- Status: complete
- Parent: [M35-02B-03L](M35-02B-03L-nested-owned-records.md)
- Depends on: M35-02B-03L-02
- Specification: [nested correspondence](../../specification/typed-generation/languages/c/rust-nested-correspondence.md)

## Contract

Freeze a bounded source grammar from L-01 observations, then certify complete
canonical source/normal-MIR correspondence using L-02 operation inputs. Start
with the observed two-level tree and make wider depth or combinations explicit
extensions, not accidental admission. Scalar constructors remain anchored in
distinct immutable parameters; final result reads an authenticated live leaf.

## Definition of done and tests

- Every compiler Box/record local, constructor, staging move, aggregate and
  nested field path is accounted for; matching type/field/drop counts alone
  cannot assign source ownership.
- Whole-inner moves transfer every remaining leaf to the new binding. Partial
  nested moves transfer only the selected leaf. Canonical declaration/source
  order and full path types are retained, including intermediate records.
- Every normal drop maps to the correct remaining leaf or intact Inner group
  in recursive declaration and lexical order. The final read and full Return
  locations are authenticated.
- Ordinary consumers assert exact source operation, staging, aggregate, nested
  move, read and cleanup projections without reading private relation state.
- Coherent wrong-owner/wrong-depth/field-type/nominal/aggregate/order/whole-move
  and missing/double/stale cleanup mutations reject. Include shadowing,
  reversed initializers, multiple partial moves and actual whole-inner moves.
- Private query-only input, wrong certificate erasure and arbitrary-MIR injection
  fail exact compile-negative checks; invalid/unsupported Rust publishes nothing.
- Full isolated historical C/Java/native/lint gates and fresh broad review pass
  before documented commit/push. Conditional initialization remains 03M.

## Implementation

The private query-only body reader now maps canonical HIR to constructor,
aggregate staging, whole-inner/leaf movements, final scalar read and exact
normal cleanup events. Intact inner records retain one grouped MIR Drop,
covering both constructor-rooted leaves; partial records retain leaf Drops.
No backend heap translation is enabled by this checkpoint.

Fourteen fixtures cover both leaves, spare/all-leaf extraction, reverse
extraction order, whole-inner multiple extraction, unopened whole-inner
cleanup, shadowed bindings, explicit return, permuted parameter anchors and
same-shaped records with different nominal identities. Three invalid Rust,
ten unsupported-source and one empty-inventory control must reject. Five
exact API compile-negative tests protect private evidence and query-only
admission. A private corruption suite exercises every constructor, both
aggregates, every nested path level, grouped/leaf cleanup and full flow;
coherent nominal substitutions reject while complete local-number bijections
remain accepted. Ordinary consumers inspect source/parameter/type/location
projections without accessing private matching state.

## Closure evidence

- Exact staged tree `32cf9fe48ecd0762c57e3feeba2efbe3c5675077` passed
  isolated gate `730fb927-2a48-4a99-b79e-e9b1ce1293df`: 508/508 tests,
  644 targets, 61.433 seconds. All 2,358 Git blobs and executable modes were
  verified before extraction; test caching remained enabled.
- Fourteen bodies reject 1,727 non-vacuous MIR corruptions and fourteen
  canonical source-owner substitutions. Fourteen complete owner-local
  bijections remain accepted. Three invalid Rust, ten unsupported source,
  one empty-inventory and five exact API-negative controls pass.
- A fresh independent Sol Extra High review inspected the exact staged
  implementation, relevant shared helpers, all fixtures and test suites,
  API contracts, Bazel wiring and documentation. It found no core defects.
- During a service outage, a separate partial review proposed adding asserts
  around consumer `before` calls. We disagreed after inspecting the exact
  helper: the independent test Trace returns unit and asserts internally,
  unlike the production predicate. That finding was explicitly retracted;
  no code change was appropriate. The partial review did not close this task.
- Final documentation-inclusive tree and gate are recorded in the commit.
  All twenty preserved unrelated-file hashes match. This completes L's
  bounded nested-record evidence, not required conditional work 03M or C/Java
  heap generation.
