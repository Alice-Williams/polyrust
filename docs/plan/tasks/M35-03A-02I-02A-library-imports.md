# M35-03A-02I-02A — Typed unnamed standard-library imports

- Status: complete
- Parent: [C/Java binary64 target values](M35-03A-02I-02-target-values.md)
- Specification: [library imports](../../specification/typed-generation/library-imports.md)

## Contract

C binary64 platform properties need float.h without a named typedef or callable.
Add a general typed, reference-free standard-library directive to the shared
linker. Do not pretend a macro is a type, add a copied include preamble, or
weaken symbol-bound imports. This prerequisite is independently committed
before C starts consuming it. Other backends keep the default empty policy.

## Definition of done and tests

- Each directive retains its typed standard-library identity and target import
  kind in a private-constructed witness, separate from name allocation.
- Canonical deduplication and ordering come from typed identities. The backend
  derives requirements from checked original file metadata.
- Certification reconstructs requirements; omitted, injected, duplicate,
  changed-origin and changed-kind directives fail.
- Unsupported required libraries return diagnostics, not silent omission.
- Positive generic linker tests show unchanged name bindings and deterministic
  output metadata. Compile-negative documentation rejects forged witnesses.
- Full Linux Bazel release/Rust/Bazel-lint gate and fresh independent Sol Extra
  High review pass before commit/push. No target float admission is claimed.

## Scope boundary

The C consuming integration, import rendering/resource accounting and native
binary64 tests remain work under the parent, not evidence for this prerequisite.

## Proof receipt

Exact implementation tree: `bdfb2e9d65a4c9acf82d2315506d159aea48d33a`.
The Linux dev-container Bazel gate passed 780/780 tests across 1,176 targets
(129 executed, 651 cached), including release and Rust/Bazel linters.
Invocation: `e155595b-9ff5-434f-9687-2791a0c26edb`.
The shared linker suite passed all 145 unit tests, including three new
positive/reconstruction/failure-policy tests; the witness privacy doctest
also passed. Existing C and Java backends retain their empty hook defaults.

A fresh independent Sol Extra High review of this exact tree against
`d0eb4694f9cfef9a07c179557ac4756b8f5b876a` found no core findings.
The review suggested an optional two-distinct-library ordering fixture.
This is non-blocking: construction uses an ordered set of typed identities
and certification compares the complete ordered vector, so reordered claims
already fail by construction. The existing focused mutations cover omission,
injection, duplication, wrong origin and wrong kind. Multi-library fixture
expansion remains useful test hardening, not missing supported behavior.
