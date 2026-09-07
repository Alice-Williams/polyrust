# M34A-10W — Make Java capability mappings authoritative

- Status: in-progress
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10R and M34A-11

## Goal

Give every Java `Supports<C>` proof a registered mapping which owns the
complete portable/CoreIR-to-Java-AST translation for capability `C`.

## Definition of done

- `crates/backend-java/src/capabilities/` has exactly one mapping file per
  supported capability and a registration-only `mod.rs`.
- Structural and value mappings accept portable/CoreIR inputs and construct
  Java AST; identity mappings over `JavaExpr`, `JavaMethod`, or
  `JavaTypeDeclaration` are absent.
- `JavaInterfaces` maps declarations, methods, implementation bindings,
  interface values, concrete calls, interface calls, and multiple
  conformance as one capability.
- `JavaEnums` emits Java enums for payload-free portable enums. It does not
  encode a portable enum as an inheritance hierarchy.
- Every mapping returns only typed Java AST, expression plans, or typed symbol
  requirements. Imports and helpers remain linker-derived.
- Dynamic Java support derives presence and strategy from the same plugin
  slots, then performs shape-specific preflight.

## Tests

- Every Java capability mapping has an instrumented invocation test.
- Removing one mapping fails both typed compilation and dynamic preflight.
- Wrong capability inputs and Java-output categories fail compilation.
- Interface binding, enum exhaustiveness, constant ordering, control-flow,
  literal, construction, and operation fixtures generate and compile.
- The Java mutation/compiler oracle covers every new mapping input family.
- Generated snapshots, all historical Java ports, Java 21
  `-Xlint:all -Werror`, Rustfmt, strict Clippy, Buildifier, full tracked
  repository, and release gates pass in the Linux development container.

## Review-remediation checklist

- [x] Replace the erased all-intrinsics input and central Java operation match
  with one exact input enum and lowering function per operation capability.
- [x] Route short-circuit Boolean logic through `BooleanLogic`.
- [x] Route target type construction and all literal/composite construction
  through the owning value, record, enum, or interface mapping.
- [x] Give `Functions` returns and parameter reads; dispatch `let`, loop, and
  pattern reads to their verified owning capabilities.
- [x] Route normal and constant references through `Constants`.
- [x] Make `Interfaces` map complete multiple-conformance bundles and make
  `PortableTests` map its own invocation forms.
- [x] Remove out-of-catalogue Java loop mapping inputs and unused duplicate
  conditional inputs.
- [x] Replace Java's blanket manual `Supports<F>` implementation with exact
  catalogue-generated delegation whose bounds require every implemented slot.
- [x] Make dynamic preflight confirm `ResultPropagation` for calls and fallible
  operations and report payload-free enum construction as native.
- [x] Extend source/layout policy for exact filenames, operation inputs,
  catalogue delegation, and registrations.
- [x] Infer `ResultPropagation` for every fallible typed intrinsic; prove a
  checked-arithmetic program fails admission without it and passes with it.
- [x] Map interface `self`, both enum comparison operators, and record/enum
  portable-value construction through their owning capabilities.
- [x] Move legacy payload-enum declaration groups through an exact `Enums`
  input, removing the remaining direct declaration bypass from the orchestrator.
- [x] Preserve exact checked conformance evidence through interface coercion
  and validate its target types and implementation methods during certification.
- [x] Exercise every closed mapping input variant and add independently
  compiled/executed Java operation and concrete-method fixture targets.
- [x] Move substantial Java entry-point tests and fixtures to focused files;
  record the source-size and Bazel action-boundary guidance.
- [ ] Pass the full local/release gates, hosted CI, and a fresh blind review
  with no accepted correctness or architecture findings.

## Commit gate

Commit and push only after the complete Java proof passes. Hosted CI for the
exact checkpoint and a fresh uncapped review are required before marking the
task complete.
