# M35-03A-02Q-02 — C wrapping-addition safety foundation

- Status: complete
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [oracle](M35-03A-02Q-01-addition-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-addition.md)

## Contract

Certify exact-width unsigned addition and guarded signed normalization using
normal typed nodes, without widening source admission. Preserve existing numeric
obligations: no signed overflow, out-of-range signed casts or misuse of wrapping
values as allocation/extent proofs.

## Definition of done and tests

Both widths certify valid constructions; unsafe missing/reversed/wrong guards,
wrong values, signed-add and unguarded-cast mutants reject. Typed shape checks
catch missing I32 normalization and safe semantic substitutions. Separate
GCC14/Zig O0/O2 plus GCC UBSan consumers match independent modular truth.
Exact imports, original callable authority, ABI headers and resource bounds hold;
no runtime/helper/math-library artifact appears. Previous range/ownership gates,
full isolated release/lint gate, fresh clean review and separate commit/push.

## Implementation and evidence

The target uses existing unsigned range transfer and conditional refinement to
prove both casts and the signed subtraction. Numeric flow, range transfer and
allocation/extent rules are unchanged. Shared profile admission is split into
focused package, expression and integer-category modules. Standard unsigned
type bindings, literal rendering, source-derived ABI queries and closed-call
effect traversal are now covered alongside the original signed API boundary.

Tests exercise each width independently, including missing/reversed/too-high/
too-low/wrong-value guards, signed addition and unguarded casts. Separate shape
controls detect safe wrong-operand/result substitutions and missing I32 result
normalization. Recursive unsigned children cannot hide an unsupported operation
or exceed the existing depth budget.

The native target harness compiles certified producer/forwarding-consumer
packages separately under GCC14 and Zig O0/O2 plus GCC UBSan. It compares both
owners over the independent 15,790-pair corpus, detects two typed safe-fault
packages, and exercises full U32/U64 maximum literals through a modulo identity.
No compiler-source admission is enabled by this target-only checkpoint.

The full-gate and independent-review receipts below close this target checkpoint.

## Release-gate receipt

Exact amended tree 82a51c6c1d2f0a25aa7324d43f8b44bd6250d2fd passes all
906 Linux Bazel release/lint targets under
83ff7455-5678-4e32-afad-d2a4320ad35b (12 executed, 894 cached).
This includes 817 passing C unit tests, all seven new target tests, native
maximum-literal and safe-fault controls, Clippy/rustfmt/buildifier/source-policy,
and the separate capacity/storage stress gates. All 355 older generated files
and 38 preserved ownership/adjacent WIP hashes match their pre-checkpoint values.

The preceding full gate passed 905 targets and identified only the missing exact
test-harness source-policy entry; its correction retains adjacent-path rejection
tests. No production template exception or test disabling was introduced.

## Independent review

Sol Extra High independently reviewed final tree
019eef1106aaaa20b26de5725234404362c9bb68 against 4719d41 and found no core
defects across typing, range refinement, UB avoidance, dependencies/imports,
ABI assertions, rendering, authority, closed effects, resources, mutations or
native oracle evidence. Its only wording correction was accepted: layout
assertions reside in the public header and guard implementation and clients
through inclusion, rather than being duplicated in both generated files.
The correction and final evidence are documentation/comment-only relative to
the passing 82a51 tree. Java and compiler-source admission remain deferred.
