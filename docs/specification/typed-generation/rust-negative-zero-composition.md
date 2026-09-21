# Runtime-free negative-zero composition

- Status: implemented and independently reviewed; 903-target gate verified
- Plan: [02P](../../plan/tasks/M35-03A-02P-negative-zero-composition.md)
- Source: existing Apache-2.0 stdlib is-negative-zero 0.2.3 snapshot under
  third_party/stdlib-is-negative-zero, with unchanged license/provenance gates.

## Shared source contract

An ordinary public Rust function accepts f64 and returns bool. It compares the
value to zero and, only for zero, compares its reciprocal to zero:
value == 0.0 && reciprocal(value) < 0.0. The reciprocal is a private,
source-owned ordinary function returning 1.0 / value. Only raw bits
0x8000000000000000 satisfy the predicate. Positive zero, finite nonzero values,
both infinities and every NaN category return false.

This covers the upstream declared numeric API, not JavaScript non-number inputs.
The upstream uses reciprocal equality to negative infinity; after the zero
guard, comparison to zero is equivalent and does not require floating constants
or special source recognition. Floating exception flags and foreign rounding
modes remain outside the pinned profile. No NaN payload/sign preservation is
needed for a Boolean observation.

No new capability is registered. Canonical primitive comparison, lazy Boolean
conjunction, ordinary calls, binary64 literals and division use their existing
typed executable bindings. Existing unsupported-source boundaries remain closed.
Function names and source fingerprints are never compiler dispatch keys.

## C17 mapping specification

Use the existing certified F64/Bool AST, ordered materialization, branch-local
right-hand prelude and ordinary source function declarations. The public
predicate is declared in the generated header; reciprocal has internal linkage
and is absent from the public header. The package owns exactly its generated
header, implementation and API manifest (plus the bundle index when bundled).
No runtime files, copied helpers, math-library entries or fmod/trunc imports
are permitted. GCC14 and Zig compile separate producer and consumer objects
with strict warnings, no fast math and disabled contraction, at O0 and O2.

## Java21 mapping specification

Use the existing primitive Double/Boolean expression AST and branch-local
materialization. The public predicate is a public static method in its generated
source owner; reciprocal is private static. No boxing, FloatBits helper,
Runtime class, Math calls or runtime-package dependency is required. The package
contains only Generated.java and its API manifest (plus the bundle index).
Compile the producer and external consumer separately with Java21 strict lint.

## Required evidence

Compare native Rust and both targets against the unmodified vendored JavaScript
implementation and an independent raw-bit equality oracle. Include all official
numeric examples, every exponent with boundary mantissas, both signs, signed
zeros, subnormals, infinities, NaNs and deterministic bit patterns.

Measure reciprocal-call traces in test-only instrumented copies of the actual
Rust and generated sources. Value-preserving eager and duplicate-call mutations
must fail the trace oracle; zero-sign/guard/comparison mutations must fail the
value oracle. Prove external private access fails, and intentionally exposing
the helper makes the same consumer succeed. Exact artifact, import, declaration,
documentation and visibility inventories must be mutation-sensitive.

Export actual tested packages and source outside Docker, retaining no generated
files in Git. Existing eight-language corpus tests remain enabled; this additive
C/Java proof does not finish a broad feature family or permit legacy retirement.
