# M35-03A-05A-01 — Compiler identity for fallible scalar values

- Status: complete
- Parent: [05A](M35-03A-05A-scalar-results.md)
- Specification: [shared](../../specification/typed-generation/rust-scalar-results.md)

## Contract

Observe checked compiler signatures for the closed standard
Result<i32, TryFromIntError> instance. Authenticate Result, its Ok/Err variants
and field types through compiler identities. Derive the error type from the
standard TryFrom<i64> for i32 associated Error projection, not a path string.
Anchor all standard items to the same external core crate as compiler language
items. Retain opaque source type/variant identity in private compiler-session
data. This is observation only, not a target certificate or source admission.

## Definition of done and tests

Real checked source with direct, renamed and normalized alias spellings observes
the same original instance; an observation of aliases does not admit them into
production. Test no_std source too. Reject same-spelled local Result, wrong
success/error type, other instantiations, generic signatures and borrowed
results. Reject invalid Rust before observation. Prove private construction
with a compile-fail target. Verify no drop obligation for the admitted instance.
Keep the probe backend-independent. Full Linux Bazel release/lint, fresh broad
review and preserved existing outputs/WIP precede separate commit/push.

## Verification history and review

The first probe build exposed pinned compiler API changes: projection creation
requires an explicit rigidity flag and the normalization wrapper. A direct
type_of query on the trait's associated Error declaration has no metadata value.
The corrected probe constructs an unnormalized projection and requires rustc
normalization; it never skips normalization. Observation output uses stable
declaration hashes, not context-dependent Debug type names (std/no_std differ).

Tree f16bcf55465f861a636f6b3afd205db4a822d0d3 passes five focused targets and
all 1,039 Linux release/lint tests. A first GPT-6-Sol extra-high review suggested
a mixed input/output instantiation negative targeting same_instance. The exact
finding is not accepted: observe independently restricts each side to the same
single canonical instance; an i64 output is already rejected before comparison,
so deleting that comparison alone cannot admit it. The comparison remains as
defense in depth. We accept the useful broader test-hardening suggestion: add
wrong output success/error payload cases and an explicit associated-projection
positive. Hardened tree 31395fbd62ba7b97a12d3d0b8818e36f54fbd31b passes
all 1,039 release/lint tests (four executed, remaining results cached). A fresh
GPT-6-SOL extra-high read-only review of all 13 changed files found no actionable
correctness, security or proof gaps. Its coverage includes six positive sources,
ten rejection controls and the private-witness compile-fail test. Existing 530
output files, four recent character-constant bundles and 45 unrelated WIP files
remain unchanged. This completes compiler identity observation only; C/Java
transport and production source admission remain separate planned checkpoints.
