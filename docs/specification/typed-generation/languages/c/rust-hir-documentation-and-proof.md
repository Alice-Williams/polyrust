# Rust documentation attributes and C lowering proof

- Status: normative M35 design; single-unit documentation implemented, public API placement follows M35-01D
- Parent: [C Rust-source lowering](rust-hir-lowering.md)

## Documentation boundary

Preserve the textual contents of resolved Rust `doc` attributes on emitted
modules, types, fields, functions and, when supported, trait/interface members.
This includes the equivalent `///` and `//!` forms and compiler-expanded
text attributes. Read the compiler's attribute representation; do not parse
Rust source again or reconstruct discarded ordinary comments.

Ordinary `//` and `/* ... */` comments, whitespace and source formatting
are explicitly out of scope. Rustdoc presentation controls such as hidden,
aliases and link configuration do not become C semantics or control API
visibility. Do not claim Rustdoc-to-Doxygen semantic equivalence.

Keep source documentation attached to its authenticated owner, in attribute
order. Preserve original text in source metadata; lower its output form using
the existing `CComment` normalizer. This normalizer intentionally makes
hostile text safe, including terminators, backslashes, trigraph hazards and
non-ASCII bytes. The C output is normalized documentation, not a byte-identical
copy of the Rust attributes.

The initial output uses canonical ordinary C documentation comments. There is
no executable documentation template and no automatic translation of embedded
Rust code examples or intra-doc links. Literal textual content is documentation,
not executable C.

## Placement using existing types

| Documentation owner | C destination |
| --- | --- |
| Module/crate | Comment associated with its module inside the owning crate's file group |
| Public emitted function/type | Comment immediately before its primary public declaration |
| Private function/type | Comment immediately before its owning private declaration/definition |
| Struct field | CComment attached to the exact CMemberRef and rendered adjacent to that member |
| Interface method, when mapped | Documentation associated with its registered method/table binding |

Use `CComment`, `CFileItem::Comment` and existing registered references.
File-level comment nodes already exist. Field/declaration attachments need a
typed metadata extension: preserve owner association through declaration
ordering and linking, rather than inserting free-floating comments early and
hoping they remain adjacent. Do not introduce another comment text type with
weaker escaping or use a map keyed by member-name strings.

For the certified shared profile, the typed attachment inventory is part of
CProjection, alongside the existing immutable C tree and registry. Its closed
owner enum uses CStructRef, CMemberRef, CFunctionRef or a module's compiler
declaration identity plus owning CFileRef. Projection lowers original metadata
to CComment exactly once, and independent reconstruction checks the complete
inventory. The renderer only places those normalized comments; it does not
interpret Rust metadata, resolve owners, read files or normalize text.

Declaration provenance carries compiler-derived root-to-owning-module ancestry,
including resolved module attributes. Conflicting descriptions of one module,
broken parent chains, foreign crate identities and missing owning modules are
diagnostics. Module documentation is deduplicated by identity, never spelling.
Every emitted attribute (including empty text) consumes a node and its normalized
bytes consume the existing comment/source budgets before certification.

Each crate has exactly one root module and each emitted source declaration has
exactly one primary typed owner. Conflicting roots, module/declaration identity
collisions and duplicate function prototypes are rejected, including when their
documentation is empty. These metadata checks establish internal consistency;
manually constructed provenance is not evidence that rustc checked a program.

Compiler extraction shares immutable module payloads and complete ancestry
chains using Arc. A wide record must not clone its module text once per field.
Before copying resolved text, extraction admits at most 16 MiB of original
documentation and 100,000 attributes, with ancestry depth limited to 128.
These preliminary allocation guards do not replace the stricter existing C
node, normalized-comment and rendered-source resource checks. They do not bound
rustc's own parsing or macro-expansion memory.

If an item is emitted more than once, choose its primary declaration
deterministically and do not duplicate its entire documentation at every use.
Apply the [crate and visibility contract](rust-hir-files-and-visibility.md):
comments retain module/declaration ownership after file flattening and cannot
move an item across crates or widen its access.
A private concrete record must not be exposed publicly merely to display its
field documentation. Items excluded from generation do not acquire orphan
comments in output.

Included documentation files must be declared compiler/Bazel inputs. Unsupported
attribute text evaluation is diagnosed rather than resolved with arbitrary
renderer-side file reads. The adapter takes explicit `--input PATH` arguments
for dependencies beyond its canonical source root. After successful compiler
analysis it checks tracked file dependencies and non-imported real SourceMap
files against that canonical allowlist, and rejects tracked environment
dependencies. This covers text includes, source includes and out-of-line
modules; it is a build-input contract, not an operating-system sandbox. rustc
has already read the inputs. A failure occurs before opening the output file.

## Required evidence

Each implementation task must supply named Bazel tests for its obligations:

1. Compiler boundary: reject wrong types, use after move, conflicting/escaping
   borrows, unsafe input, non-Rust entry ABI and unimplemented source forms.
   Exercise errors in non-entry functions too. Ambient RUSTC_BOOTSTRAP cannot
   enable unstable customer input. Failure neither creates nor modifies output.
2. Existing-type integration: compile-fail tests prevent forged source proof,
   wrong-registry references, cross-owner fields, detached scopes, wrong
   signatures and rendering before certification. Source-level checks assert
   no second C model, raw syntax path or legacy-generator fallback.
3. Mapping identity: same-spelled distinct structs/locals/methods, shadowing,
   reordered field initializers, aliases, nested shared references and rejected
   overloaded/reference comparisons. Assert typed mapped structure as well as
   emitted text.
4. Structured control: nested braces/scopes, both branches, integer boundaries,
   explicit sequencing and no goto/label emission from the Rust-source path.
   A mutation which flattens an inner CBlock must fail its regression.
5. Documentation: equivalent /// versus doc attributes, inner/module docs,
   repeated attribute order, fields with identical names in different types,
   empty/multiline text, macro-expanded text and negative undeclared includes.
   Hostile text containing */ or preprocessing hazards must remain comments.
   Ordinary comments do not enter output; repeated output is byte-identical.
6. Native evidence: same inputs run against native Rust and strict C17 at
   O0/O2, separate public-header consumer, deterministic rendering and style
   checks. Run the existing pinned compiler/platform probes; add GCC
   ASan/UBSan gates for admitted memory mappings, including shared borrows.
7. Tooling: Bazel is authoritative in the Linux container. Run Rustfmt,
   Clippy, Buildifier and documentation checks. Recursive source inputs must
   include split modules in both compilation and formatting action keys.
8. Integration: preserve the existing historical-port/release gates. Document
   exact invocations, limitations, generated artifact paths and review findings.
   Generated output remains ignored, never committed.

Passing a handful of source fixtures does not establish all-Rust support or
complete portable capability support. Add accepted shapes only alongside
closed admission, typed mapping and permanent native/negative proof.
