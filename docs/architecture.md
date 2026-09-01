# PolyRust v0 technical specification

Status: design draft

The words MUST, MUST NOT, SHOULD, and MAY describe intended requirements for the
v0.1 implementation. This document specifies behavior; the exact Rust API may
evolve during implementation.

## 1. System model

PolyRust is the compiler for one portable code-generation language. It is not a
text templater, a machine-code compiler, or a translator between existing source
languages.

```text
Rust builder program          Serialized .poly.json
          │                             │
          └──────────────┬──────────────┘
                         ▼
                 Unchecked PolyIR
                         │
               resolve + type-check
                         ▼
                  CheckedProgram
                  │             │
          evaluator          capability scan
                  │             │
                  │       backend preflight
                  │             │
                  │      target lowering
                  │             ▼
                  │       OutputManifest
                  │             │
                  │       atomic file write
                  ▼             ▼
       portable tests/oracle  native format/build/test/run
                  └──────── behavior comparison ────────┘
```

No public safe backend API accepts `UncheckedProgram`. A generated package is an
artifact tree described by an `OutputManifest`, not an arbitrary sequence of
filesystem writes.

The Rust builder is the reference v0 authoring frontend. Future Rust-like macros
or a restricted Rust parser MAY also construct `UncheckedProgram`, but MUST pass
through the identical checker and MUST NOT access checked-program constructors or
backend APIs directly. Such frontends accept only syntax with specified PolyRust
semantics; ordinary Rust semantics are not inherited automatically.

## 2. Proposed Cargo workspace

```text
polyrust/
  Cargo.toml
  crates/
    polyrust-ir/             versioned syntax and semantic nodes
    polyrust-diagnostics/    codes, spans, renderers
    polyrust-check/          resolution, types, capabilities
    polyrust-eval/           reference evaluator and portable test runner
    polyrust-build/          typed Rust builder API
    polyrust-codegen/        backend contract, writer, manifests
    polyrust-backend-rust/
    polyrust-backend-typescript/
    polyrust-backend-python/
    polyrust-backend-go/
    polyrust-conformance/    evaluator/native differential harness
    polyrust-cli/
  conformance/               shared programs and canonical vectors
  examples/
  generated-fixtures/        regenerated in tests; policy documented
```

Dependencies MUST point inward: backends depend on checked IR and codegen APIs;
core crates MUST NOT depend on concrete backends. The CLI performs registration.
There MUST NOT be a `match target_name { "rust" => ... }` in a core crate.

## 3. PolyIR layers

### 3.1 Unchecked syntax IR

The syntax layer retains names and source locations and is serializable. Its root
contains:

```text
Document {
  ir_version,
  module,
  metadata,
}

Module {
  name,
  declarations: [Declaration],
}

Declaration = Constant | Alias | Record | Enum | Contract | Implementation
            | Function | Test
```

Every declaration has a stable local `NodeId`, identifier, visibility, optional
documentation, and `SourceRef`. A `SourceRef` can be a byte span in a file or a
logical builder path such as `module(example).function(validate).body`.

### 3.2 Checked semantic IR

Checking resolves identifiers to stable IDs, computes every expression type,
proves contract implementations, match exhaustiveness, return coverage, and
portable-test validity, records capabilities, and canonicalizes declaration
ordering where ordering is not semantic.

`CheckedProgram` constructors are crate-private. Its public API exposes immutable
queries. Backends MUST NOT mutate it.

### 3.3 Serialization

- v0 uses canonical UTF-8 JSON with the suffix `.poly.json`.
- The root MUST contain `ir_version` as semantic version text.
- Object keys and unordered declarations MUST serialize in specified order.
- No timestamps, absolute local paths, random IDs, or map iteration order may
  affect canonical bytes.
- Readers MUST reject unsupported major versions.
- The v0 policy for unknown fields is reject-with-diagnostic, avoiding accidental
  semantic loss. A later minor-version preservation scheme may revise this before
  1.0.
- Parse → serialize → parse MUST preserve equality, excluding non-semantic source
  display caches.

## 4. v0 semantic model

### 4.1 Types

Required types are:

- `Unit`
- `Bool`
- `I32`
- `I64`
- `F64`
- Unicode-scalar `Char`
- `String`
- `Bytes`
- `List<T>`
- `Option<T>`
- `Result<T, E>`
- named records
- named tagged enums with zero or one record-shaped payload per variant
- named restricted contract parameter views
- type aliases that cannot be recursive without an indirection approved by a
  later specification

The Core prelude defines closed error enums used by semantic operations:
`ArithmeticError` (`Overflow`, `DivisionByZero`, `InvalidShiftCount`),
`ConversionError` (`OutOfRange`), and `Utf8Error` (`InvalidEncoding`). Their
serialized tags and meanings are part of the compatibility contract.

No implicit conversions exist. Null is not a type or value. Generic user-defined
declarations, maps, sets, tuples, function values, references, and pointers are
deferred.

### 4.2 Constants

A constant is a deterministic immutable module value, not a promise that every
target can represent it with its native compile-time `const` syntax. Core
constant expressions contain literals, immutable composite construction,
references to earlier acyclic constants, and operations specifically designated
`const_safe`. A backend MAY emit a native constant, static/lazy value, or
immutable accessor, but observable value and initialization behavior MUST match.

### 4.3 Values and identity

PolyRust values have immutable, structural value semantics. Assignment and
argument passing cannot expose shared mutable identity. Backends MAY use pointers
or references internally, but mutation through one value MUST NOT observably
change another.

This rule is especially important for Go slices and pointers, Python lists, and
TypeScript arrays/objects. Initially, list operations MUST return new values.
Generated public APIs SHOULD discourage mutation through target-idiomatic readonly
or defensive representations where practical. Rust generated code MUST use safe
types and contain no `unsafe`.

Raw pointer operations are not in v0. Adding them later requires a distinct
capability and a memory/aliasing specification; the existence of pointers in Rust
and Go does not make their semantics automatically portable.

### 4.4 Numeric behavior

- `I32` and `I64` are exactly two's-complement signed widths.
- Integer arithmetic nodes are explicit: `add_checked`, `sub_checked`,
  `mul_checked`, and their `*_wrapping` counterparts. There is no ambiguous plain
  integer arithmetic node in v0.
- Checked operations return `Result<T, ArithmeticError>`; wrapping operations
  return `T` modulo the width.
- Checked division truncates toward zero. Checked remainder has the dividend's
  sign and satisfies `a = (a / b) * b + (a % b)`. Division by zero and
  minimum-value / -1 return a structured arithmetic error.
- `F64` follows IEEE 754 binary64. Canonical conformance encoding represents NaN,
  infinities, and negative zero with tagged strings so JSON does not erase them.
- Float `rem_trunc` is defined as `a - trunc(a / b) * b`, with IEEE special-value
  behavior specified by conformance vectors rather than a target's `%` default.
- Float equality is IEEE equality; a separate total-order operation is deferred.
- Backends MUST inject helpers where target-native operators do not implement
  these rules, notably TypeScript and Python integer widths.

### 4.5 Text and bytes

- `String` is a sequence of Unicode scalar values. Ill-formed surrogate values
  are not representable.
- String length and indexing, if provided, operate on scalar values, not UTF-8
  bytes or UTF-16 code units.
- v0 SHOULD include scalar iteration but MAY omit random indexing until its error
  type is finalized.
- `Bytes` is an immutable sequence of octets and never implicitly converts to
  `String`.
- No normalization is implicit. Equal strings contain the same scalar sequence.

Rust is natively UTF-8, ECMAScript uses 16-bit string elements, and Go strings are
byte sequences. Therefore backend helpers and conformance cases are required for
astral characters, combining marks, and invalid construction boundaries.

### 4.6 Control flow and functions

v0 supports:

- immutable `let` bindings;
- literals, constructors, field access, calls, comparisons, Boolean operators,
  explicit numeric operations, and list operations;
- `if` expressions;
- exhaustive `match` over enums, `Option`, and `Result`;
- bounded `for each` over a list;
- explicit `return` or expression-bodied functions;
- pure record methods with immutable `self`; and
- calls through restricted contract parameters.

Evaluation order is left-to-right. Boolean `and` and `or` short-circuit.
Functions are pure: they read parameters/constants and return values without
observable host effects. Recursion is deferred for v0 to keep evaluator limits
and target stack behavior out of the first contract.

Exceptions and panics are not PolyRust control flow. A backend MUST NOT implement
a normal `Result` path by throwing. Internal generated helpers may fail only for
violations that the checker proves unreachable; such failure is a generator bug.

### 4.7 Contracts and implementations

A v0 contract declares required immutable instance methods. A record explicitly
declares an implementation and supplies a body for every method. The checker MUST
verify exact parameter and return compatibility and MUST reject extra/missing,
generic, default, associated-type, static, or mutable-self contract members.

A contract type MAY appear only as a function or method parameter in v0. A
matching concrete record value can be passed to it. Contract values MUST NOT be
returned, stored in fields, collections, constants or results, compared, cloned,
downcast, or observed for identity. This supports useful abstract dispatch while
avoiding a premature portable ownership/reference model.

The mapping is:

- Rust: trait, explicit impl, and `&dyn Trait` parameter;
- TypeScript: interface plus explicit `implements` declaration;
- Python: `typing.Protocol` plus explicit generated conformance;
- Go: interface, methods, and compile-time implementation assertion.

PolyRust conformance is nominal even when a target's type system is structural.

### 4.8 Portable tests

A test declaration has a stable ID/name, a checked function or method invocation,
typed canonical arguments, and either an expected typed value or expected
structured error. Tests are part of the program model.

The evaluator MUST execute portable tests directly. Every backend MUST lower them
to its native test framework and MUST also expose canonical results to the shared
conformance harness. A test cannot contain target source, host effects, timing,
randomness, or assertions outside the portable value/error model.

### 4.9 Capabilities

Each semantic feature has a stable capability identifier, for example:

```text
core.record
core.enum.tagged
core.contract.parameter_dispatch
core.test.value
core.i64.checked
core.i64.wrapping
core.string.scalar_iteration
core.list.persistent
```

The checker computes the minimal required set for a program. Backends report
support as `native`, `helper`, or `unsupported`, with notes about injected files
and dependencies. Generation MUST preflight the whole set before producing a
manifest.

Future host effects use declared adapter capabilities such as `host.clock.now`.
Every requested adapter must be bound per target. There is no implicit access to
the Rust host program from generated code.

## 5. Backend contract

Illustrative API:

```rust
pub trait Backend: Send + Sync {
    fn descriptor(&self) -> BackendDescriptor;
    fn support(&self, capability: CapabilityId) -> Support;
    fn options_schema(&self) -> OptionsSchema;
    fn generate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, Diagnostics>;
}

pub struct BackendDescriptor {
    pub target: TargetId,
    pub backend_version: Version,
    pub supported_ir: VersionReq,
}

pub struct OutputManifest {
    pub files: Vec<OutputFile>,
    pub dependencies: Vec<DeclaredDependency>,
    pub helpers: Vec<InjectedHelper>,
}
```

Rules:

- `TargetId` is namespaced text, not a closed Rust enum.
- Backends MUST return relative normalized paths only.
- Duplicate paths, absolute paths, drive prefixes, `.`/`..`, reserved output
  metadata paths, and case-folding collisions MUST be rejected.
- Output files carry UTF-8 text or bytes explicitly.
- Backend options are validated before generation and included in the
  determinism key.
- A backend cannot weaken validation. Target-specific reserved-name and layout
  checks may add diagnostics during preflight.
- Formatters run as an explicit post-process and MUST be pinned in CI. The raw
  generator remains deterministic.

## 6. Shared document writer

Backends use a small pretty-printing algebra rather than concatenate indentation
manually. Required operations include text, hard/soft line breaks, indentation,
groups, joining, and target-owned escaping. The shared writer handles layout but
does not know language syntax, identifiers, imports, or keywords.

Each backend owns:

- identifier validation and collision-safe name allocation;
- reserved keywords;
- literals and escaping;
- type mapping;
- symbol/import mapping;
- declaration/expression lowering;
- package layout and build metadata; and
- semantic helper selection.

## 7. Required target mappings

| PolyRust concept | Rust | TypeScript | Python | Go |
| --- | --- | --- | --- | --- |
| `Unit` | `()` | generated singleton unit value | generated singleton unit value | `struct{}` |
| `I32` | `i32` | `number` + checked helpers | `int` + width helpers | `int32` |
| `I64` | `i64` | `bigint` + helpers | `int` + width helpers | `int64` |
| `Char` | `char` | validated one-scalar string | validated one-scalar string | `rune` |
| constant | native `const` or static/lazy/accessor | exported readonly binding | module binding with immutable representation | native `const` or immutable package API |
| `Option<T>` | `Option<T>` | tagged/readonly option, not nullable | generated tagged generic/type | generated generic option struct |
| `Result<T,E>` | `Result<T,E>` | tagged union | generated tagged generic/type | generated generic result struct |
| tagged enum | `enum` | discriminated union | sealed generated variants/dataclasses | interface or tag+payload representation selected by spec |
| restricted contract | `trait` + `impl` + `&dyn` parameter | `interface` + `implements` | `Protocol` + declared conformance | `interface` + methods + compile assertion |
| portable test | `#[test]` | selected native test runner | `pytest` | `go test` |
| immutable list | owned/borrowed safe wrapper or `Vec` by value | `ReadonlyArray` plus copy helpers | tuple or non-mutating generated API | defined slice wrapper/copy discipline |
| Unicode scalar iteration | `.chars()` | code-point iterator with surrogate validation | code points with surrogate rejection | `range` plus UTF-8 validation |

The concrete `Option`, `Result`, enum, and list public shapes require snapshots and
API review in each backend task. Go pointer use is an implementation/API-layout
choice, never an exposed PolyRust semantic.

## 8. Diagnostics

Diagnostics have:

- stable code (`P` + four digits);
- severity;
- concise message;
- primary `SourceRef`;
- zero or more related locations;
- remediation hint; and
- optional target/backend context.

Examples:

- `P0001` unsupported IR major version;
- `P0102` duplicate declaration;
- `P0207` type mismatch;
- `P0214` non-exhaustive match;
- `P0220` contract implementation does not conform;
- `P0230` invalid portable test invocation or expectation;
- `P0301` impure operation in pure function;
- `P0404` target capability unsupported;
- `P0502` unsafe output path.

Codes become compatibility surface after v0.1. Tests compare structured
diagnostics, not terminal color text alone.

## 9. Evaluator, portable tests, and conformance protocol

The evaluator executes checked IR only. It is deliberately simple and
unoptimized. It has configurable fuel and collection-size limits and returns a
structured limit error rather than hanging or exhausting the process stack.

Portable test declarations are the primary user-authored behavioral cases. The
compiler's additional conformance cases contain:

```text
case id
IR program fixture
exported function
canonical input values
expected canonical value or structured error
required capabilities
```

For each required target, native generated tests execute the declared cases and a
generated runner encodes canonical results. The harness compares evaluator,
Rust, TypeScript, JavaScript, Python, Go, and Java results after canonical
encoding.

Tests MUST cover boundary integers, overflow, negative zero, NaN, Unicode scalar
length, nested `Option`/`Result`, every enum variant, evaluation order, short
circuiting, list non-aliasing, reserved identifiers, and empty collections.

## 10. Atomic output and security

- Generation creates a complete in-memory manifest before filesystem mutation.
- The writer stages files in a sibling temporary directory, verifies all resolved
  paths remain below it, then swaps/replaces according to a documented recovery
  procedure.
- v0 MUST NOT delete unknown files from an existing output directory. A later
  `--clean` mode requires explicit ownership metadata and user intent.
- Serialized IR is untrusted input. Parsing, validation, interpretation, and
  rendering have configurable size/depth/fuel limits and MUST NOT panic.
- Documentation text is escaped as target comments/docstrings and cannot inject
  syntax.
- Backend dependency declarations are data; PolyRust does not fetch or execute
  them during pure generation.

## 11. Test strategy

1. Unit tests for IR constructors, canonical serialization, checker rules,
   writer layout, escaping, and every lowering function.
2. Negative compile/check fixtures with stable structured diagnostics.
3. Golden snapshots for complete packages and difficult syntax fragments.
4. Regeneration tests proving byte-identical output.
5. Generated portable tests in every target's native test framework.
6. Native checks: Rust format/clippy/test; TypeScript format/typecheck/test;
   Python format/typecheck/compile/test; Go format/vet/test.
7. Differential conformance against the evaluator.
8. Property tests for serialization round trips, name allocation, numeric helper
   boundaries, and path rejection.
9. Mutation testing or targeted fault injection for the checker and semantic
   helpers before alpha.

Snapshot approval alone never establishes semantic correctness.

## 12. Compatibility and extension policy

- IR, backend API, diagnostic codes, and canonical encoding are independently
  versioned.
- A backend declares an IR version range and its exact capability support.
- Adding an optional declaration field is a minor IR change only if old readers
  have a specified safe behavior; otherwise it is major before 1.0.
- Changing evaluation behavior, type mapping visible at public APIs, canonical
  encoding, or helper error behavior is semantic and requires a compatibility
  review.
- A new core IR feature requires designs and conformance cases for all eight
  required targets before merge.
- An external “toy” backend fixture remains outside core and compiles in CI to
  detect accidental coupling.

## 13. Deferred design areas

- Rust-like macro/attribute syntax for the same PolyRust program model.
- A documented restricted-Rust parser that lowers into unchecked PolyIR and
  rejects unsupported Rust constructs.
- Mutable collections and a formal aliasing model.
- Portable references or pointers.
- Maps and deterministic iteration.
- Recursion and resource behavior.
- Effects/adapters for clocks, randomness, files, networking, and logging.
- Async/concurrency.
- Generic/default contracts, associated types, inheritance, closures, and
  higher-order functions.
- Source maps back to Rust builder call sites.
- Optimization passes and incremental generation.
- WebAssembly or binary targets.
