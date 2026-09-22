# M35-03A-02S-01 — Independent wrapping-multiplication oracle

- Status: complete
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [02R](M35-03A-02R-wrapping-subtraction.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-multiplication.md)

## Contract

Compute signed modulo-2^32/2^64 products with unbounded integers independently
of native/generated results. Reuse the reviewed signed-pair inputs, but add
multiplication-specific cross-products and boundaries where required. Pin corpus
size, distinctness and full-width coverage. Keep support test-only and cached
separately from compiler/target changes.

## Definition of done and tests

Known extrema, MIN*-1, zero/one/minus-one identities, commutativity, range and
modular congruence pass. Exercise both mathematical overflow directions, every
power-of-two cross-product, and products around signed and unsigned width limits.
Detect saturation, addition, carryless multiplication, narrowed operands and
narrowed results independently at both widths. Pinned native Rust wrapping_mul
must agree under O0/checks-on and O2/checks-off. Rustfmt, Clippy, buildifier and
the full gate pass; independent review is clean. No target/source admission.

## Test evidence

The oracle computes unbounded products independently, sharing only input/domain
helpers with addition. Its pinned 34,546 distinct cases add all signed power-of-two
cross-products and neighbours of signed/unsigned square-root overflow boundaries.
Both overflow directions, full-width factors, extrema, range/congruence, zero/one/
minus-one identities, commutativity and modular distributivity pass. Saturation,
addition, carryless multiplication, narrowed operands and narrowed results differ
from truth independently at both widths.

Native Rust agrees under O0/checks-on and O2/checks-off. Focused oracle, Clippy,
rustfmt, buildifier and docs gates pass (fbb6fe21-e997-4ea6-abdf-5595c413632b).
Tree 7b4fc6f9df747018fd0b1547d1718d330c3bae3f passes all 948 release/lint
targets, d79ba42f-1f3e-41d1-a941-be66fb6f00fc (10 executed, 938 cached).
All 389 previous generated files and 38 unrelated WIP hashes match. No compiler
admission, production target mapping, renderer or legacy gate changed.

Independent whole-scope Sol Extra High review is clean: no core defects or proof
gaps. It independently counted 10,169 i32 and 24,377 i64 cases, verified retention
of every earlier input and every signed power cross-product/boundary neighbour,
and confirmed thousands of witnesses for each fault at both widths. Both one-sided
operand-narrowing faults are also observable in the corpus.

Optional explicit all-sign neighbourhood assertions and separately named one-sided
narrowing models are deferred: all such inputs are already present, every input
is compared with both native builds, inventory size is pinned, and the independent
review checked those specific memberships and fault sensitivity. They are useful
future diagnostic refinements, not missing acceptance evidence or production fixes.
The final documentation-only closure is gated again before separate commit/push.
