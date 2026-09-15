# M35-03A-02F-02B-02A — Owned readonly C scalar objects

- Status: complete
- Parent: [C constant APIs](M35-03A-02F-02B-02-c-constant-api.md)
- Depends on: M35-03A-02F-02B-01

## Contract

Implement owned declaration, definition, read, projection and certification
under the [C constant specification](../../specification/typed-generation/languages/c/rust-public-constants.md).
Keep CObjectRef/CObjectType/CConstness and existing initializer/literal nodes.
Add a typed global value binding to the shared projection and preserve exact
primary header/implementation identity. No imported object witness yet.

Only literal-initialized const bool/i32/i64 objects join the bounded profile.
Admit constants-only public header/source pairs without synthetic functions.
Retain all complete context, static initialization, numeric, storage, resource,
platform, namespace and original-projection checks. Do not enable mutable
statics, aggregate globals, address borrowing or general runtime initializers.

## Definition of done and tests

- Registered scalar declarations/definitions project and certify; global reads
  preserve readonly place type and the existing unqualified scalar value type.
- Constants-only and mixed packages render normal headers/sources with one
  declaration/definition, standard includes and exact declaration ownership.
- Strict independently compiled GCC/Zig O0/O2 consumers prove both booleans,
  i32/i64 extremes and wide values; native assignment to each const fails.
- Missing/duplicate definitions, linkage/file/type/qualifier/value-node changes,
  unsupported globals and coupled projection tampering reject.
- Resource and documentation routing include objects and reads. No custom
  runtime/source fragments are emitted. Existing source/function gates remain.
- Dedicated focused fixtures/tests, clean fresh review, full isolated Bazel/lint
  gate and scoped commit/push. Export inspectable ignored examples.

## Boundary

Dependency APIs reject constant-bearing packages until child02B, including
synthesized constants absent from the compiler export map. They must not claim
a function-only public inventory for a header/object that also emits globals.
This step proves target package generation, not HIR source admission, producer
metadata or complete public-constant parity.

## Implementation and proof inventory

- CValueBinding::Global retains the registered CObjectRef. The shared symbol
  inventory retains its const-qualified storage type and exported visibility;
  expression reads use the existing unqualified scalar conversion.
- Unit projection selects both referenced objects and objects defined in the
  current implementation. Primary declaration ownership stays on the header;
  the shared linker produces the source-to-header dependency.
- A dedicated bounded-profile module admits only public-header const bool/i32/i64
  objects with exact scalar literal initializers. Full contextual, storage,
  numeric, platform and resource checks remain in the certification path.
- Documentation follows object origins. Constants-only packages have no
  function frames or automatic storage; formatted bytes remain bounded.
- Focused fixture/test modules cover constants-only and mixed packages,
  reversed file order, typed reads, matching source/header bindings, documentation,
  resources, missing/duplicate declarations and definitions, wrong registry/
  type/file/linkage, mutable/unsigned/aggregate objects, forbidden initializers,
  addresses and writes, and metadata/binding/primary-placement mutations.
- The native proof separately compiles implementation and consumer under GCC
  14.2 and Zig at O0/O2. Its independent truth covers both booleans, signed32/64
  extremes, an integer beyond binary64's exact range, and an ordinary value.
  Every object assignment fails compilation. Replacing the ordinary value in
  generated source with 17 still compiles but fails the independent consumer.
- Generated examples and consumers are exported as Bazel undeclared outputs;
  an inspected host copy is in ignored generated/examples/c-owned-constants-419a55e.

## Verification and review evidence

Tree 61537ec6b8b86aa727a821e53dd97fc5cd30f4df passed the five-target preflight:
717 ordinary C unit tests, Rust Clippy, rustfmt, buildifier and documentation.
Invocation 54cecd5e-2bba-4358-817a-d3ab199d5de8; 75.714 seconds.
The full gate additionally identified a missing explicit cfg(test) marker on
the handwritten readonly-consumer helper. The marker is added; the source-policy
rule and all tests remain enabled. That superseded run passed 623 tests and
failed source policy; it is not counted as the successful release gate.

The first independent Sol Extra High review found two required dependency
boundary fixes. Both were accepted:

- Direct and transitive complete-export collision checks omitted owned globals.
  They now share an exact resolved file-scope name inventory containing owned
  functions and Global values, excluding local/parameter/member namespaces.
  Tests cover both normalized/requested spellings, called and uncalled imports,
  unselected direct exports and transitive exports. Independent GCC/Zig O0/O2
  controls prove the corresponding header compile and external-link failures
  after mutating an otherwise valid generated constant name.
- The old function-only dependency API could accept mixed producers containing
  synthesized constants while omitting those constants from its public inventory.
  It now rejects any certified object-bearing producer until child02B. The
  check scans actual definitions in complete certified sources, whose context
  proof already requires every registered object to have exactly one definition.
  Constants-only, ordinary mixed and synthesized-constant mixed controls reject.

Tree 419a55ede2b8bc4eccb3c54e750795c7f67d5124 passed all six preflight targets,
including source policy and all 720 ordinary C unit tests. Invocation
218deec2-589c-4e08-8ccc-71ab9c83edc3; 78.706 seconds.

A fresh second independent Sol Extra High review of that implementation tree
found no remaining current-scope errors. Both first-review findings were fixed;
none was dismissed as optional. The reviewer independently traced admission,
context reconstruction, readonly reads, projection, rendering, documentation,
resources, dependency collisions and the producer API boundary.

The same implementation tree passed the isolated full release/integration gate:
624 tests across 816 targets, 576.963 seconds, invocation
9ea6b076-6516-4eed-905f-4fbcdcdd25cb. The archive matched all 2,535 Git blob bytes
and executable modes. Targets included release_gate, all rustc-frontend
experiments, C/Java unit and compile-negative suites, shared codegen and typed
pipeline tests, documentation and Rust/Bazel/source-policy lint gates. Existing
test caching remained enabled. No test, legacy entry point or runtime was
disabled or removed.

Child02B remains required for imported constants, followed by Java target APIs,
compiler source admission and bundle publication. This completion does not claim
public-constant Rust-source parity or authorize legacy runtime deletion.
