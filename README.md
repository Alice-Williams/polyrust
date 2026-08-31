# PolyRust

PolyRust is a proposed Rust-authored, multi-target code generator. Authors define
one program using PolyRust's typed, language-neutral programming model, including
types, constants, functions, implementations, and tests. PolyRust then validates,
evaluates, and emits that program as readable source code for several languages.

PolyRust is not an existing-language translator: it does not map arbitrary Rust
to Python or TypeScript to Go. The common program model is the source of truth and
grows deliberately as portable functionality is specified and tested.

The name is currently a working name and will not define the public API. The
repository contains the feasibility research, implementation plan, and the
first executable development-environment baseline.

## Decision snapshot

- **Product:** one extensible portable code-generation language and toolchain.
- **Authoring:** write the generator once through a Rust builder API.
- **Implementation language:** Rust.
- **Initial authoring interface:** a typed Rust builder API, plus a versioned
  serialized IR for testing and tooling.
- **Required proof-of-concept targets:** Rust, TypeScript, Python, and Go.
- **Correctness strategy:** validate before emission, run a reference evaluator,
  generate native tests, compile/type-check every target, then compare behavior.

Rust output is a first-milestone requirement. The Rust backend is the reference
backend and must not bypass the same public backend contract used by other
targets.

## Documents

- [Product charter](docs/charter.md)
- [Feasibility study](docs/plan/analysis.md)
- [Technical architecture](docs/architecture.md)
- [Portable language/functionality map](docs/portable-language.md)
- [Engineering plan](docs/plan/README.md)
- [Development environment](docs/DEVELOPMENT.md)
- [Architecture decisions](docs/adr/0000-template.md)

## Development

Step 0 uses the checked-in Linux Dev Container and Bazel configuration. On a
non-Linux host, open the repository in that container and run:

```text
bazel test //...
```

This initially proves both pinned Rust and Go toolchains. All implementation and
generated-language verification will be added beneath the same command.

## Recommended first demonstration

Use one Rust program to construct a small PolyRust module containing:

- constants, records, and tagged enums;
- a restricted contract/interface and an explicit record implementation;
- `Option` and `Result` values;
- pure concrete and contract-dispatched validation/transformation functions;
- explicit checked and wrapping integer operations; and
- at least ten portable test declarations.

Generate a Rust crate, TypeScript package, Python package, and Go module. Each
generated package must pass native formatting and static checks, run the same
portable/conformance tests, and produce the same canonical result as the PolyRust
evaluator.

## Product boundary

PolyRust should say “unsupported” with a precise diagnostic instead of silently
changing meaning. Platform I/O, reflection, concurrency, unsafe memory, macros,
inheritance, exceptions, and arbitrary third-party libraries are outside the
first portable language. Later versions can add them through explicitly
versioned capabilities and target adapters.
