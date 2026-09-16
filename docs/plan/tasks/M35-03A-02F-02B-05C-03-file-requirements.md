# M35-03A-02F-02B-05C-03 — Checked symbol-independent file requirements

- Status: planned
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-02

## Contract

A source file may require another generated file without consuming any symbol.
C alias-only packages require their own public header this way. Do not fabricate
a function/value merely to cause an include. Represent the dependency using the
target module handle and validated output path, resolve it through the shared
file catalogue, and retain the checked edge separately from named symbols.

Introduce TargetFileRequirement<D> (module: D::ModuleDeclaration, path:
RelativeOutputPath) with private fields and read-only accessors. It is unresolved
data, not authority. LinkerDialect::file_requirements(&TargetFile<Self>) returns
these requirements (empty by default); plugins derive them from their typed AST.
The shared linker locates the exact path and module together, rejects absent,
wrong-owner and self references, deduplicates equal edges, applies role visibility
and cycle checks, and calls the existing resolve_file_import hook.

Reference-derived and file-only edges share one canonical dependency graph.
Independent post-link verification rederives both classes from unresolved input
and rejects missing, extra or retargeted edges/directives. File-only requirements
never allocate fake symbol names, introduce dependencies on arbitrary filesystem
contents or bypass private implementation visibility. Other dialects retain the
empty default and unchanged behavior.

C derives the implementation-to-public-header requirement from its explicit
CSourcePackage registration. The public header has no reverse requirement.
Generated includes remain typed CGeneratedHeader values and use the structural
renderer. Source and header need no duplicated declarations. Alias-only package
publication itself remains disabled pending 05C target-export evidence.

## Definition of done

- Shared tests: zero-symbol two-file packages link with exact checked edge,
  duplicates deduplicate, bad module/path/self/visibility/cycle requests fail.
- Reconstruction mutations: removing/adding/retargeting a linked edge or include
  fails certification; ordinary symbol-derived imports remain unchanged.
- C tests: explicit source-package header edge is present, file ordering is
  irrelevant, no duplicate include when an owned symbol already requires it.
- Existing Java/other default dialect output stays unchanged.
- Full Linux Bazel release/lint/native regression gates and independent review.
