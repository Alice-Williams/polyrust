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

The 03C flow domain may retain normalized finite unions of the checked integer
intervals. This represents exclusions such as a signed divisor not equal to
zero without pretending its interval hull excludes zero. Union/intersection
preserve the actual scalar identity; transfer considers every retained operand
interval combination. Empty domains represent unreachable numeric states, not
an arbitrary default value. No caller supplies these sets or exclusions.
Floating restrictions retain NaN possibility and both zero signs where needed.

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

Ordinary branch joins retain the union of possible numeric values. Growing
loop-head inputs widen conservatively to the actual type domain when necessary;
stable cells need not lose their facts. Solve-time imprecision or an unproved
operation produces an unknown result, never an optimistic success or an
unreachable edge. A separate strict pass checks operations against the converged
inputs. Predicate inversion through integer promotions/conversions is permitted
only when the preimage is proved sound; modulo narrowing cannot be inverted by
copying target bounds onto the original operand.

Storage keys retain closed Local/Parameter/Global roots and actual member or
exact-index selectors. Root-wide invalidation is a permissible conservative
fallback for overlapping writes. Opaque indirect writes and calls invalidate
globals and address-exposed roots; nonexposure comes from an all-syntax address
inventory. Predicates tied to a materialized known-call result also retain
their dependencies. Exposure is derived from actual AddressOf expression nodes,
not a walker's non-reading traversal of a member/array base. Ordinary field or
element reads do not expose automatic storage; explicit addresses of those
subobjects expose the containing root, even in unreachable syntax. Globals
remain exposed independently of explicit address-taking. Predicates retain
their actual operand dependencies and are killed when those dependencies change.
Arithmetic provenance survives materialization: an already-wrapped product is
not rehabilitated merely by reading it through a Size local.

Unproved provenance is a closed set of reasons referencing actual arithmetic,
aggregate expressions, reads, writes or calls. A widened type domain with an
unresolved origin is not proved-clean size arithmetic. Aggregate assignment and
expression initialization project/rebase the actual source subobject snapshot;
complex or indirect aggregate sources retain an unresolved source obligation.
Root fallback origins cover numeric fields not yet materialized in the state.
Unknown-index/indirect writes and opaque effects poison those fallbacks as well
as existing cells. Scope exits remove expired roots; joins union unresolved
origins, including an absent cell on one predecessor. Exact subsequent scalar
assignment can establish a new clean value without laundering sibling fields.
Absent global cells intrinsically retain authenticated global-object origins,
including when constructed for a missing join predecessor or guard refinement.
A later read is not responsible for discovering this absence. A global is clean
after a branch join only when every incoming path established a clean value.
Unproved global reads and generated-call returns retain explicit origins until
04/05 establishes the relevant storage/body contract. Numeric library transforms
such as trunc/fmod inherit argument history; classifiers and Bool normalization
produce new classifier values and do not inherit extent arithmetic history.

Size algebra has a closed additional relation grammar: for value-preserving
Size/U64 terms, a dominating a<=SIZE_MAX-b proves nonwrapping addition, and
a<=SIZE_MAX/b with b proved positive proves nonwrapping multiplication.
Strict comparisons also entail the corresponding non-strict guard; reversed
operand order and false-edge integer complements are normalized structurally.
Terms are authenticated storage reads, exact constants or representation-
identical unsigned conversions, not arbitrary source text. Relation keys retain
their actual guard witnesses and storage dependencies. Writes, scope exit and
opaque effects kill affected relations; joins retain only relations established
on every incoming path, retaining the witnesses from each path.
An algebraic relation may refine the primitive result/loss proof, but does not
erase earlier operand wrap history or discharge pointer/allocator obligations.
The implicit element-size product in WriteBytes uses the same relation proof
over that exact known call's actual arguments.

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

The path proof's verifier-derived step phase is retained per actual program
point. BeforeStep permits the strict counter<bound relation; AfterStep permits
only counter<=bound; a join of phases retains only their common facts. Branch
updates, nested-loop cycles and Continue edges cannot transplant a strict
pre-update relation onto a post-update value.

## Composition and remaining obligations

03C composes ranges/progress with ContextFacts over the same borrowed package.
Retain private actual-operation nonwrapping and index/extent obligations for
04/05. Numeric range alone does not authenticate a pointer, allocation extent,
nonnull state, initialization, union member, allocator or lifetime. Generated
call summaries and mutually dependent storage/range conditions are rechecked
during complete safety composition. Compiler/resource admission and portable
source-order certificates remain their existing later-stage gates.

Every retained Calculation, Index and Call obligation is tied to an actual
function/graph point and rechecked for pointer-identical occurrence inside that
point's action. Re-deriving a previously materialized predicate is proof work,
not another runtime execution of its old expression at the new point; it cannot
create transplanted runtime obligations. The retained call contains its actual
argument trees and the implicit WriteBytes product checked at that call point.
