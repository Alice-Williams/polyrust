# Layer 7: runtime helpers, files, and packages

- Status: normative
- Inputs: unresolved target declarations and helper references
- Outputs: deterministic unresolved/resolved package graphs

## Runtime helper catalogue

Every plugin owns a typed runtime-helper enum or generated typed helper IDs:

```rust
enum JavaRuntimeHelper {
    CallableResult,
    CheckedI32Add,
    CheckedI64Add,
    FloatAbs,
    DecodeUtf8,
}
```

Helper identity is never a free-form string in production APIs. A helper
specification contains:

- typed helper ID;
- deterministic order key;
- structural target AST declarations;
- typed references to other helpers;
- placement class;
- visibility;
- provenance pointing to the mapping which required it; and
- no manually attached imports.

## Helper selection

Target lowering emits a typed `HelperRef` only where target code uses that
helper. The linker computes one transitive closure over helper references in
program and helper AST.

It rejects:

- missing helpers;
- duplicate definitions;
- cycles;
- conflicting placement;
- illegal public helper exposure; and
- helper declarations containing opaque code.

Each selected helper is emitted once. Every unselected helper and its exclusive
external dependencies must be absent.

## Runtime representation

Checked-in executable runtime templates are prohibited. Runtime declarations
are built from the same target AST used for generated declarations and rendered
through the same total structural grammar renderer.

A future typed quasiquotation facility may construct target AST, but it must
parse and resolve every reference and cannot produce an opaque node.

## Package graph

```rust
struct TargetPackage<D: TargetDialect, S: PackageState> {
    groups: Vec<FileGroup<D, S>>,
    dependencies: PackageDependencies<D, S>,
}

struct FileGroup<D: TargetDialect, S: PackageState> {
    id: FileGroupId,
    role: FileGroupRole,
    files: Vec<TargetFile<D, S>>,
}
```

`FileGroupRole` is a closed enum such as `PublicApi`, `Implementation`,
`Runtime`, `NativeTests`, `Conformance`, `NegativeTests`, `Metadata`, and
`Documentation`.

## Source files

Source files contain typed language AST items. They do not contain preamble,
body, or epilogue documents.

```rust
struct TargetSourceFile<D: TargetDialect, S: PackageState> {
    id: TargetFileId,
    relative_path: RelativeOutputPath,
    role: SourceRole,
    module: D::ModuleDeclaration,
    items: Vec<D::TopLevelItem>,
    resolution: S::ResolutionData,
}
```

Package declarations, module attributes, pragmas, include guards, and linkage
blocks are typed dialect nodes or resolved file metadata.

## Non-source files

Documentation, metadata, and binary/text assets may contain validated raw
contents. Their role types cannot be converted into source roles.

Package manifests such as `Cargo.toml`, `package.json`, `pyproject.toml`, and
`go.mod` use typed metadata models and dedicated renderers/templates rather
than arbitrary source-file nodes.

## Paths

All paths are normalized relative paths. Constructors reject:

- absolute paths;
- drive/UNC prefixes;
- empty components;
- `.` and `..` components;
- NUL/control bytes;
- target-ambiguous separators;
- reserved output collisions; and
- duplicate normalized paths.

## File placement

Each language specification defines deterministic placement for:

- public declarations;
- private implementation;
- runtime helpers;
- interface witnesses;
- tests and conformance;
- C/C++ declarations and definitions;
- negative type-check fixtures; and
- metadata.

Placement is decided before rendering. Templates cannot move declarations.

## Dependency graph

Cross-file references create typed file-graph edges. The resolver validates:

- target-permitted cycles;
- declaration-before-use constraints where applicable;
- header/module import direction;
- public API dependency visibility;
- runtime-to-user dependency prohibition; and
- test-only dependency isolation.

## JavaScript derivation

JavaScript executable source is produced only by compiling the resolved and
rendered TypeScript source with the pinned TypeScript compiler. It is not
represented by an independently lowered executable package.

The JavaScript package assembler may add derived metadata and copy compiler
outputs, but cannot rewrite executable JavaScript or add semantic helpers.

## Required proof

- Missing/duplicate/cyclic helper tests.
- One-feature-at-a-time helper presence/absence matrices.
- Runtime AST source-policy tests.
- File role compile-fail tests.
- Invalid and colliding path matrices.
- Cross-file cycle and visibility tests.
- Minimal programs emit no optional runtime helpers.
- Package manifests contain exact derived dependencies.
- Generated JavaScript source hashes match compiler output.
- Three complete package graphs and rendered artifact trees are identical.
