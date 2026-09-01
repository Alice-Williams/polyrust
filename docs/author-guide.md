# Author guide

PolyRust has three deliberately separate layers:

1. **Rust host code** is the normal Rust program that calls the typed builder.
   It runs only while defining and generating a module.
2. **PolyRust portable code** is the checked, target-independent module produced
   by that builder. Its types and operations have the semantics described in
   the [portable language map](portable-language.md).
3. **Generated target code** in Rust, TypeScript, JavaScript, Python, Go, or
   Java is an output package. It is reviewed and compiled like ordinary target
   code but is never the authoring source of truth.

The complete compiled source is the
[`models-and-validation` example](../examples/models-and-validation/src/lib.rs).
It declares a constant, records, a restricted `Validator` contract, its
`AgeValidatorImpl`, concrete and abstract dispatch functions, and ten portable
tests. Typed handles prevent using a record field, contract method, or function
ID in the wrong builder position. `ModuleBuilder::finish` runs the checker and
returns sorted diagnostics instead of passing invalid input to a backend.

## Clean generation and test walkthrough

Run every command inside the [Linux development container](DEVELOPMENT.md):

```sh
rm -rf /tmp/polyrust-models-and-validation
bazelisk run //examples/models-and-validation:generate -- /tmp/polyrust-models-and-validation
find /tmp/polyrust-models-and-validation -type f | sort
bazelisk test //examples/models-and-validation:all
```

The `generate` invocation is the single command that creates all seven current packages
without hand edits. The `all` test runs the ten common tests in the reference
evaluator and all seven generated native frameworks.
It also generates twice in clean temporary directories, deletes one output,
regenerates it, and requires byte-identical trees. The walkthrough, source
example, external-backend template, and documentation links are all compiled or
scripted test inputs, so the guide does not depend on hidden local state.

## Builder, diagnostics, and capabilities

Start with `ModuleBuilder::new`, retain the typed handles returned by each
declaration, add `portable_test` declarations, and call `finish`. The detailed
API and semantics are in the [builder guide](builder-v0.md), [IR guide](ir-v0.md),
and [checker guide](checker-v0.md). Render any failure using the stable
[diagnostics model](diagnostics.md).

Generation accepts only `CheckedProgram`. A backend registry validates the IR
version and options, then compares the checked program's complete capability set
with the selected backend before invoking it. See the
[backend contract](backend-api-v0.md) and [backend author guide](backend-author-guide.md).

## Unsupported features and regeneration

PolyRust v0 intentionally rejects exceptions, async/concurrency, shared mutable
state, target FFI, reflection, implicit numeric conversion, inheritance, and
unbounded platform integer types. The complete boundary is documented in the
[portable language map](portable-language.md). Do not approximate an unsupported
construct in one emitter: extend the IR and checker semantics first, declare a
capability, update the evaluator and conformance vectors, then implement every
affected backend.

Generated packages are replaceable artifacts. Generate into a clean directory,
run the `all` gate, review the conventions in the
[generated-code review guide](generated-code-review.md), and replace downstream
output only after the manifest is deterministic.
