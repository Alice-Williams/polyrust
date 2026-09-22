# M35-03A-02Q-03 — Java wrapping-addition foundation

- Status: complete
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [C foundation](M35-03A-02Q-02-c-addition.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-addition.md)

## Contract

Admit structural Add for exact primitive Int/Long operands and result with
Additive precedence. Keep existing Double arithmetic unchanged; do not admit
other integer operators, boxing, mixed widths or helper calls.

## Definition of done and tests

Typed shape and malformed type/precedence/operator controls pass. Strict
separate Java21 producer/client compilation matches independent signed modular
truth at both widths; wrong-width/saturation/operation controls are detected.
Original dependencies, API/resource/byte accounting and no-runtime inventory
hold. No frontend admission change. Full isolated release/lint gate, fresh clean
review and separate commit/push.

## Implementation and proof

The dependency body reader has a separate exact Int/Long Add arm; the Double
arithmetic arm and structural renderer are unchanged. Existing reader-level
cross-product tests now include that one admitted integer operation. Focused
public-boundary tests cover both widths, every precedence, wrong operand/result
types, boxing, nested unsupported operators/casts, traversal depth, missing
original import registration and wrong arity on either operand.

The native fixture builds separate original producer and forwarding packages.
Strict Java21 compilation and independent modular truth cover all 15,790 input
pairs at each entry point (31,580 observations per run). Five compiling controls
exercise saturation, carryless addition, subtraction, narrowing and a duplicated
operand. Package inventory, original dependencies, call height and source-byte
bounds are checked. These are target-only fixtures, not Rust compiler witnesses.

Focused gate passed on candidate `95464a6433205627af810f4dc8a5f06be61bdf77`
(invocation `3949dc94-a546-4cdf-8ad0-c4e26cbc5dae`). Source admission remains
deferred to 02Q-04.

## Release gate and independent review

Amended tree `7157f8224f8b5054714ebc87b71c545adef876a2` passes all 906
Linux Bazel release/lint targets under invocation
`bb008d83-c22e-4c2e-b565-ee860853e017` (9 executed, 897 cached), including all
395 Java unit tests, native Java21 checks, Clippy/rustfmt/buildifier and policy
gates. All 355 earlier generated bundle files and 38 preserved WIP hashes match.

The preceding gate identified one obsolete assertion that all integer Add nodes
must reject. The corrected test proves Add acceptance while retaining Subtract
rejection; direct reader controls additionally reject boxed Int/Long and String
concatenation. No production correction or test disabling was required.

Sol Extra High independently reviewed the original candidate and amended delta,
finding no remaining core defects across exact typing/precedence, recursive
authority/budgets, dependencies, independent native oracle/fault controls and
resource/byte accounting. No extra feature was substituted for an error fix.
