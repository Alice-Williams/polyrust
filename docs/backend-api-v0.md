# Backend API v0

Status: frozen for the v0 required backends (M08).

## Boundary

Every backend implements `portable_codegen::Backend`. Its generation method
receives only an immutable `portable_check::v0::CheckedProgram`; unchecked IR
has no safe route into this API. The application owns a `BackendRegistry`, so a
new backend is registered without edits or target-name branches in core crates.

Target IDs are open, lowercase namespaced text such as
`org.example.experimental-language`. They are not a closed enum. A descriptor
also carries the backend version and its inclusive compatible IR range.

## Generation protocol

The registry performs these deterministic steps before invoking a backend:

1. verify that the checked document version is inside the descriptor's range;
2. validate every supplied option against the backend's stable schema;
3. compare the complete checked-program capability report with the backend's
   `native`, `helper`, or `unsupported` declarations; and
4. report every unsupported capability, including requiring node IDs and target
   context, without invoking generation.

`check_backend_contract` is the reusable external-backend contract kit. It
checks descriptor, schema, support-table, and repeated generation stability.

## Manifest safety

Successful generation returns a fully constructed `OutputManifest` containing
explicit UTF-8 text or byte files, declared dependencies, and injected-helper
metadata. Construction validates and then sorts all entries. It rejects empty,
absolute, rooted, drive-prefixed, UNC, backslash, repeated-separator, dot and
parent traversal, control-text, Windows device, trailing-dot/space, reserved
metadata, duplicate, and Unicode case-fold-colliding paths.

The manifest API has no filesystem handle or output-directory argument. M09 is
the only layer permitted to materialize a validated manifest.

Source backends additionally use the compatible language-plugin path:
their translator consumes `CheckedProgram` and produces a validated generic
`LanguagePackage` from dependency-bearing fragments, helper closure, and closed
source files. A syntax-only renderer then flattens its file groups and dynamic
import requirements into the same `OutputManifest`. `SourceFileRole` and
`TextFileRole` make raw source-role construction impossible through the public
API. This is a separation beneath the frozen `Backend::generate` entry point,
not a second way to pass unchecked IR to generation.

## Compatibility freeze

M10-M13 may extend target-owned option schemas and metadata values, but must not
bypass the registry's version/options/capability sequence, accept unchecked IR,
or mutate the filesystem during pure generation. Any breaking change to the
types or sequencing above requires an architecture decision record.
