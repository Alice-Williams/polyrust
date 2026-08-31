# PolyRust feasibility study

Research date: 2026-08-31

## Executive verdict

The project is feasible as one extensible portable code-generation language.
Authors use Rust to construct a PolyRust program containing declarations,
behavior, and tests; PolyRust checks and evaluates that program, then renders it
through supported target backends. There is no source-language-to-source-language
mapping matrix.

The central engineering work is a common functionality map: every type,
operator, abstraction, and control-flow construct needs one target-independent
meaning and verified lowerings for Rust, TypeScript, Python, and Go. Unsupported
functionality remains a visible capability gap until all required layers exist.

See the [portable language map](../portable-language.md) for the proposed Core,
Next, Later, Adapter, and Excluded feature sets.

## Why the base language needs explicit semantics

### Semantic differences are observable

- Rust `String` is valid UTF-8, while ECMAScript strings are sequences of 16-bit
  values generally interpreted as UTF-16. Even `length` can mean different
  things. See the [Rust string documentation](https://doc.rust-lang.org/std/string/index.html)
  and the [ECMAScript String definition](https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-string-type).
- Rust integer overflow can panic or wrap depending on the operation and compiler
  configuration; Go defines deterministic overflow for its integer operations;
  ECMAScript `Number` is binary64 and cannot exactly represent every 64-bit
  integer. See the [Rust overflow rules](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#integer-overflow),
  [Go specification](https://go.dev/ref/spec#Integer_overflow), and
  [ECMAScript numeric types](https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types-number-type).
- Ownership and borrowing have no direct Python or TypeScript equivalent;
  garbage-collected reference identity has no automatic value-semantics mapping
  back to safe Rust.
- Exceptions, panics, `Result`, sentinel values, and process termination carry
  different control-flow and cleanup behavior.
- File systems, clocks, networking, threads, async runtimes, reflection, and FFI
  depend on a target platform, not only a target syntax.

These mismatches require a PolyRust semantic contract and, sometimes, generated
runtime helpers. Rust is the authoring host and an output target; arbitrary Rust
semantics are not the PolyRust base language.

### A low-level universal IR solves a different problem

[LLVM IR](https://llvm.org/docs/LangRef.html) deliberately works as a typed,
low-level common representation for machine-code compilation. It discards or
lowers many high-level concepts needed to reconstruct readable, idiomatic source.
Rust itself lowers typed high-level forms to control-flow-oriented
[MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html), then to LLVM IR.
Using MIR would also couple PolyRust to compiler-private details and Rust-specific
semantics.

WebAssembly is valuable when the desired result is one portable executable
artifact. Its [specification](https://webassembly.org/specs/) and component work
address execution and interoperability, not generation of maintainable native
source in each language. It is a possible future runtime target, not the core
source-generation IR.

## Strong precedents

The idea is viable within an explicit common subset:

- [Haxe](https://haxe.org/documentation/introduction/compiler-targets.html) is a
  statically typed source language with JavaScript, Python, C++, JVM, and other
  targets. Its existence validates the multi-backend model; its target tiers also
  show the ongoing maintenance cost of each backend.
- [MLIR](https://mlir.llvm.org/) demonstrates the value of specified IRs,
  verifiers, textual forms, modular libraries, and extensible dialects. PolyRust
  should borrow those architectural principles without taking on MLIR's C++
  infrastructure or optimization scope.
- [Smithy's code-generation architecture](https://smithy.io/2.0/guides/building-codegen/index.html)
  separates a semantic model, symbols, language-specific writers, and pluggable
  integrations. It explicitly notes that each target language is unique.
- [Protocol Buffer compiler plugins](https://protobuf.dev/reference/cpp/api-docs/google.protobuf.compiler.plugin.pb/)
  show how a versioned request/response boundary can let generators evolve
  independently.
- [Kotlin Multiplatform](https://kotlinlang.org/docs/multiplatform/multiplatform-expect-actual.html)
  uses common declarations plus required platform implementations. This supports
  the proposed capability/adapter model for operations that cannot be portable.
- [C2Rust](https://github.com/immunant/c2rust) preserves much C behavior by first
  producing unsafe, unidiomatic Rust, then requiring refactoring. This is useful
  evidence that semantic preservation and idiomatic output are separate goals.

## Recommended product definition

PolyRust is a compiler toolkit with four layers:

1. A typed, immutable, versioned high-level IR with specified behavior.
2. A checker and small reference evaluator/portable test runner.
3. A public backend interface with feature/capability negotiation.
4. Target backends that emit deterministic, formatted, buildable source.

The authoring surface is a normal Rust library. Users write generator programs
that call builders such as `ModuleBuilder`, `RecordBuilder`, `ContractBuilder`,
`FunctionBuilder`, and `TestBuilder`. This satisfies “write the generation code
once” while keeping the PolyRust program distinct from arbitrary host Rust. A
serialized form supports fixtures, inspection, caching, and tooling without
becoming an arbitrary-source parser.

## Is Rust the right choice?

### As the implementation and authoring host: yes

Rust is a good fit because enums model IR nodes precisely, traits make backend
contracts explicit, exhaustive matching exposes missing lowering cases, and the
type system helps contain accidental mutation. Cargo workspaces also suit a core
plus backend-crates architecture.

Verbosity is not the decisive advantage. The important qualities are explicit
data modeling, exhaustive cases, performance, safe concurrency if later needed,
and distribution as a native CLI.

### As the program being copied into every output: no

Full Rust adds macro expansion, name resolution, trait solving, monomorphization,
lifetimes, unsafe code, conditional compilation, target-dependent layout, and an
enormous standard-library surface. Reusing `rustc` internals would create a
versioning burden. Parsing Rust syntax with a library would still not supply all
the semantic information required for correct translation.

In v0, PolyRust authors express portable concepts through the builder API rather
than asking the tool to reinterpret ordinary Rust syntax. Once the common model
is stable, a Rust-like macro or restricted Rust parser can lower supported syntax
into the same unchecked IR. It remains authoring convenience: unsupported Rust
constructs are rejected, and every accepted construct still uses the same
checker, evaluator, backends, and conformance tests.

## Required proof-of-concept targets

| Target | Role | Advantages | Main semantic work |
| --- | --- | --- | --- |
| **Rust** | Required reference backend | Strong types, exhaustive enums, native match for IR concepts, validates that generated Rust is useful | ownership-friendly generated APIs, module/import layout, `Result`/`Option` |
| **TypeScript** | Required contrast backend | Large SDK/tooling audience, static checking, browser and Node reach | 64-bit integers via `bigint`, UTF-16 strings, tagged unions, mutability |
| **Python** | Required contrast backend | Very readable output, fast test loop, popular for generated clients/tools | dynamic runtime checks, indentation, type hints, integer width enforcement |
| **Go** | Required middle-ground backend | Static compiler, explicit pointers and value copying, simple syntax, explicit error handling, excellent generated-code ecosystem | tagged unions, option representation, slice aliasing, nil, pointer/value API choices |

Rust must ship in the same proof-of-concept acceptance gate as TypeScript,
Python, and Go. It should be implemented first, but the milestone is not complete
until all four pass identical behavioral vectors. Go tests whether the IR's value
semantics survive a second static compiled language with explicit pointers,
value copying, slices, and nil; these behaviors are defined in the
[Go language specification](https://go.dev/ref/spec#Pointer_types). PolyRust v0
does not expose portable raw pointer
operations; generated Go may use pointers internally when that preserves the
specified value/API contract. Go is not installed on the current machine, so its
task includes toolchain setup or CI execution.

## Best first generated-code use cases

1. Data models: records, tagged enums, optional values, constructors, and
   equality.
2. Pure validators: range checks, string checks, and structured errors.
3. Pure transformations: mapping a request model to a wire model or domain model.
4. Small protocol/SDK support types with no networking in the generated layer.

These exercise real semantics without making file systems, async runtimes, or
third-party libraries part of the first contract.

## Proposed portable subset (v0)

- Modules, imports within the generated package, constants, records, tagged
  enums, type aliases, contracts/interfaces, explicit implementations, pure
  functions/methods, and portable test declarations.
- `Unit`, `Bool`, `I32`, `I64`, `F64`, Unicode-scalar `Char` and `String`,
  `Bytes`, `List<T>`, `Option<T>`, and `Result<T, E>`.
- Local bindings, calls, construction, field access, `if`, exhaustive `match`,
  bounded iteration over lists, and return.
- Explicit checked and wrapping integer operators; no target-default integer
  arithmetic.
- Immutable value semantics and no observable object identity.
- No implicit null, exceptions, inheritance, reflection, unsafe memory, async,
  threads, macros, user-defined operator overloading, or arbitrary host-library
  calls.

`I64`, scalar-based string operations, tagged enums, restricted contract
dispatch, and portable tests are intentionally kept in v0 because they force
real cross-language design work. A proof of concept that only emits
syntax-identical arithmetic would not sufficiently test feasibility.

## Extensibility requirements

- Every IR document carries a schema version.
- Every node and operation has specified semantics independent of a backend.
- Every backend advertises target name, backend version, supported IR range, and
  capabilities.
- Validation computes required capabilities before output files are written.
- Missing support is a structured diagnostic, never a fallback guess.
- Backends are separate crates behind a stable Rust trait; Rust is not privileged.
- Target-specific adapters are declared at the boundary and supplied per target.
- Output is represented as a file manifest, allowing packages rather than only
  single files.
- Golden tests, native compilation/type checking, and differential execution are
  mandatory for every required backend.

## Main risks and mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Scope expands toward full language translation | Project never reaches a reliable release | Publish a feature matrix and reject unsupported nodes |
| IR becomes “Rust with renamed syntax” | Other targets produce awkward or incorrect code | Define semantics first; require four-target design review for new features |
| Output compiles but behaves differently | Loss of trust | Interpreter oracle plus cross-target differential tests |
| Generated code is unreadable | Users will not adopt it | Stable formatting, target idiom guidelines, snapshot review |
| Runtime helpers grow into hidden frameworks | Generated packages become heavy | Small audited helpers; report every injected helper and dependency |
| Backends drift | Feature matrix becomes misleading | Shared conformance kit and backend compatibility metadata |
| Name collision | Publishing/discovery friction | [`polyrust` already names an unrelated Polymarket project](https://github.com/nniel-ape/polyrust); perform registry, repository, package, and trademark checks before publication |

## Effort range

For one experienced compiler/tooling engineer working mostly full-time:

- architecture spikes and semantic contract: 1–3 engineering weeks;
- useful Rust/TypeScript/Python/Go proof of concept including contracts and
  portable tests: roughly 12–20 weeks total;
- documented alpha with extension API, robust diagnostics, packaging, and CI:
  roughly 16–28 weeks total;
- broad “general-purpose language” coverage: multi-year and permanently ongoing.

These are scope ranges, not delivery commitments. The quickest falsification gate
is the four-backend conformance milestone: if a deliberately difficult v0 subset
cannot preserve behavior cleanly, the program model should change before more
features are added.

## Conclusion

Proceed with PolyRust as a portable code-generation language and toolchain. Rust
hosts the authoring API and is also a required output target. The project expands
one functionality row at a time only when the same specified program and its
tests behave identically in Rust, TypeScript, Python, and Go.
