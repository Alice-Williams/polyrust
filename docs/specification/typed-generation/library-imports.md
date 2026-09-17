# Typed unnamed standard-library imports

A library directive may be needed without a named type, callable or field.
C platform macros are one example. Never invent a fake type merely to trigger
an include and never attach raw directive strings to the renderer.

## Plugin contract

LinkerDialect::file_standard_libraries derives standard-library identities
from a checked TargetFile and its typed items. Its default returns no
requirements. resolve_standard_library_import maps one identity to an
ImportKind, or returns a diagnostic. The default resolver rejects requirements.

The shared linker deduplicates and orders requirements by their typed
identities. ResolvedLibraryImport retains both StandardLibrary and ImportKind
with no public constructor or mutable fields. It does not reserve a symbol
name or create a generated-file edge. It is distinct from a symbol import or
a generated-file dependency; these keep their existing checks.

## Certification and rendering

The verifier reconstructs the complete list from the retained unresolved
package and compares exact identities, order, multiplicity and kinds.
Linked claims are not authority. A package with any inserted, removed,
duplicated or rewritten library directive cannot become RenderReadyPackage.

A consuming backend must render these witnesses through its normal import
printer and account for them in output/resource bounds. StructuralImportRenderer
receives separate typed slices for symbol imports, generated-file imports and
unnamed library imports. Directive spelling stays inside that renderer contract;
a backend combines and deduplicates only their already-resolved import kinds. The shared linker
does not infer that an arbitrary backend printer consumes a new import kind;
each opting-in plugin must prove its own implementation and tests. Backends
with the default empty policy preserve their existing output.

No user-provided library names, paths, aliases or source text are introduced
by this API. A plugin remains responsible for authenticating its file AST and
for mapping its closed library inventory into legal target syntax.
