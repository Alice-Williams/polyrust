# M34A-11-02D-02 — C sequencing and typed control-flow facts

- Status: planned
- Depends on: M34A-11-02D-01

## Goal

Make actual evaluation and control edges available to the safety analyses without
caller-supplied dominance or positional guesses about graph successors.

## Definition of done

- Enforce the exact conservative full-expression call policy in the grammar.
  Permit a call root only in the specified direct-local initializer/assignment,
  return/discard/effect positions; all arguments and callable operands are call-free.
- Reject calls hidden in conditions, nested operands, aggregate initializers,
  member/index/dereference places and conditional branches. Lowering, not the
  renderer/verifier, must materialize ordered conditional prefixes.
- Extend the existing actual-AST graph with closed edge meaning: ordinary flow,
  true/false predicates, switch selection/default, backedges and explicit exits.
  Preserve statement, function, scope and exact target identity.
- Derive scopes crossed by break/continue/return/cleanup edges; do not pretend
  a jump executes a skipped ScopeExit node. Keep lexical validation exhaustive.
- Provide a private immutable context-facts boundary to ownership/range modules.
  Do not expose mutable graphs, public program-point constructors, certificate
  constructors or a second caller-authored AST projection.

## Tests and proof

- Every admitted root versus nested/sibling/argument/condition/initializer call,
  direct and indirect/known calls, plus calls hidden behind labels and places.
- True/false edge reversal, wrong case/loop identity, nested switch/loop Continue,
  early exits and sibling/ancestor scope-exit inventories.
- Context-fact mutation/access compile-fail controls and existing initialization
  regression matrix, full cached tracked/release/lint/eight-target gates.

## Commit gate

Record evidence and commit/push M34A-11-02D-02. Target sequencing does not by
itself prove portable source evaluation order; mapping certificates own that.
