# C17 sequencing and immutable control facts

- Status: normative for M34A-11-02D-02
- Prerequisite: contextual reconstruction and exact constant/layout checking

## Call positions

All calls require sequencing, including known library calls. A call may be the
entire value of a direct automatic local Expression initializer, an assignment
to a direct Local place, Return or Discard, or the call owned by Evaluate.
Its arguments and indirect callable expression must contain no calls.

A conversion, unary/binary operator or conditional wrapping a call is not a
root call. Materialize its result into a local first. Calls in member/index/
dereference assignment RHS values are also rejected even if the destination
currently appears call-free. Array, struct and union initializers recursively
require call-free children. Conditions, static initializers/assertions, all
other expressions and every place operand are call-free. A label does not
change its child statement's rule.

Function checks inspect every actual graph node, not a reachable-only traversal.
Discard and predicate Read have distinct action variants; no origin-name or
successor-position test decides call-root legality. The builder exhaustively
maps every statement, including labelled children and unreachable nodes.
This is an exhaustive all-syntax check, including unselected branches and
statements following Return. No purity flag, reachability shortcut or known
operation exemption can admit an extra call. The verifier diagnoses but does
not rewrite; lowering owns ordered temporaries and branch-local prefixes.
Target sequencing alone is not proof of portable source evaluation order.

## Actual graph and typed edges

Reuse the contextual graph, splitting its immutable model and private builder
into cohesive modules. Nodes retain borrowed original statements/actions and
exact lexical scopes. A graph belongs to the actual function body. Program
points are opaque private indices, not caller-authored evidence.

Each edge has a closed destination: an actual program point or FunctionReturn.
Its closed meaning distinguishes ordinary flow, If/loop predicate polarity,
switch case-list/default selection, ordinary loop backedge, Break, Continue,
Return and CleanupJump. Branch edges borrow the actual predicate; switch edges
borrow the actual discriminant, switch identity and case list. Loop edges
retain the exact loop identity. Meaning never depends on successor order.
Return cannot be represented as fallthrough to FunctionEnd.

Derive exited scopes innermost-first from source/destination ancestry. A normal
ScopeExit action includes its own scope, including root-to-FunctionEnd flow.
A jump skipping that action still crosses the same lexical scopes. Return exits
all scopes including the function root. Entering a child block exits none;
ordinary flow between sibling blocks exits only the source branch through the
common ancestor. Cleanup jumps themselves remain forward ancestor/same-scope
only; this stage does not admit sibling-scope gotos.
Continue inside a switch retains the innermost loop target, crossing the switch
arm and intervening scopes. Break retains the actual innermost loop/switch.

The initialization analysis consumes Point destinations and applies the
edge's scope exits once. ScopeExit nodes remain structural landmarks, not a
second independent destruction event. Later storage analysis consumes the same
edge inventory; it cannot assume skipped nodes executed.

## Immutable context boundary

A private ContextFacts value borrows the same registry and source files that
passed reconstruction, lexical/initialization and constant/layout checking.
It owns derived per-function immutable graphs and exposes only read access.
Its constructor runs those prerequisites and the all-syntax sequencing pass.
There is no mutable graph accessor, public point/fact constructor, caller
projection, or rendering certificate. Diagnostic public entry points may
report success/failure but cannot return proof-bearing mutable state.

Range, ownership and callable analyses remain later obligations. This slice
neither assumes their findings nor advertises C capability implementations.

## Proof

- Positive root calls and rejected nested/wrapped/sibling/argument calls across
  direct, indirect and known identities; call-free indirect operands remain
  legal while a hidden callable-operand call fails.
- Root-versus-nested local/aggregate initializers, local-versus-other assignment
  destinations, labels, all conditions and nested place operands.
- Read typed branch edges independent of vector order; verify actual predicate,
  case-list, loop and cleanup identities and target node actions.
- Nested switch/loop Continue, ordinary backedges, Break, early Return and
  ordinary sibling-flow and cleanup ancestor exits with exact ordered crossed-scope inventories.
- External compile-fail controls for graph/context constructors and mutation;
  existing reconstruction, initialization and return regressions remain green.
- Full cached tracked/release/lint and eight-target determinism gates, followed
  by independent uncapped review before checkpoint closure.
