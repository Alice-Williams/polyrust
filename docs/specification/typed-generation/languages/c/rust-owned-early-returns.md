# Early owned return and lexical continuation

- Status: complete
- Plan: [M35-02B-03E](../../../../plan/tasks/M35-02B-03E-early-owned-returns.md)
- Foundation: [guarded owned exits](rust-owned-guarded-exits.md)

## Closed source shape

Keep the guarded contract's safe nongeneric function signature: exactly one
immutable bool parameter plus immutable i32 parameters and an i32 result.
The root block contains only authenticated Box<i32> constructors and whole-owner
moves, followed by one `if flag { return *first; }` without an else arm. The
continuation is immediately `*second`, `return *second`, or `return *second;`.
The early arm contains only its explicit return; both semicolon forms are valid.
All return values dereference current chain ends. Existing resource limits and
distinct constructor-parameter anchors remain in force.

Reject intervening or suffix statements, extra conditions, nested ownership
operations, explicit else arms, non-parameter conditions, indirect return values,
partial moves, custom Drop, unwinding and unsupported calls. Do not generalize
this grammar by silently dropping unreachable code or flattening lexical scopes.

## Canonical exits and scope distinction

Retain the actual root block, If statement/expression, Boolean condition,
early-arm block, explicit return/value, and continuation expression. Use enums
to distinguish early-arm versus continuation paths and tail versus explicit
return exits. The false path's read scope is the real root scope; no synthetic
else block or invented source HirId may replace it.

Canonical containment certification must independently identify each selected
source route. The normal guard proof ties the actual MIR SwitchInt discriminator
to the Boolean parameter and selects the true/false successor by value. The
existing finite-path relation then authenticates owner producers, moves, reads,
ordered cleanup and Return for both outcomes, including shared suffixes.

The false path can retain the If statement's unit result. Only this grammar
additionally permits unprojected unit assignments whose operand is an evaluated
`Const::Val(ConstValue::ZeroSized, ())`, after the complete MIR visitor proves
that the local has no reads or other uses beyond those exact writes and storage
markers. Unevaluated constants, other types, nonconstant producers and observed
unit values reject. This is not permission to ignore arbitrary dead-looking
operations. The compiler's [constant forms](https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/mir/consts/enum.Const.html)
and [zero-sized value representation](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.ConstValue.html)
describe the API; pinned compilation and corruption tests establish this use.

For both Boolean and unit bookkeeping, validate every definition of each
candidate local throughout the complete body, including the other exit's cleanup
blocks. Every such definition must be an unprojected constant of the admitted
kind; the whole-body visitor then excludes all readers and other uses. Charge
only the selected path's actual writes to that path's instruction inventory.
An unexecuted sibling definition is never treated as a selected-path operation.

Preserve the difference between a complete function certificate and an individual
path certificate. Safe construction receives only the compiler context/owner;
caller-authored claims and altered MIR remain private corruption-test inputs.
The existing guarded-if/else entry must not silently accept this new grammar.

## Verification boundary

Fixtures cover each continuation kind, both early-return spellings, moves,
parameter renaming/reordering, separate read/drop scopes and both guard outcomes.
Negative controls substitute guards, exits, scopes, selected owners and cleanup.
Require exact private-construction/non-erasure failures, source E0382/E0502,
nonempty inventories and all prior compiler/C/Java/native/lint gates.

This supplies structured source/compiler exit correspondence, not generated
allocator or destructor behavior. Conditional moves, partial records and call
transfers still require their own proofs before M35-02B can close. Typed C heap
mapping and native cleanup equivalence remain M35-02C/D; Java mapping is separate.
