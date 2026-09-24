# M35-03A-05A-02 — Certified C scalar-result transport

- Status: complete
- Parent: [05A](M35-03A-05A-scalar-results.md)
- Depends on: [identity probe](M35-03A-05A-01-result-identity.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

Ordered checkpoints:

1. [02A: private value transport](M35-03A-05A-02A-c-private-results.md).
2. [02B: public nominal ABI](M35-03A-05A-02B-c-public-results.md).
3. [02C: measured selected-arm execution](M35-03A-05A-02C-c-result-branches.md).

02A establishes target-level layout/call safety, not source Result identity.
02B and 02C must complete before claiming this parent or compiler result admission.

Extend the certified C public signature/dependency profile narrowly for a
source-derived complete result struct passed/returned by value. Do not simply
relax the scalar-signature allowlist: authenticate layout ownership, header
completeness, member identity, imported aggregate identity, frame/call budgets
and transitive dependencies. Define explicit success/error construction and
guarded observation with ordinary typed C AST.

## Tests and definition of done

Strict standalone headers and separately compiled producer/relay/consumer
packages agree on tags and exact I32 payloads, copies, returns, fallback and
effects under GCC14/Zig O0/O2 and UBSan. Inject wrong nominal/layout/member/
owner/dependency facts and unsupported pointer/union/nested/heap layouts;
reject before rendering. Bound recursion, metadata, frames and output. Native
faults must expose swapped tags, lost payloads and eager inactive-arm effects.
Full Linux release/lint, fresh review and separate commit; no compiler admission.

## Completion

02A, 02B and 02C are complete. Their combined evidence includes exact public
nominal imports and measured value-preserving execution-order faults. All 1,041
Linux release/lint targets pass at the 02C checkpoint, independent review is
clean, and existing output/WIP is unchanged. This closes the bounded C target
transport foundation; it does not admit Rust Result syntax or retire legacy code.
