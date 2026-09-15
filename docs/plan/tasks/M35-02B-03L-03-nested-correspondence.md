# M35-02B-03L-03 — Complete nested-record ownership correspondence

- Status: planned
- Parent: [M35-02B-03L](M35-02B-03L-nested-owned-records.md)
- Depends on: M35-02B-03L-02

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
- Every normal drop maps to the correct remaining leaf in recursive declaration
  and lexical order. The final read and full Return locations are authenticated.
- Ordinary consumers assert exact source operation, staging, aggregate, nested
  move, read and cleanup projections without reading private relation state.
- Coherent wrong-owner/wrong-depth/field-type/nominal/aggregate/order/whole-move
  and missing/double/stale cleanup mutations reject. Include shadowing,
  reversed initializers, multiple partial moves and actual whole-inner moves.
- Private query-only input, wrong certificate erasure and arbitrary-MIR injection
  fail exact compile-negative checks; invalid/unsupported Rust publishes nothing.
- Full isolated historical C/Java/native/lint gates and fresh broad review pass
  before documented commit/push. Conditional initialization remains 03M.
