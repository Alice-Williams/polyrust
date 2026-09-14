# Explicit owned return exits

- Status: implemented compiler-only proof; target heap output remains disabled
- Plan: [M35-02B-03C](../../../../plan/tasks/M35-02B-03C-explicit-owned-returns.md)
- Foundation: [multiple owned chains](rust-multiple-owned-chains.md)

## Closed source shape

Keep the distinct immutable i32 parameter anchors, Box<i32> construction and
whole-move chains of the multiple-owner contract. The root and successive tail
blocks contain only those lets, followed by one final `return *live_owner`.
Admit both an explicit return tail expression and a final semicolon statement
with no block tail. A nested block itself must still be in tail position.

No statements may follow the return, even if rustc classifies them unreachable.
No return in an initializer, conditional return, sibling block or non-dereference
return value is admitted. Existing resource limits and ambiguity exclusions
remain unchanged. Tail-only APIs retain their previous source grammar.

## Typed source exit and compiler relation

Represent exit kind with an enum, not a Boolean or a string. Preserve canonical
HIR references for the return expression and its dereferenced value, plus the
lexical block containing them. Independently certify those references against
the actual body walk; matching types, names, spans or expression counts do not
identify a return.

The established MIR relation must authenticate the value's live owner, every
remaining owner's exact cleanup obligation and order, and the single final
Return reached after those drops. Retain that Return location in private exit
evidence. The safe construction API accepts only the compiler context and owner
identity, querying the analyzed body itself. Arbitrary MIR and claimed source
exit metadata are available only to private corruption tests.

This adds source-exit information for later structured lowering. It does not
render MIR edges as gotos, authorize target heap allocation, prove target runtime
equivalence or admit branching early exits. Branch/partial-move and call-boundary
proofs remain separate prerequisites for closing structured ownership work.

## Required controls

Test both source spellings in root/nested blocks, distinct cleanup/read scopes,
multiple owners, moves and shadowing. Check actual canonical return/value
identities and MIR Return order, not just output markers. Mutate the selected
owner, exit scope, return location and cleanup obligations; each mismatch must
reject. Include complete local-renaming invariance through existing regressions.

Dedicated runtime, format and exact privacy-failure Bazel targets cover this
increment. All existing tail-only proof targets remain enabled and unchanged in
their acceptance contracts. Unsupported valid Rust must produce no proof;
E0382/E0502 source errors must occur before successful analysis output.

## Implemented evidence boundary

`ReturningOwnedBody::read` is the separate compiler-query entry. It uses the
existing multiple-chain relation, then retains private `ReturnEvidence` alongside
the resulting body. `exits::Mode` and `exits::Exit` distinguish tail/return syntax;
canonical scope certification walks the body independently and verifies the
actual return/value references. No public constructor accepts arbitrary claims.

`owned_return_test` requires nine positives, five valid exclusions and fourteen
private corruptions, plus invalid-source and empty-inventory controls.
`owned_return_private_body_test` and `owned_return_private_exit_test` require
exact E0451 diagnostics; `owned_return_format_test` covers all relevant files.
Three intentionally unformatted fixture functions retain no-semicolon returns
using `rustfmt::skip`; assertions require those exact HIR tail forms so formatter
normalization cannot silently erase that coverage. This skips no test or linter.
