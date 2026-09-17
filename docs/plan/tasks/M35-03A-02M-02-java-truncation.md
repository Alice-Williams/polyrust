# M35-03A-02M-02 — Certified Java rounding-call foundation

- Status: complete
- Parent: [truncation](M35-03A-02M-floating-truncation.md)
- Depends on: [C target proof](M35-03A-02M-01-c-truncation.md)

## Contract

Admit only typed JavaKnownCallable::MathFloor and MathCeil with exact
(double)->double signatures, no receiver and Primary precedence. Retain normal
symbol resolution, call argument traversal and original dependency authority.
Extend the source reservation to account for the actual resolved standard
owner and catalogue-member names; do not turn off source bounds or allow arbitrary known calls.

## Definition of done and tests

Certify producer/imported-owner packages and compile with Java21 strict lint.
Check floor, ceil and conditional truncation against independent integer-bit
oracles, including signed zeros, subnormals, fractions, infinities and NaNs.
Detect swapped rounding branches, zero-sign loss and dropped/duplicated receiver
calls. Reject malformed signatures, operands, results, precedence, receiver,
unadmitted known callables and forged dependencies. Actual rendered bytes fit
the computed reservation. Full Linux Bazel release/lint gates and independent
review pass; record exact tree and invocation before commit.

## Initial validation correction

The first targeted run passed native values/traces and malformed-call checks,
but two new assertions followed the wrong rendering path. Existing known calls
render a resolved KnownType owner plus the closed catalogue member; the
KnownCallable map entry is not emitted. Source reservation now follows that
actual path and keeps the renderer unchanged. Each-call floor/ceil spelling
length differences test that reservation non-vacuously.

The item-level verifier's independent name check covers opaque imported
dependencies. It is not the full package certification boundary: common
certification rederives and compares exact resolved items against the original
unresolved AST/reference map. Requiring that narrower item hook to reject an
unused known-call map entry was an incorrect test expectation, not evidence
that an altered certified package could be rendered through a safe API.
Original dependency substitution checks remain, and explicit parameter/local
Math/java shadowing rejection tests cover the qualifiers actually emitted.

## Target proof and review

Corrected implementation tree: 9df200ae532097ddf4f0b6c6a4b558270b40ef3b.
All six targeted tests executed and passed in invocation
be95644f-f50d-442b-a4a3-81ebf2c53b6d (61.053 seconds total).

The native test compiles original and importing owners separately with Java21
-Xlint:all -Werror -implicit:none. Independent integer-bit expectations cover
floor, ceil and truncation, both signed zeros, subnormals, integral boundaries,
infinities and NaN category. Seven variants include the baseline plus wrong
floor, wrong ceil, reversed truncation branches, erased zero sign, dropped
receiver calls and duplicated receiver calls. Value assertions and A/B/C call
traces are independent; dropped calls recompute the correct value so the trace
control is not accidentally redundant.

Both independent reviews of that exact corrected tree found no core defect or
required proof gap. Both independently confirmed the documented item-hook
correction from common package re-resolution evidence. No review finding was
waived as a feature request. Full release evidence follows.

## Completion

All 840 Linux Bazel release tests pass on the reviewed implementation tree
9df200ae532097ddf4f0b6c6a4b558270b40ef3b, invocation
30cdbf09-0e2f-4e89-ab02-39ce398b4da5 (636.691 seconds; 98 executed,
742 cached). Java's full unit/native suite passed 376 tests, with none ignored
or filtered. The release gate includes Rust formatting/Clippy, Bazel lint,
source-policy, all generated-language tests and the real-world differential
examples. No tests were disabled.

All 138 hashes across 18 earlier C/Java compiler bundles remain byte-identical;
all 19 preserved ownership-work file hashes are unchanged. Both independent
reviews are clean. This completes only the target foundation; original Rust
source discovery, executable capability registration, library manifest metadata
and end-to-end source examples remain 02M-03.

The local checkpoint is not a claim that unpublished GitHub CI has passed.
Publishing remains subject to the previously reported approval block.
