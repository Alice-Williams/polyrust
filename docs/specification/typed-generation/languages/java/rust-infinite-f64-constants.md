# Rust signed-infinity constants in Java21

- Status: planned
- Contract: [shared](../../rust-infinite-f64-constants.md)

## Representation and catalogue

Use typed JavaValueRef::KnownField references for Double.POSITIVE_INFINITY
and Double.NEGATIVE_INFINITY, with closed JavaKnownField/JavaMemberName entries.
Both have primitive Double type, not boxed Double. Their owner is the existing
java.lang.Double known type; qualified names/imports come from typed symbol
resolution. [Java21 Double](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Double.html)
defines both fields as static final double constants.
No custom Runtime, raw expression fragments, bit-conversion calls, boxing or
runtime initialization is needed.

## Certification and constant authority

Extend constant-value descriptions through a closed enum distinguishing
existing literal values from Infinity(Binary64Sign). Accept only the exact
typed standard fields for the new branch. Preserve public/static/final,
original registration, declared type, source identity and owner certificates.
Reject arbitrary field references, wrong annotations, mutable/nonpublic
fields, lookalike owners and wrong signs. The renderer remains structural;
the known-field dependency and byte bounds must be included in certification.

## Required evidence

Strict Java21 separate producer/client compilation must pass with all warnings
treated as errors. Observe exact raw bits in normal and interpreted runs,
including original fields, aliases and generated imported readers. Faults for
sign loss, finite clamping and zero replacement must compile and be detected.
Recompile all dependents after producer mutations because javac may inline
constant fields. Prove resource bounds, original authority and unchanged old
output bytes before compiler source admission.
