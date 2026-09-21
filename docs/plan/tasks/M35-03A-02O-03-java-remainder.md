# M35-03A-02O-03 — Java binary64 remainder foundation

- Status: complete
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02O-02

## Contract

Admit the structural JavaBinaryOperator::Remainder with primitive Double
operands/result and Multiplicative precedence. Preserve recursive dependency
authority and byte/resource bounds. Do not use Math.IEEEremainder or a runtime.

## Definition of done and tests

Separate producer/importer/client compilation under Java21 strict lint agrees
with the independent remainder oracle. Compiling value/trace faults are detected.
Private-reader and public-certification tests reject wrong types, precedence,
arity, recursively unadmitted operands and unauthenticated imports. Existing
integer remainder rejection remains intact. Full gate and fresh review pass.

## Implementation and focused evidence

The dependency-body reader and source-byte reservation now admit only the
structural primitive Double remainder at Multiplicative precedence. Existing
typed rendering handles parentheses and needs no new template or runtime.
The exhaustive private-reader matrix covers operand/result types and precedence;
public tests retain integer remainder rejection, nested shape checks, import
registration/arity and composition in either arithmetic operand position.

Tree f962cccabd10b63829ceae3b418c3064f79f3de1 passed all 389 Java backend
tests plus rustfmt, buildifier and source-policy gates in the Linux container.
Three certified owners are compiled separately from their consumer under
Java21 --release 21 -Xlint:all -Werror -implicit:none and an empty sourcepath.
Each of eight runs checks 22,832 exact bit/NaN-category results from 11,416 pairs.
Three compiling value faults (nearest-quotient family, swapped operands and
zero-sign loss) and three value-preserving producer-call faults (dropped,
duplicated and reversed) are checked against independent value/trace oracles.

Rust-source admission is not part of this target checkpoint; it follows in 02O-04.

The same isolated tree subsequently passed all 882 release/lint targets,
invocation b3ad2ac2-e9ba-4c50-bf93-f5b9ec4d9a41 (62 executed, 820 cached).
The checkout used LF archive export and --lockfile_mode=off; pinned metadata
and ordinary test caching remained unchanged. The focused invocation was
a97db790-0744-4d5b-8695-0ccb29bc52e6.

A fresh independent Sol Extra High review of this exact source tree found no
concrete correctness or proof defect. It audited recursive type/authority
admission, byte/resource traversal, native compilation and oracle sensitivity;
no extra feature was treated as a blocker for this bounded target checkpoint.
