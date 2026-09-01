# Compositional target-language IR contract

Status: normative for M30 and every later backend change

This document defines the mandatory boundary between checked PolyRust semantics
and rendered target source. The key words **MUST**, **MUST NOT**, **SHOULD**, and
**MAY** are normative.

## 1. Required pipeline

Every source backend MUST implement this one-way pipeline:

```text
CheckedProgram
  -> target translator
  -> dependency-complete target fragments
  -> runtime-helper dependency closure
  -> language files and file groups
  -> syntax-only renderer
  -> OutputManifest
```

The `CheckedProgram` is the last target-independent representation. A target
translator MAY use a target-specific AST or the shared `Document` algebra, but
its result is target-language IR, not rendered source text. The renderer MUST
not inspect checked IR, infer features, select helpers, or repair dependencies.

## 2. Normative data model

The implementation names may evolve, but the following responsibilities MUST
remain distinct.

```text
LanguageFragment<Import, HelperId>
  syntax: Document
  imports: ImportSet<Import>
  helper_roots: ordered set<HelperId>

RuntimeHelper<Import, HelperId>
  id: HelperId
  fragment: LanguageFragment<Import, HelperId>
  helper_dependencies: ordered set<HelperId>

LanguageUnit<Import, HelperId>
  composed fragment for one file section

LanguageSourceFile<Import, HelperId>
  path, role, preamble, body, epilogue, render options

FileGroup
  stable package-layout grouping of complete files
```

A fragment is the smallest dependency-ownership boundary. Any mapping that
introduces target syntax MUST return its syntax, direct imports, and direct
runtime-helper roots together. A caller MUST be able to compose the fragment
without knowing which target constructs it contains.

Fragment composition MUST merge all three components. Empty, optional,
sequence, joined, indented, grouped, and nested composition MUST preserve
requirements. Composition MUST be associative with deterministic output.

A language unit is a closed file section made only by composing fragments. It
MUST NOT expose `set_document`, `require_import`, `require_helper`, or an
equivalent repair path after construction. A source file may acquire
requirements only by accepting closed units.

File groups describe package layout and deterministic ordering. They MUST NOT
be used as a dependency side channel or as a substitute for fragments.

## 3. Dependency ownership rules

### A1 — Mapping-local ownership

Type, literal, expression, declaration, portable-test, preamble, and helper
mappings MUST return dependency-complete fragments. A mapping MUST NOT return a
naked `String` or `Document` while requiring its caller to remember an import or
helper.

### A2 — Structured imports

Import keys MUST represent semantic import data such as module, qualified name,
symbol, alias, visibility, type-only status, and system/local class. They MUST
NOT contain a complete rendered directive. Constructors MUST reject invalid
names. Only the target renderer may spell `use`, `import`, `from`, `#include`,
or an equivalent dependency directive.

### A3 — No parallel dependency scan

A backend MUST NOT traverse declarations, types, values, capabilities, or
rendered text solely to reconstruct requirements after syntax was produced.
Requirements MUST arise in the mapping that produced the dependent syntax.
Feature analysis for semantic lowering is allowed; repeating it only to repair
imports is not.

### A4 — Helper graph

Every runtime/support declaration MUST belong to a stable helper node. A helper
node owns its syntax, imports, and direct helper dependencies. Program and test
fragments select helper roots. The backend computes one deterministic transitive
closure, rejects missing nodes and cycles, and emits each selected helper once
in stable topological order.

A monolithic runtime MAY remain one node only when it is genuinely irreducible:
every declaration and import in that node must be required whenever the root is
selected, and a minimality test must prove that claim. Copying an all-features
runtime and attaching a fixed import list is non-compliant.

### A5 — Source-role closure

Files with roles `Source`, `Runtime`, `Test`, `Conformance`, or `NegativeTest`
MUST be `LanguageSourceFile` values built from closed units. Raw `Text` files
are allowed only for `Metadata`, `Documentation`, and text `Asset` roles. This
prevents a runtime or declaration file from bypassing dependency collection.

### A6 — Syntax-only rendering

The renderer receives only the already-resolved language file and structured
requirements. It sorts, deduplicates, groups, and spells directives, then
renders documents. It MUST NOT receive `CheckedProgram`, capability data, or a
runtime-helper registry. A file with no import requirements MUST have no import
section.

### A7 — No directive text in bodies

Generated body templates and mapping-produced raw text MUST NOT contain import,
include, or use directives. Handwritten upstream fixtures and native consumer
fixtures are allowed only through an explicit policy-test allowlist. Package and
namespace declarations, include guards, comments, and language pragmas are not
dependency directives, but still belong to structured preamble fragments.

## 4. Translation responsibilities

The target translator owns all semantic and target-policy decisions:

- identifier allocation and escaping;
- target type and literal selection;
- expression, declaration, and portable-test lowering;
- direct import and helper-root selection;
- runtime-helper registry construction;
- file roles, paths, and groups; and
- declared package dependencies.

The shared generator only validates package structure, resolves helper graphs,
merges requirements, invokes the renderer, and creates the manifest. It MUST
not contain branches for Rust, TypeScript, JavaScript, Python, Go, Java, C++, or
C syntax.

JavaScript remains derived from the TypeScript target implementation. Its
fragments MAY erase type-only syntax and imports, but MUST preserve the same
helper ownership and runtime behavior.

## 5. Verification contract

A backend is compliant only when all of the following are permanently tested:

1. fragment composition preserves imports and helper roots through empty,
   optional, sequence, joined, indented, grouped, and nested cases;
2. imports and helper roots are sorted and deduplicated deterministically;
3. invalid structured import keys, missing helpers, and helper cycles fail with
   stable diagnostics;
4. a minimal checked program emits no unrelated import or helper;
5. a one-feature-at-a-time matrix proves exact import and helper presence and
   absence for every dependency-bearing mapping;
6. a source-policy test rejects dependency directives outside renderers and
   explicit fixture allowlists;
7. three complete generations are byte-identical; and
8. target formatters, linters, compilers, native tests, public consumers, and
   applicable ABI/sanitizer tests pass.

Compilation alone is not proof of compliance: an unconditional catch-all import
block can compile while violating mapping-local ownership and minimality.

## 6. Change protocol

Adding a portable feature that needs target dependencies requires, in order:

1. target-independent semantics and checker/evaluator coverage;
2. one dependency-complete mapping fragment per target;
3. helper nodes and graph edges, when support code is required;
4. exact presence/absence matrix cases;
5. native equivalence tests in all supported targets; and
6. an update to the compliance ledger when an invariant or admitted exception
   changes.

No code-review approval or milestone completion may waive this contract without
a new architecture decision record that changes the contract explicitly.
