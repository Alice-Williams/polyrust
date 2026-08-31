# Structured diagnostics contract

This document defines the initial stable diagnostic model shared by IR readers,
the checker, backends, and safe output handling. Diagnostic codes and JSON field
names are compatibility surface from v0.1 onward.

## Model

A `Diagnostic` contains, in serialized field order:

1. stable `DiagnosticCode`;
2. `error`, `warning`, or `note` severity;
3. concise message;
4. zero or more primary or secondary labels;
5. zero or more related locations;
6. notes;
7. an optional remediation hint; and
8. optional target/backend context.

Each label has a `SourceRef`, style, and message. A file source is a half-open
UTF-8 byte range `[start, end)` in a portable file label. A logical source is an
ordered builder path such as `module(example) > record(User)`. Consumers may
construct and compare the model through the `code` and `model` modules without
importing the terminal renderer module or any terminal UI dependency.

## Initial code registry

| Code | Variant | Short explanation |
| --- | --- | --- |
| `P0001` | `UnsupportedIrMajor` | unsupported IR major version |
| `P0102` | `DuplicateDeclaration` | duplicate declaration |
| `P0207` | `TypeMismatch` | type mismatch |
| `P0214` | `NonExhaustiveMatch` | non-exhaustive match |
| `P0220` | `ContractNonconformance` | contract implementation does not conform |
| `P0230` | `InvalidPortableTest` | invalid portable test |
| `P0301` | `ImpureOperation` | impure operation |
| `P0404` | `UnsupportedCapability` | target capability unsupported |
| `P0502` | `UnsafeOutputPath` | unsafe output path |

`DiagnosticCode::ALL` is the centralized registry. The
`explain(DiagnosticCode)` API returns a non-empty short and long explanation
for every entry. Tests reject duplicate code strings or missing explanations.

## Determinism and rendering

`sort_diagnostics` orders diagnostics by their first label's source and then by
code. File labels sort by file, start, and end; logical sources sort by their
ordered path. Callers that require stable collections must sort before rendering.

`render_json` emits compact UTF-8 JSON using the model's declared field order.
ANSI control bytes are escaped by JSON serialization and are never inserted by
the JSON renderer.

`render_terminal` emits LF text with optional ANSI color. The options API can
emit either LF or CRLF output. Source frames:

- clamp all `u64` offsets to the available source;
- move offsets inside a UTF-8 scalar to the preceding valid boundary;
- turn reversed ranges into a safe zero-width range;
- mark a zero-width range with one character; and
- report unavailable source content without failing.

These rules make rendering safe for untrusted spans, Unicode filenames and
source text, oversized offsets, and missing source files. User-visible strings
are data, so callers that embed terminal output in another protocol remain
responsible for that protocol's escaping requirements.

## Stable JSON shape

The initial compact shape is:

```json
[
  {
    "code": "P0207",
    "severity": "error",
    "message": "expected String",
    "labels": [
      {
        "style": "primary",
        "source": {
          "kind": "file",
          "data": {"file": "example.poly", "start": 0, "end": 4}
        },
        "message": "found integer"
      }
    ],
    "related": [],
    "notes": [],
    "hint": "convert the value",
    "target": "rust"
  }
]
```

The source object uses the same tagged `SourceRef` schema as portable IR v0.
