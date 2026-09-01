# PolyRust product requirements document

Status: proposed v0.1

## Product statement

PolyRust is one extensible portable code-generation language hosted in Rust.
Generator authors describe constants, types, contracts/interfaces,
implementations, functions, and tests once, then emit readable, checked source
packages for Rust, TypeScript, JavaScript, Python, Go, and Java. New
functionality is added to the
common model one specified and conformance-tested capability at a time.

## Problem

Teams that publish the same models, validators, transformations, or SDK support
logic in several languages currently duplicate generators and semantic fixes.
Template-only systems centralize text but do not provide a shared type system or
behavioral contract. Conventional compiler IRs generally target executables and
discard information needed for maintainable high-level source.

## Primary users

- Library and SDK maintainers who publish equivalent packages in multiple
  ecosystems.
- Tooling authors who generate data models and pure business rules.
- Compiler contributors adding target backends or portable-language features.

## User jobs

1. As a generator author, I define a module once through a Rust API and generate
   Rust, TypeScript, JavaScript, Python, Go, and Java packages.
2. As a generator author, I define tests beside the common code and run the same
   tests in the evaluator and every generated target.
3. As a package maintainer, I can inspect, format, build, and test generated code
   with ordinary target-language tools.
4. As a backend author, I can add a target without modifying the core IR or other
   backends.
5. As a reviewer, I can see exactly which semantics and runtime helpers a
   generated package requires.
6. As a user with unsupported logic, I receive a source-located diagnostic before
   partial output is committed.

## Goals for v0.1

- Provide a typed Rust builder API and versioned serialized IR.
- Specify and validate the Core functionality in the portable language map.
- Include constants, records, enums, restricted contracts/interfaces, explicit
  implementations, pure functions/methods, and portable tests.
- Generate deterministic multi-file Rust, TypeScript, JavaScript, Python, Go,
  and Java packages.
- Format and statically check/compile all generated packages.
- Execute shared conformance vectors against a reference evaluator and all
  all required targets.
- Prove that all required outputs work through the same public core interfaces.
- Produce actionable diagnostics with codes, source paths, and suggestions.

## Non-goals for v0.1

- Translate or ingest ordinary Rust, TypeScript, Python, or Go source.
- Preserve comments or formatting from another language.
- Guarantee maximally idiomatic hand-written output.
- Support platform I/O, networking, databases, async, threads, reflection,
  inheritance, unsafe code, macros, or arbitrary dependencies.
- Optimize generated code beyond simple canonicalization.
- Round-trip generated source back into the IR.
- Provide binary ABI interoperability.

## Required target policy

Rust, TypeScript, JavaScript, Python, Go, and Java are release-blocking targets. Rust is
implemented first as the reference source backend, but it uses the same
`Backend` interface, capability checks, file manifest, snapshots, and conformance
suite as the others. A feature is not complete unless all required
backends implement it or the feature is explicitly moved out of v0.1.

## Functional requirements

### FR-1: Build a portable module in Rust

The library shall expose typed builders for modules, constants, records, enums,
aliases, contracts, implementations, functions, methods, expressions,
statements, portable tests, and source labels. Builders shall reject structurally
invalid states where practical and return diagnostics rather than panic on user
input.

### FR-2: Serialize and inspect IR

The CLI shall read and write a canonical, versioned textual IR format. Equivalent
IR values shall serialize byte-for-byte identically. Unknown major versions shall
be rejected; unknown optional fields may be preserved or rejected according to a
documented compatibility rule.

### FR-3: Validate before generation

The checker shall perform name resolution, duplicate detection, type checking,
contract-conformance checking, exhaustiveness checks, return-path checks, purity
checks, portable-test validation, and capability collection. No backend may
receive unchecked IR through its public safe API.

### FR-4: Define and run portable tests

The program model shall support named tests that invoke pure functions or methods
with canonical typed values and expect a typed value or structured error. The
reference evaluator shall run them directly. Every backend shall emit equivalent
native tests, and the conformance harness shall compare their canonical outcomes.

### FR-5: Generate Rust

The Rust backend shall emit a buildable Cargo library package with constants,
records, enums, traits/implementations, functions/methods, and native tests, with
deterministic imports and no `unsafe` code. Generated code shall pass `cargo fmt
--check`, `cargo clippy` under the documented lint policy, and `cargo test`.

### FR-6: Generate TypeScript

The TypeScript backend shall emit an ESM package with strict type checking,
interfaces/implementations, tagged unions, portable native tests, deterministic
imports, and any explicitly declared semantic helpers. Generated code shall pass
formatting, `tsc --noEmit`, and tests on the supported Node version.

### FR-7: Generate Python

The Python backend shall emit a typed package with protocols/implementations,
portable native tests, deterministic imports, and a documented minimum Python
version. Generated code shall pass formatting, static type checking, bytecode
compilation, and tests on the supported Python versions.

### FR-8: Generate Go

The Go backend shall emit a buildable module with interfaces/implementations,
portable native tests, deterministic imports, and explicit documented choices
for values, pointers, slices, options, and tagged enums. Generated code shall
pass `gofmt`, `go vet`, and `go test` on the supported Go version. Portable raw
pointer operations are not required in v0.

### FR-9: Explain target support

`polyrust targets` shall list backend versions and capabilities. `polyrust check
--target <target>` shall report every unsupported required capability. `polyrust
explain <diagnostic-code>` shall provide longer guidance.

### FR-10: Emit atomically

Generation shall first produce an in-memory file manifest. If validation or
generation fails, the requested output directory shall not be partially updated.
Writing the manifest shall reject absolute paths and `..` traversal.

### FR-11: Evaluate and compare behavior

A reference evaluator shall execute the common program and its declared portable
tests. The test harness shall run equivalent generated native tests and compare
canonical outputs or canonical structured errors.

### FR-12: Extend through backends

A backend shall be a separate crate implementing a documented stable trait. It
shall declare supported IR versions and capabilities, map symbols and types,
generate a file manifest, and contribute conformance metadata without requiring
core switches on its target name.

## Non-functional requirements

- **Correctness:** no silent semantic fallback; unsupported behavior is an error.
- **Determinism:** same tool version, IR, options, and target produce identical
  bytes on supported hosts.
- **Safety:** safe Rust core; no generated Rust `unsafe`; path-safe atomic writes.
- **Diagnostics:** stable codes, primary span/path, optional related locations,
  and a remediation hint.
- **Performance:** generate a 1,000-declaration fixture in under 2 seconds and
  under 512 MB on the documented benchmark machine; correctness takes priority.
- **Compatibility:** serialized IR and backend compatibility policy documented
  before v0.1 release.
- **Dependency transparency:** manifest and CLI report injected runtime helpers
  and package dependencies.
- **Testability:** every IR feature has checker, evaluator, and all-target
  coverage.

## Proposed user experience

Rust authoring code:

```rust
use polyrust::prelude::*;

fn module() -> Result<Module, Diagnostics> {
    ModuleBuilder::new("example")
        .constant("ADULT_AGE", Type::i32(), expr::i32(18))
        .contract("Named", |c| {
            c.method("name", |m| m.returns(Type::string()))
        })
        .record("User", |r| {
            r.field("name", Type::string())
             .field("age", Type::i32())
        })
        .implementation("User", "Named", |i| {
            i.method("name", |m| m.body(expr::field("self", "name")))
        })
        .function("is_adult", |f| {
            f.param("user", Type::named("User"))
             .returns(Type::bool())
             .body(expr::gte(expr::field("user", "age"), expr::constant("ADULT_AGE")))
        })
        .test("adult user", |t| {
            t.call("is_adult", [
                value::record("User")
                    .field("name", value::string("Alice"))
                    .field("age", value::i32(20)),
            ])
             .expect(value::bool(true))
        })
        .finish()
}
```

Illustrative CLI:

```text
polyrust check model.poly.json --target rust --target typescript --target python --target go
polyrust emit model.poly.json --target rust --out generated/rust
polyrust emit model.poly.json --target typescript --out generated/typescript
polyrust emit model.poly.json --target python --out generated/python
polyrust emit model.poly.json --target go --out generated/go
polyrust targets
polyrust explain P0017
```

Exact API spelling is not contractual until the builder task's design review.

## v0 portable feature acceptance rule

A proposed IR feature is accepted only when it has:

1. target-independent semantics in the technical specification;
2. IR and Rust-builder representation;
3. validation rules and stable diagnostic examples;
4. reference-evaluator behavior;
5. Rust, TypeScript, JavaScript, Python, Go, and Java lowering designs;
6. generated native and shared conformance tests; and
7. a compatibility note for serialized IR.

## Success measures

- One non-trivial example generates all required packages without
  hand-editing.
- The example includes constants, a contract and implementation, concrete and
  contract-dispatched functions, and at least ten portable tests.
- At least 50 shared semantic vectors pass in the evaluator and all targets,
  including overflow, Unicode, nested tagged enums, `Option`, and `Result`.
- Every generated package passes its native formatter and static/build checks.
- Repeated generation is byte-identical.
- A minimal external test backend implements the public interface without core
  target-name switches; required core changes are treated as design feedback.
- Deliberately unsupported features fail before filesystem mutation with a stable
  diagnostic.

## Release gates

### Gate A: semantic spike

The program model can represent the demonstration, including its contract and
tests, and the checker rejects ambiguous numeric, string, and abstraction
behavior.

### Gate B: Rust reference generation

The generated Rust crate builds, tests, contains no `unsafe`, and agrees with the
evaluator.

### Gate C: cross-target proof

Rust, TypeScript, JavaScript, Python, Go, and Java agree with the evaluator for all required
vectors.

### Gate D: extensibility proof

A minimal backend outside the core crates can use the documented backend API and
pass the backend contract tests.

## Product decisions still open

- Canonical serialized form: JSON first, or a custom readable syntax plus JSON.
- Minimum Python and Node versions for CI.
- Whether runtime helpers are emitted inline, as per-package modules, or as
  versioned target runtime packages.
- Whether public generated collections are deeply immutable or only treated as
  immutable by PolyRust-generated logic.
- Name availability: an unrelated project already uses `polyrust` on GitHub, so
  package and public branding must be checked before publication.
