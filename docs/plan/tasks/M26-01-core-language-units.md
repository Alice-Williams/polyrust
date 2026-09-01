# M26-01 — Core dependency-bearing language units

- Status: complete

## Goal

Introduce the flat target-language unit and make it the only way translated
source syntax and its imports enter a language source file.

## Definition of done

- `LanguageUnit<Import>` owns a `Document` and an `ImportSet<Import>`.
- `LanguageSourceFile` accepts units for all source sections.
- There is no public file-level import mutation API.
- Unit imports merge deterministically into the owning file.

## Tests

- Merge imports from preamble, body, and epilogue units.
- Deduplicate repeated requirements within and across units.
- Omit the import section when every unit has an empty requirement set.
- Preserve duplicate-path, invalid-group, renderer-error, and limit tests.
