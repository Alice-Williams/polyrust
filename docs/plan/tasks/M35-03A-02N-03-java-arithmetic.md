# M35-03A-02N-03 — Java binary64 arithmetic target foundation

- Status: complete
- Parent: [binary64 arithmetic](M35-03A-02N-floating-arithmetic.md)
- Depends on: 02N-01 and 02N-02
- Specification: [Java mapping](../../specification/typed-generation/languages/java/rust-floating-arithmetic.md)

## Contract

Admit exact Double binary arithmetic in certified dependency bodies, with
operator-specific precedence and recursively authenticated original operands.
Do not infer integer arithmetic or general library-call permission.

## Definition of done and tests

Java21 strict-lint original/importing packages execute all four operations and
nested expressions against 02N-01 bit/category expectations. Include wrong
type/precedence/operator/authority controls, operand evaluation traces,
value-preserving dropped/duplicated calls, grouping and fused-result controls.
Rendered bytes stay within actual source reservations; existing package bytes,
full Linux release/lint gate and independent review pass. No runtime helper or
new Rust source admission is claimed by this target-only checkpoint.

## Implementation and focused evidence

Dependency-body admission accepts only Add/Subtract/Multiply/Divide with exact
primitive Double result/operands and matching Additive/Multiplicative precedence.
It visits both operands using the existing authority/budget checks. The source
reservation walker also traverses both children. Rendering is unchanged:
every binary expression remains fully parenthesized structural Java syntax.
No runtime class, standard-library helper or production import is introduced.

The target-only proof has three original owner certificates: leaf identity
methods, a middle arithmetic package and a forwarding root. Its six functions
exercise each operator, (left + right) * right and (left * right) + (-1).
Both call operands are stored once in ordered final locals. Actual combined
rendered source bytes remain within each owner's certified reservation.

Focused tree 62316bd24a00a6279b0b1d9b54823d1eadd3f11a passed 11 selected
Rust tests in invocation c9e2e886-617b-4ee6-b618-1e877d76c089 (84.158 seconds;
native suite 44.77 seconds). Six tests are new; the selection also includes
existing arithmetic regression tests.

- 960 direct private-reader combinations cover operand/result types,
  operators and precedence, independently of the public verifier.
- Public package tests cover exact successful cases and wrong type,
  precedence, remainder, nested remainder/cast operands, unregistered calls
  and wrong arities. Existing integer arithmetic rejection remains intact.
- 3,217 pairs yield 38,604 independently expected binary64 bit/NaN-category
  observations in each of ten Java21 native variants. Owners are separately
  compiled with --release 21, -Xlint:all, -Werror, -implicit:none and an empty
  source path; the consumer sees their compiled classes.
- Five value faults (operator, operand reversal, zero sign, grouping, fusion)
  and three value-preserving trace faults (dropped, duplicated, reordered calls)
  all compile and are independently observable. The fusion control uses the
  exact normal pair (1+2^-27, 1-2^-27) and a rational single-round expectation.
- Source-policy exceptions are only the two test-harness paths, with explicit
  copy/adjacent-production/neighboring-name rejection controls.

Independent review of that exact candidate found no core correctness defect
or required-proof gap. It checked production admission/rendering/bounds,
recursive authority, fixture dataflow, oracle independence, all eight faults,
and exact-path policy classification. The full release evidence is recorded below; Rust-source admission remains
the separate 02N-04 task.

## Completion receipt

The reviewed implementation tree 62316bd24a00a6279b0b1d9b54823d1eadd3f11a
passed all 860 Linux Bazel release/lint test targets, invocation
db02c75d-e997-4d58-ba9c-08f45eb9b2c6 (560.448 seconds; 101 executed,
759 cached). This includes Rust formatting/Clippy, Bazel formatting/lint,
Java21 compiler/native checks and source-policy failure controls. No test was
disabled, weakened or waived. All 138 file hashes across 18 older generated
compiler bundles and all 19 preserved ownership-work hashes are unchanged.

The independent reviewer found no core defect or required-proof gap on this
exact implementation. Compiler-integration drafts are excluded from this
checkpoint and do not yet enable new Rust syntax. Publishing remains blocked
by the prior push-approval decision; this is local evidence, not a remote CI claim.
