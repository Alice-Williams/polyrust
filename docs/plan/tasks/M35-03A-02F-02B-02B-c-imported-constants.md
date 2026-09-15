# M35-03A-02F-02B-02B — Certified C constant imports

- Status: planned
- Parent: [C constant APIs](M35-03A-02F-02B-02-c-constant-api.md)
- Depends on: M35-03A-02F-02B-02A

## Contract

Finish the dependency boundary in the
[C constant specification](../../specification/typed-generation/languages/c/rust-public-constants.md).
Derive CDependencyConstant only from an immutable producer certificate; register
a consumer-branded CImportedValue retaining exact producer, source identity,
const storage, unqualified scalar read type, literal value, symbol and header.
No name-, JSON- or caller-signature construction may create authority.

Reconstruct complete function/object export inventories, including unreferenced
constants. Support constants-only producer APIs without taking a first function.
Extend context, scalar-effect, resource, source-origin, import and read checking
using those certificates. Foreign objects never become fake owned definitions.

## Definition of done and tests

- Separate producer/consumer GCC/Zig O0/O2 compilation/execution agrees with
  independent exact truth; values-only and mixed dependencies emit each required
  header once while retaining every typed binding.
- Registry scope, same-crate certificate conflicts, output paths, complete
  native symbol collisions, types, values, owners and stale witnesses are tested.
- Compile-negative controls prohibit forged certificates/import witnesses;
  structural and coupled reference/catalogue/export mutations reject.
- Readonly constant effects do not introduce fabricated runtime call frames or
  weaken numeric/storage verification; complete resources remain accounted for.
- Fresh clean review, isolated full Bazel/lint, inspectable ignored examples and
  scoped commit/push. Complete parent02 only after both children pass.

## Boundary

Rust HIR admission and multi-crate metadata publication remain children04/05.
No remaining legacy constant family or runtime is removed by this target step.
