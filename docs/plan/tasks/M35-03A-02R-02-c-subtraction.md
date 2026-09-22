# M35-03A-02R-02 — C wrapping-subtraction safety foundation

- Status: complete
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [oracle](M35-03A-02R-01-subtraction-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-subtraction.md)

## Contract

Admit exact internal U32/U64 subtraction in the certified dependency profile.
Use existing unsigned arithmetic flow and guarded signed reconstruction, without
special trusted formula recognition. Do not widen source/public signature types.

## Definition of done and tests

Typed subtraction/operand/result shape and recursive-child tests pass. Missing,
reversed and wrong-value guards reject unsafe signed conversions; direct signed
overflow still rejects. Modular loss cannot become nonwrapping extent evidence.
Original import authority, depth/resources and dependency-derived headers hold.
Separate strict GCC14/Zig producers/clients at O0/O2 and GCC UBSan agree with the
independent oracle. Compiling wrong-operation, reversed-result and narrowing
faults are detected. Full gate, old-output/WIP preservation and clean fresh review
precede the separate commit/push. No compiler source admission in this checkpoint.

## Implementation and proof boundaries

The production change admits same-width U32/U64 Subtract beside Add in the
existing internal integer profile. Both actual children are scheduled recursively.
Range transfer, guarded conversion proof, signed overflow checks, ownership and
resource analysis are unchanged; no formula receives special trust.

Addition and subtraction share focused test-only body/package fixtures. Existing
addition cases remain independently named tests. Subtraction checks exact types,
ordered input locals, the unsigned difference and signed reconstruction; unsafe
guard/cast/arithmetic variants reject, while safe semantic substitutions certify
as C but fail the operation-specific dataflow assertion.

Native checks render certified producer and forwarding packages, compile their
implementations and clients separately with strict GCC14/Zig at O0/O2 and GCC
UBSan, and compile each header independently. Both owners are checked over 15,790
input pairs. Five compiling safe-fault packages (wrong operand, result, operation,
reversed operands and half-width narrowing) differ from independent truth. The
narrowing mutation uses only defined unsigned arithmetic before the original
guarded signed reconstruction. Maximum unsigned literals are also exercised by
an identity-preserving variant.

Additional tests cover actual child scheduling, mixed signed/unsigned widths,
recursive unsupported multiplication, depth limits, original import authority,
derived standard headers, source-byte/stack bounds, public unsigned rejection,
and underflow remaining MayWrap rather than allocation/extent evidence.

## Release and preservation evidence

Tree 638828b4e2b85effacc8c26c2cbe5c18fa64f3ec passed all 927 isolated Linux
Bazel release/lint targets under d6fc78a6-d3d6-434f-83cf-84385f61b5f9
(136 executed, 791 cached). The C unit partition passed 823 tests, including all
six new subtraction tests; five capacity cases remain in their dedicated gates.
Clippy, rustfmt, buildifier, source policies and capacity/storage gates all pass.
All 372 older generated files and all 38 protected ownership/adjacent WIP files
retain their pre-checkpoint hashes. No test was disabled or cache bypass added.

The first focused command passed five new tests but accidentally filtered out
the sixth profile test by using the wrong fully qualified module path. The full
unit log explicitly confirms that exact profile test also ran and passed.

Independent Sol Extra High review found no core defect in admission, range/UB
proof, typed shape, fixture reuse, native mutations, dependencies or resources.
Its optional suggestion is a further allocation-use integration case for a
wrapping subtraction result. This is not a required safety repair: the production
loss-provenance/allocation rules are unchanged, the new test directly verifies
MayWrap underflow, and the existing allocation/extent suite remains passing.
Java target support and checked source admission remain separate checkpoints.
