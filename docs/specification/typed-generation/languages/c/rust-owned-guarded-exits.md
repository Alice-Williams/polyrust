# Guard-authenticated owned return paths

- Status: implemented compiler-only proof; target heap output remains disabled
- Plan: [M35-02B-03D](../../../../plan/tasks/M35-02B-03D-guarded-owned-exits.md)
- Foundation: [explicit return exits](rust-owned-return-exits.md)

## Initial grammar

A safe ordinary nongeneric Rust function takes exactly one immutable bool
parameter and one or more immutable i32 parameters, in any order, and returns
i32. Its root block contains authenticated Box<i32> constructions and whole
immutable owner moves, followed by a tail `if flag { return *left; } else {
return *right; }`. Both explicit return spellings are allowed. The condition
resolves directly to the Boolean parameter, without negation, calls or aliases.
Each constructor uses a distinct scalar parameter; each returned binding is a
live chain end. The branch bodies contain no declarations or other operations.

Keep existing ownership budgets (128 total parameters and owned bindings,
512 MIR blocks, 1024 locals) and add exactly one source condition/two paths.
Reject extra conditions, loops, partial moves, branch-local owner operations,
implicit-return branches, custom destructors, unwinding and async cleanup.

## Canonical source and typed guard

Retain actual HIR references for the root block, If expression, condition,
then/else blocks and return/value expressions. Use a typed enum for true/false
outcomes. Parameter ordinals connect canonical HIR parameters to MIR arguments;
names, source spans and debug variables are not authority.

Trace the actual Boolean SwitchInt operand through unique typed copy/move
producers to the condition's parameter. Constants, negation and another
parameter cannot stand in for that guard. Authenticate the actual 0/1 successor
mapping, not basic-block order. Reject unsupported extra switch values/edges.

## Complete path evidence

Inventory both finite normal execution paths with their common prefix and any
shared suffix. The union must account for all blocks and every admitted edge.
Each path separately authenticates parameter-derived constructor identities,
whole-owner moves, selected dereference, lexical cleanup order and Return.
Shared blocks denote one operation per execution, not duplicated dynamic events.
Per-path evidence has its own type and cannot be converted into the existing
single-path whole-body certificate; an individual branch must not masquerade
as a complete function after losing its guard.

Use path-specific reaching definitions for return-value pointer temporaries;
assignments in the other branch cannot satisfy this path's read. Common owner
chains remain fixed before the branch. Boolean guard instructions are consumed
by guard evidence, not treated as unread residual bookkeeping. Any remaining
constant bookkeeping retains the existing complete no-reader visitor check.

The safe entry constructs private evidence only from successful compiler
analysis and its queried canonical bodies. Unsupported shapes produce a
diagnostic, not a partially checked certificate. HIR remains the source of
structured target branches; these compiler paths never authorize goto output.

## Proof boundary

Test both outcomes and deliberate guard/edge/owner/drop substitutions, including
shared cleanup suffixes. Include exact privacy failures and nonempty fixture
inventories, and retain all existing single-path source contracts and gates.
This establishes source/compiler correspondence only. C allocation policy,
generated cleanup and native sanitizer/failure-injection proof remain M35-02C/D.

## Implementation and tests

`GuardedOwnedBody::read` is the separate safe compiler-query entry. It retains
canonical condition/branch references and the Boolean parameter's HIR/MIR pair;
each private GuardedPath retains its own chains, source scopes, read, ordered
drop obligations and final Return. A complete two-path inventory authenticates
one shared switch location and covers the entire bounded graph.

The existing producer tracer handles an explicitly typed Boolean parameter set
for guard proof; i32 constructor tracing remains restricted to scalar producers.
Guard assignments are consumed before the shared ownership relation accounts
for the rest of each path. No public API accepts caller-provided traces or MIR.

`owned_guard_test` requires sixteen named fixtures, both outcomes, twenty-two
corruption failures, an extra guard-copy positive control, actual shared-cleanup
evidence, invalid-source E0382/E0502 and empty/changed-source negative controls.
Three privacy targets require E0451; `owned_guard_not_whole_test` requires E0308
when a guarded path is assigned to the existing whole-body type. The separate
format target and strict compiler-adapter Clippy remain enabled. The single
no-semicolon fixture uses rustfmt::skip, and canonical HIR assertions require
that exact source form so formatting cannot remove the coverage.
