# C17 numeric ranges and counted progress

- Status: normative for M34A-11-02D-03
- Prerequisite: immutable sequencing/control facts from 02D-02

## Proof boundaries

This analysis proves the admitted subset's evaluated numeric operations from
actual AST children and dominating conditions. An imprecise range may reject a
program; it cannot justify an optimistic result. Unknown input ranges are the
actual target type domains. Input-authored ranges, a Size type, a progress
registration or a catalogue name are not evidence.

Implementation uses three bounded checkpoints: private numeric domains and
operator transfer (03A), actual counted-loop structural/path evidence (03B),
then convergent numeric flow and composition (03C). The parent remains open
until all three and the combined mutation matrix pass. Private intermediate
kernels are not public authoring inputs or a renderer entry point.
03A connects exact constant-tree operators/conversions to the transfer kernel.
The independent exact-value arithmetic implementation remains its singleton
oracle; this integration does not add runtime range or dominance claims.

## Typed numeric domains

Integer intervals retain the exact scalar identity and inclusive i128 bounds
within its measured representation. This host carrier contains every admitted
integer value, including U64_MAX; intermediate unsigned products use checked
wide unsigned arithmetic when required. Bool is 0..1. Closed representation,
promotion and usual-conversion rules come from the existing authoritative model,
not a second table.

A floating domain distinguishes exact numeric observations, bounded ordered
values and unknown values. NaN possibility is explicit; infinity and signed zero
cannot be silently converted to ordinary finite singleton facts. In particular,
a range numerically equal to zero may include both signs and cannot be folded
as a known positive zero when deriving a reciprocal. Arithmetic NaN payload
identity is not a promised observation of this domain.

Private constructors reject invalid bounds. Joins contain every incoming value.
Transfer results overapproximate every defined result for every value admitted
by their input domains. Exact singleton evaluation reuses the checked constant
model. Finite exhaustive small-domain and boundary-sampled controls independently
compare results against that model; tests must exercise useful bounded results,
not merely assert that an unknown domain contains everything.

## Operations and conversions

Apply actual promotions/usual conversions before an operation. Signed overflow,
zero divisors, minimum/-1 for both division and remainder, invalid shift counts,
negative signed left operands and out-of-range signed shifts must be ruled out
before evaluation. Narrowing signed and float-to-integer conversions require
representability; floating conversion checks finite truncated values and exact
exclusive upper power-of-two boundaries.

Unsigned modulo arithmetic remains valid target arithmetic, but the analysis
distinguishes it from an independently derived nonwrapping size calculation.
A potentially wrapped multiplication cannot serve as allocation-extent proof.
Transfer classification concerns one primitive operation after its C input
conversions; it is not a recursive certificate of its operands' computation.
Later size evidence must retain actual operand/conversion provenance as well.
For floating results, absence of integer modulo loss does not imply finiteness.
A later bound check cannot repair an earlier overflowing operation. Bitwise and
shift transfer must respect the actual promoted width, not the literal width.

Binary64 uses the frozen nontrapping environment. Non-singleton arithmetic may
conservatively lose precision; subsequent actual guards may recover bounded
conversion facts. Ordered comparison truth excludes NaN; its false edge does
not generally do so. isnan, signbit and ferror have zero/nonzero Int results,
not assumed 0/1 values. Generated calls default to the result type domain until
an authenticated body summary establishes more. Known-call numerical relations
may be used only through their authoritative closed contracts, while storage
and call-precondition obligations remain outstanding.

## Actual flow and invalidation

Numeric state uses authenticated storage identities, never variable spellings.
Read/write facts are keyed by actual program points and declarations/places.
Closed graph edge meanings supply predicate polarity, case/default selection,
loop identity and exited scopes. Conditions refine before the selected branch;
short-circuit and conditional expression children receive their corresponding
guard-local states. Every syntactic child was already reconstructed even when
a numeric path is unreachable.

Writes invalidate dependent ranges and relations before installing a newly
derived result. Escapes and opaque call/indirect-write effects invalidate
possibly aliased storage and globals. Never-address-taken automatic local or
parameter storage cannot be changed by an ordinary defined call; this fact
must be derived from actual addresses, not caller purity metadata. Counter
alias prohibition and the immutable bound permit unrelated calls inside loops.

Use conservative joins and a monotone worklist with widening where intervals
would otherwise grow one value per iteration. Widening loses precision rather
than declaring success. Check operation safety against converged incoming
facts, not a convenient earlier iteration. No semantic iteration cap, timeout
success or arbitrary portable arity limit is introduced.

## Counted-loop evidence

The [exact counted-while grammar](counted-loops.md) remains authoritative.
Only the counter initializer is restricted to literal Size(0). The const Size
bound is an actual immutable snapshot and may have a runtime initializer.

Authenticate the condition tree, actual declarations and all counter writes.
Derive exact-step occurrences from ordinary assignment nodes. Reject other
writes, address aliases, intervening mutation and nested-counter substitution.
Each reachable continuing path, including Continue, executes exactly one step;
early Break, Return or cleanup exits need none. Multiple mutually exclusive
branch-local steps remain legal. Loop identities, lexical owners and entry/
backedge facts come from the immutable graph, not a supplied step list.

At a step, counter < bound <= SIZE_MAX proves counter+1 representable. The
updated relation is counter <= bound. Preserve the possible zero-iteration
exit and handle zero, one and multiple iterations without unrolling enormous
bounds. Direct SIZE_MAX-1/SIZE_MAX proof tests do not execute SIZE_MAX loops.

## Composition and remaining obligations

03C composes ranges/progress with ContextFacts over the same borrowed package.
Retain private actual-operation nonwrapping and index/extent obligations for
04/05. Numeric range alone does not authenticate a pointer, allocation extent,
nonnull state, initialization, union member, allocator or lifetime. Generated
call summaries and mutually dependent storage/range conditions are rechecked
during complete safety composition. Compiler/resource admission and portable
source-order certificates remain their existing later-stage gates.
