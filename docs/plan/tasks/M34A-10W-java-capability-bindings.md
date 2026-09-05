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

## Commit gate

Commit and push only after the complete Java proof passes. Hosted CI for the
exact checkpoint and a fresh uncapped review are required before marking the
task complete.
