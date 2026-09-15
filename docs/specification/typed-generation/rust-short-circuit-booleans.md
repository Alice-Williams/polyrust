# Rust-source short-circuit Boolean expressions

- Status: implemented and native-tested for the closed no-heap source grammar
- Task: [M35-03A-02B](../../plan/tasks/M35-03A-02B-short-circuit-booleans.md)

## Shared compiler input

ShortCircuitBooleans has a checked session-bound input containing private left
and right HIR operands and a typed And/Or enum. Construction requires built-in
Rust lazy operators, bool operands/result, no overload resolution and no
implicit adjustments. Backend registration binds this capability to executable
lowering with the existing Reader context and typed target value result.

The operand grammar is the already supported scalar expression grammar. Integer
bitwise operations, overloads, expression blocks and new ownership forms remain
separate work. Unsupported operands reject the whole package before publication.

## C lowering

Evaluate the left operand once using the current reader and initialize a fresh
private bool local from its result. For And, test that local; for Or, test its
typed logical negation converted from C int to bool. Create two child scopes.
Only the selected branch contains the right operand's entire evaluation prelude
and an assignment of its result to the private bool local; the other is empty.
Return a typed read of the initialized result local. Nested logic repeats this
construction inside its containing branch. No operand statements cross branches.

Use CRegistry, CStatements and CExpressions for every declaration, scope, read,
assignment and condition. Synthetic evaluation scopes/locals have explicit typed
origins and unique identities. Restore parent scope/prelude even on rejection.
The closed shared profile admits only bool assignments to local places, not
parameters, members or dereferences. Scalar-call derivation, ownership/storage,
resource accounting and the structural renderer must cover the same shape.

## Java lowering

Initialize a fresh non-final bool local from the left value. Use an If node,
with Not for Or, and put the right prelude and result assignment inside its
selected JavaBlock. Both branches are explicit. Restore the parent source scope
and prelude on success or rejection; right temporaries do not escape that block.

Dependency-body admission tracks initialized mutable bool locals by lexical
scope, resetting state between methods. Assignments must target those locals;
parameters, fields, immutable locals and other mutable types remain unadmitted.
Existing AST certification checks types, visibility and definite initialization.
Existing source-size measurement includes both branch blocks and assignments.

## Rendering and proof

Neither renderer discovers control flow, invents temporaries nor requests a
runtime. It serializes certified typed nodes. Exact source-owned file inventories
and public/dependency identities remain unchanged.

Prove truth tables and nesting against Rust and an independent model, then
separately observe the taken/skipped order of operand calls. Structural probes
and injected eager/duplicate/reordered-call controls supplement native traces;
equal final values alone do not prove short-circuit evaluation. Require native
Java lint, GCC/Zig O0/O2, negative registration/source tests and full gates.
