# C17 safety-analysis composition

- Status: normative for M34A-11-02D
- Prerequisite: the complete contextual checks in M34A-11-02C

## Layers and evidence boundaries

The target verifier proves the admitted C subset, not every valid C program.
Unknown target AST remains fallible. Private evidence can be produced only by
checking the actual immutable registry and AST; typing restricts who may carry
and consume it, not permission to trust input-authored claims.

| Layer | Inputs | Derived result | Owner |
| --- | --- | --- | --- |
| Known-call foundation | closed catalogue identities | exact types and outstanding operand obligations | dialect/catalogue |
| Constants/layout | actual types and constant children | checked values, size/alignment and assertion truth | ownership/constants and ownership/layout |
| Context/evaluation | reconstructed files and lexical structure | immutable actual control edges and call-root legality | ast/contextual and ownership/sequencing |
| Numeric flow | actual conditions/writes/edges | dominating ranges, relations and counted progress | ownership/ranges and ownership/loops |
| Storage flow | allocation/borrow/write/exit actions | extent, allocator, initialized-prefix, active-member and lifetime facts | ownership/storage |
| Callable composition | actual bodies, call targets and known obligations | authenticated precondition/success/failure summaries | ownership/calls |
| Combined verification | all preceding checks over the same package | private immutable safety state for shared verification/linking | ownership entry point |

Names identify cohesive module responsibilities, not mandatory directory depth.
Production files follow the existing size policy. No wildcard parent imports,
numbered file fragments, second mutable AST or public proof-state constructor.

The closed library catalogue is needed before call-effect verification. Its
identity/signature/obligation foundation therefore belongs to 02D-00. Stage 03
retains actual includes, dependency closure, names, shared bindings and linked
revalidation; it consumes the same catalogue rather than reauthoring signatures.

## Control and fixed points

The [constant/layout contract](constant-and-layout-proof.md) specifies exact
numeric conversions, evaluated versus all-syntax traversal and private facts.

Reuse the existing actual-AST graph. Extend it with closed edge meanings,
including predicate polarity, switch selection/default and loop/backedge/exit
identity. Do not infer truth from a successor's position in a Vec. Scope exits
crossed by jumps are derived from lexical owners; skipping a ScopeExit node
does not preserve a borrow or excuse destruction.

Facts are typed and keyed by authenticated declarations/storage, not names.
Writes, escapes and call effects invalidate dependent facts. Intersections keep
only facts valid on all reachable predecessors; loop analysis converges
conservatively and may reject unproved code, but cannot assume success to break
a cycle. Constant unreachable evaluation may prune runtime effects only after
all-syntax structural checking. Case/tag facts and active union members are
separate from the 02C containing-object initialization fact.

The fixed counted-loop grammar has a dedicated proof over actual counter/bound
declarations, condition, writes and every continuing edge. Range safety cannot
be inferred merely from a progress registration. Bounds and nonnull/provenance
facts must dominate pointer formation, not only the eventual memory read.

## Storage and interprocedural composition

Uninitialized storage, initialized prefixes and complete live owners are distinct.
Zero initializes empty lifecycle slots, not valid inhabited values. Allocation
restore preserves actual storage state. Copy, clone, move, drop and borrow are
different transitions with exact allocator/extent/lifetime obligations.

Known-call metadata lists obligations; only checked operands/context discharge
them. Generated summaries are derived from bodies and preserve normal/failure
outputs, aliases, cleanup and invocation allocator identity. Same-prototype
substitution cannot transfer a summary. Legal lifecycle specialization graphs
are preregistered and checked without topological body-order assumptions, while
their execution remains the specified iterative work engine. Circular summary
claims are not induction, and recursive user calls remain excluded by Core.

This stage implements the frozen ownership, callable, counted-loop and traversal
contracts. It cannot weaken those contracts to make a sample pass. Stage 04 still
owns compiler/resource admission and rendering; stage 06 must prove actual
generated lifecycle bodies under fault injection and sanitizers; stage 07 proves
portable mapping identities/evaluation order. Passing this analysis alone cannot
render legacy source or advertise a missing Supports mapping.

## Required integration proof

Each substage has its own task and positive/rejected mutation matrix. Final 02D
review additionally exercises combinations that defeat isolated checks: a valid
type with stale range, right prototype with wrong allocator/contract, initialized
union with wrong active member, complete storage with an expired borrow, and
correct local cleanup with a bypassing exit. Compile-fail tests protect private
evidence boundaries; native later-stage oracles remain mandatory, not substitutes
for the structural proof or vice versa.
