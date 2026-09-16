# M35-03A-02F-02B-05C-03 — Checked symbol-independent file requirements

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-02
- Specification: [typed file requirements](../../specification/typed-generation/file-requirements.md)

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

Implementation separates file-graph and file-requirement logic into focused
modules rather than growing the existing linking.rs monolith. Count all explicit
requests, including duplicates, against a 100,000-request package budget before
resolving them. Use iterative cycle detection; deep finite file chains must not
consume the native call stack. A selective cycle policy must check every simple
directed cycle, including disconnected and overlapping cycles. Bound the exhaustive
selective-policy traversal to 100,000 root/edge steps and reject on exhaustion;
the default all-forbidden policy and acyclic graphs retain a fast linear walk. A requirement is distinct from a named symbol
reference and cannot be used to bypass symbol visibility or bind a private value.

## Definition of done

- Shared tests: zero-symbol two-file packages link with exact checked edge,
  duplicates deduplicate, bad module/path/self/visibility/cycle requests fail.
- Reconstruction mutations: removing/adding/retargeting a linked edge or include
  fails certification; ordinary symbol-derived imports remain unchanged.
- C tests: explicit source-package header edge is present, file ordering is
  irrelevant, no duplicate include when an owned symbol already requires it.
- Existing Java/other default dialect output stays unchanged.
- Full Linux Bazel release/lint/native regression gates and independent review.

## Review and focused evidence

The first independent Sol Extra High review found one latent shared-contract
defect: only the first discovered cycle was checked against a selective dialect
policy. Although inherited from the old traversal and unused by current C, this
was accepted as a core finding and fixed, not dismissed as an extension.

The corrected validator checks canonical simple directed cycles with fail-closed
work accounting. Tests include disconnected and overlapping selective policies,
independent post-link reconstruction, and comparison with an independent cycle
inventory across all 65,536 directed four-vertex graphs.

A fresh Sol Extra High review of implementation tree
`3e61cdaa45b286cd4bbc363fc5197c87bae7eafe` found no remaining core errors.
Its optional request for another integration-level budget-diagnostic assertion
was not treated as a blocker: unit tests exercise the actual traversal budget
error, production maps that enum directly to TargetResourceLimit, and both linker
and reconstruction call the same validator. No error report was waived.

Focused Linux Bazel proof on that exact tree:
`//crates/codegen:target_linker_test`,
`//crates/codegen:portable_codegen_test`, and
`//crates/backend-c:portable_backend_c_unit_test`: **3/3 passed**, 87.489 seconds,
invocation `a87d1f40-aca0-46e9-b76b-b21403afaefe`.
The superseded pre-fix full run was intentionally interrupted and is not counted
as completion evidence.

Complete Linux dev-container proof on implementation tree
`3e61cdaa45b286cd4bbc363fc5197c87bae7eafe`:
`bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test //... //:release_gate --noshow_progress --noverbose_failures --test_output=errors --test_summary=terse --keep_going`
passed **738/738 tests**, 1,093 targets, 121 executed and 617 cached,
709.104 seconds, invocation `cc349a36-11c1-4994-bcbd-a7f561dd93cd`.
This includes Rust/Bazel linters, all native generated-language checks, and the
separately scheduled long C capacity cases; none were disabled or relaxed.
The shared codegen suite passed 137 cases and the C unit suite passed 754 cases
(with its five expensive cases covered by their existing separate Bazel targets).

All 20 preserved ownership-work hashes remained unchanged. The next task's
draft test fixtures are excluded from this checkpoint. Constant re-export
publication and legacy runtime retirement remain unfinished.
