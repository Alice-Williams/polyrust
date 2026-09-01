# M21 — Add JavaScript and Java outputs

- Status: complete
- Phase: 7
- Depends on: M14, M16, M20

## Outcome

Expand every checked PolyRust program from four to six supported outputs:
Rust, TypeScript, JavaScript, Python, Go, and Java.

JavaScript is derived from the TypeScript implementation and tested both as a
standalone Node.js package and against the corresponding TypeScript behavior.
Java is a first-class backend that consumes only `CheckedProgram` and uses a
pinned hermetic Java 21 toolchain.

## Implementation checklist

- Share TypeScript/JavaScript declaration and runtime generation so behavior
  cannot drift behind two unrelated implementations.
- Emit a standalone ESM JavaScript package that needs no TypeScript compiler at
  consumer runtime.
- Add Java backend descriptor, capability map, typed declarations, runtime,
  portable tests, deterministic manifest, and safe identifier/literal handling.
- Pin Java 21 through Bazel and add Java formatting or compiler-lint gates.
- Extend conformance, models-and-validation, and all four completed real-world
  ports to generate and natively test all six outputs.
- Update architecture, portable-language, compatibility, and language-roadmap
  documentation.

## Required exit evidence

- JavaScript output is mechanically tied to TypeScript generation and passes
  Node.js tests without loading TypeScript sources.
- Generated Java compiles with Java 21, passes strict compiler diagnostics, and
  executes all portable and semantic conformance tests.
- All six manifests regenerate byte-identically.
- Each completed real-world port passes its upstream differential oracle and
  native tests in all six outputs.
- Repository Rust and Bazel linters and the complete release gate pass.

## Scope boundary

JavaScript is a distribution target derived from TypeScript, not a second
independent interpretation of checked IR. Java remains an independent semantic
backend because it has its own type system, runtime values, integer behavior,
Unicode representation, and identifier constraints.

## Completion evidence

- `//crates/backend-typescript:javascript_runtime_derivation_test`
  recompiles `runtime.ts` with TypeScript 7.0.2 in ESM mode and requires
  byte-identical `runtime.js`; its standalone package contains no TypeScript
  sources and passes Prettier plus Node 24 tests.
- Java packages compile with the Java 21 language/runtime contract, strict
  `-Xlint:all -Werror` diagnostics, and Bazel's hermetic remote JDK. Portable,
  20-vector conformance, and external-package public API tests pass.
- The models-and-validation example and all four completed real-world ports
  regenerate six byte-identical manifests. Every pinned upstream differential
  oracle and every generated native package passes.
- The expanded 1,000-declaration benchmark generates six targets, 34 files, and
  1,708,562 bytes in 277 ms with 11.22 MiB peak RSS, within the unchanged gate.
- `bazelisk test //...` passes all repository tests; the explicit
  `//:release_gate` passes 48 release-blocking targets including Buildifier,
  Rustfmt, warning-denied Clippy, release policies, and native output tests.
- `polyrust-conformance --all-targets --determinism` reports 50 cases,
  one portable fixture test, six agreeing targets, and byte-identical manifests.
