# M35-02B-03 — Extend structured ownership correspondence

- Status: in-progress
- Parent: [M35-02B](M35-02B-structured-owned-mapping.md)
- Depends on: M35-02B-02

## Contract

Extend the proven closed relation in explicit increments to nested scopes,
multiple owners, conditional/partial moves, owned records, early returns and
function boundaries required by M35-02. Give each increment a typed source-shape
contract before admission. Compiler drop flags and place projections are evidence;
they are not a request for goto-based generation or a custom source borrow checker.

## Definition of done and tests

The first increment is [M35-02B-03A — Tail-nested lexical scopes](M35-02B-03A-tail-scopes.md).
It deliberately retains one owner chain and one normal path. The remaining
multi-owner, branch/partial-move, early-return and call-boundary increments stay
unimplemented until their own closed contracts and proof tasks are written.

The next increment is [M35-02B-03B — Multiple owned producer chains](M35-02B-03B-multiple-owned-chains.md),
using distinct scalar parameter identities as construction anchors.

[M35-02B-03C — Explicit owned returns](M35-02B-03C-explicit-owned-returns.md)
then retains canonical return expressions and their authenticated cleanup/return
locations. Conditional early exits remain a later, separate increment.

[M35-02B-03D — Guard-authenticated owned exits](M35-02B-03D-guarded-owned-exits.md)
introduces one Boolean condition and two complete explicit-return paths before
conditional moves or compiler drop flags are admitted.

[M35-02B-03E — Early return and continuation](M35-02B-03E-early-owned-returns.md)
then preserves an early arm and a false-path continuation in the actual root
scope, without inventing an else block.

- Map every admitted structured exit and owned operation unambiguously to the
  compiler's corresponding places/drop obligations; unsupported shapes diagnose.
- Distinct same-type owners, branch guards, field projections and lexical
  cleanup order cannot be substituted for one another through matching counts.
- Tests cover both branches, shadowing/sibling scopes, remaining fields after
  partial moves, early-return cleanup and admitted call-boundary transfers.
- Typed mismatches, missing/duplicate edges, wrong fields/guards and moved-owner
  substitutions fail the appropriate correspondence oracle.
- Explicitly document deferred shapes and runtime policies before M35-02C uses
  the evidence. Java heap support and final native cleanup proof remain separate.
- Each completed increment passes fresh review and all historical/isolated
  Linux/Bazel gates before its own milestone commit and push.
