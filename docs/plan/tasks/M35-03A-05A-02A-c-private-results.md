# M35-03A-05A-02A — C private scalar-result value transport

- Status: complete
- Parent: [C results](M35-03A-05A-02-c-results.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

Admit by-value arguments and returns for source-private functions using a
complete nominal struct containing exactly an unqualified Bool followed by I32.
Authenticate registered owner/member identity; no name-based recognition.
Do not admit pointers, unions, nested structs, different widths, incomplete
records, foreign records, or public aggregate signatures. Definitions and
calls retain closed effect, initialization, sequencing and frame checks.
Require the struct declaration before any signature using it.

This is a C representation proof only, not authentication of a Rust Result or
permission to read its inactive source payload. The source lowerer remains
responsible for canonical Err construction and tag-guarded source observations.

## Definition of done and tests

Build actual typed C AST with private construct/copy/return/inspect helpers and
a scalar external entry point. Certify and render it; strict GCC14 and Zig at
O0/O2 plus UBSan must preserve both tags and I32 min/zero/max values. Negative
controls reject unsupported shapes, public signatures, declaration-order
violations and cross-registry/nominal mixing. Verify complete initialization,
call effects, recursion rejection and conservative output/frame accounting.
Full Linux release/lint, preserved output/WIP and fresh GPT-6-SOL review precede
commit/push. No public dependency ABI or compiler admission in this checkpoint.

## Verification history

The initial fixture attempted a nested call. The unchanged sequencing verifier
rejected it; the fixture now materializes both intermediate aggregate values.
The swapped-tag fault initially used C logical-not directly as a Bool field.
The typed builder correctly rejected its Int result; an explicit Bool conversion
makes the faulty program compile so the native behavioral oracle can reject it.
Neither failure was worked around by weakening production checks.

Reviewed implementation tree 7b4639ae3ae194aa94f42fd8339783fa609d57c3 includes
four focused unit tests. The native test compiles the actual certified renderer
output with GCC14, Zig and GCC14/UBSan at O0/O2. Each correct executable checks
262,168 observations: both tags, tag/payload inspection, all I16 payloads and
six I32 edge/sentinel values. Twelve compiling faulty executables must exit
with the exact swapped-tag or lost-payload failure code, without sanitizer
diagnostics. Native frame reports and private/public symbol linkage are checked
against the measured source call graph; automatic objects total exactly 60
bytes and the admitted struct layout is exactly 8 bytes/alignment 4.

A broad GPT-6-SOL extra-high review of the exact implementation and surrounding
signature, storage/effect, projection, rendering, resources and dependency
paths found no actionable correctness or safety gaps. The exact implementation
passes all 1,039 Linux Bazel release/lint tests: 131 rerun and the remainder
cached. This includes 862 C unit/native cases and the nine separately targeted
large C suites. Existing 530 generated files, four recent character-constant
bundles and all 45 unrelated WIP files are byte-identical. No tests were disabled,
no legacy runtime was removed, and no public/source aggregate admission was added.
