# M35-02B-03C — Preserve explicit owned return exits

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-03B
- Specification: [explicit return exits](../../specification/typed-generation/languages/c/rust-owned-return-exits.md)

## Contract

Extend the multiple-chain relation to a final explicit `return *owner`, with
or without a semicolon, at the end of a root or tail-nested block. Retain the
canonical return expression as typed exit evidence instead of erasing it into
an implicit tail expression. Authenticate the scalar read, all remaining
owner drops and the final MIR Return using the existing complete-path relation.

This is a prerequisite for branch-dependent early returns, not their admission.
Statements after a return, sibling blocks, conditional exits and return values
other than a direct live-owner dereference remain unsupported.

## Definition of done and tests

- A separate explicit-return entry requires that exit shape. Existing tail-only
  APIs continue to reject explicit returns.
- Private exit evidence retains the actual canonical HIR return/value, lexical
  read scope and the authenticated MIR return location. No user-supplied
  expression or place can fabricate an exit certificate.
- Root and nested explicit returns, semicolon/no-semicolon forms, interleaved
  moves, unused owners and inner versus outer returned owners pass.
- Wrong return owner, scope, missing/extra cleanup and return-location
  substitutions reject; retained return/value identities equal canonical HIR.
- Conditional returns, unreachable suffix statements and scalar-return
  substitutions reject as valid-but-unsupported Rust; invalid ownership is
  rejected by rustc before successful proof output.
- Exact privacy compile failures and a nonempty fixture inventory are enforced.
  Keep all prior ownership, C/Java, native, Rust/Bazel lint and format gates.
- Complete a fresh independent review, isolated exact-tree full gate,
  documented evidence, milestone commit and push. No target heap output yet.

## Current evidence

- Focused Linux/Bazel gate `f0b22970-ce80-4f25-8e75-21db5132a521` passed
  4/4 tests in 18.127 seconds, including strict compiler-adapter Clippy,
  formatting and exact E0451 privacy failures for the wrapper and exit evidence.
- Nine supported and five unsupported valid-Rust functions exercise both return
  spellings, nested/moved/shadowed owners and separate read/drop scopes.
  Fourteen private source-exit, scope and MIR cleanup corruptions reject;
  valid source substitutions, empty inventory and E0382/E0502 controls fail
  before a successful proof marker. Canonical pointers, not just IDs, are checked.
- Historical root/tail/multiple ownership runtime targets passed during the
  initial integration gate. The new driver also consumes construction capability
  bindings and checks a tail-only fixture through all three historical APIs.
- Isolated tree `c53f7b0e3abe411f14038068ae097226d228a87a` passed full gate
  `afe62614-6799-492b-942c-dea447428d93`: 398/398 tests across 518 targets,
  49.869 seconds, eleven tests executed and all valid cached results retained.
  Historical C/Java/native, compiler frontend, Rust/Bazel lint and format gates
  remained enabled. Exact archive blob bytes and executable modes were verified.
- Fresh independent Sol Extra High review found no confirmed correctness,
  contract, test, privacy or Bazel issues. It checked both source spellings,
  canonical return/value pointers, ordered cleanup, private evidence, non-vacuous
  corruption controls, historical API compatibility and capability consumption.
- Conditional exits are still required by the parent milestone but are not part
  of this closed increment; the review correctly treated them as out of scope.
  All twenty preserved ownership files still match their baseline. The final
  documentation-inclusive tree and gate are recorded in the checkpoint commit.
