# M34A-11-02D-03B — counted loop evidence

- Status: complete
- Depends on: M34A-11-02D-03A

## Goal

Actual-AST counted-loop structural and path evidence; facts name the exact loop/declarations/steps, not caller metadata.

## Definition of done

Implement after the reviewed 03A checkpoint ee2b571, in this order:

1. Derive loop/declaration views from ContextFacts and exact structural shapes.
2. Inventory every counter/bound write and address, including unreachable
   syntax; reject shared counters and steps owned by nested loops.
3. Count updates over actual graph edges with finite zero/one path states,
   rejecting a second update and any continuing edge without one update.
4. Add construction, mutation, branch/exit/nesting, privacy and boundary tests,
   then full cached gates and independent uncapped review.

- Actual counter Size(0) declaration and immutable Size bound snapshot dominate entry; bound need not be a literal.
- Exact Numeric(Bool,Less(Read(counter),Read(bound))) and actual Add Size(1) assignment nodes.
- Inventory all writes/addresses and reject foreign/nested mutation, counter sharing/substitution and writable bound aliases.
- Path-state counts exactly one step on every continuing path; early exits need no step; nested loops and switches retain exact targets.
- Private evidence consumed later for interval refinement, not whole numeric or lifetime certificate.

The initial alias policy is deliberately conservative: reject any counter or
bound address exposure, including a const-bound address. Merely retaining a
const pointer type is not an authenticated transitive call/storage effect
proof. Later 02D-04/05 may admit proved read-only bound aliases; no const
qualification stripping or invented purity flag is accepted here.

Path exploration uses exact checked constant predicates when available and
otherwise keeps every possible branch, including switch alternatives. It
does not assume a runtime bound is constant, unroll iterations or impose an
iteration cap. This slice establishes structural loop progress; 03C owns
numeric flow refinement and update representability.

The detailed numeric contract is in
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
The exact loop form remains [counted loops](../../specification/typed-generation/languages/c/counted-loops.md).
These slices implement the parent task; none weakens its complete-stage gate.

## Tests and proof

- Complete counted-loops.md mutation matrix with zero/one/multiple/SizeMax boundary controls.
- Branch-local one-step positive vs missing/double steps and Continue bypasses.
- Nested-loop/switch exits, declaration reentry/intervening writes, alias/escape and owner substitution.
- Full cached gates and independent review.

## Commit gate

Record focused and full cached Linux Dev Container evidence and an independent
uncapped Sol Extra High review. Commit and push M34A-11-02D-03B separately. Keep
parent 02D-03 open until every slice and its combined checklist pass. No partial
numeric or loop fact is a C safety, allocation or rendering certificate.

## Implementation and evidence

Private loop views borrow the actual statement, progress references, condition,
body and declaration occurrences from ContextFacts. Shape checks enforce literal
Size(0), exact less-than/Bool condition and immutable initialized bound; the
bound can be a runtime expression or direct Size-returning call. The all-syntax
inventory authenticates every actual assignment and forbids shared counters,
foreign/nested writes and counter/bound address exposure.

The path checker visits each (actual graph point, Zero/One step count) pair at
most once per loop. A second update is rejected immediately; every own Continue
or backedge requires One. Early exits need no update, while a cleanup jump
remaining within the loop cannot bypass the step. Nested loops preserve outer
counts and exact switch/loop targets. Only successfully evaluated exact
constant predicates can prune edges; unknown alternatives stay possible.
The three production helper modules contain 74–86 lines each.

Five focused test modules (including the shared fixture) cover the exact shape
and mutation inventory, runtime bounds, boundary bounds through SIZE_MAX,
branch-local updates, missing/double steps, Continue bypasses, ordinary/labelled
blocks, early exits, nested loops/switches, declaration reentry, independent
counters sharing a bound, address exposure in dead syntax/call operands and
indirect writes, immutable bound reconstruction and numeric narrowing in
constant predicates. The C unit suite contains 257 tests. A Rustdoc control
rejects external access to private loop evidence.

Passing cached Linux Dev Container evidence on the reviewed frozen source:

- Focused C units, Rustdoc, Clippy, Buildifier and documentation:
  `061124b9-2a5f-4c01-aaf0-d61ebdb70f6e` (all five targets pass).
- Full tracked gate: `ec29836e-cd42-48d8-b897-12354297e584`
  (445 rules; all 320 test targets pass, 45 executed).
- Release gate: `2367bd83-b563-4d95-a115-6d7e9f76f9a2`
  (all 257 test targets pass).
- Eight-target conformance: `0ba285a9-83a4-4bc2-a558-6f6c24856277`
  (50 cases and one portable test agree with the evaluator; repeated manifests
  are byte-identical).

Earlier invocation `90b90137-7a7f-471b-bca1-c607cfe91713` failed to compile a
test fixture using a nonexistent Free enum variant. The actual closed catalogue
identity is Release. Only the test spelling was corrected; no catalogue or
production acceptance rule was changed. That invocation is not claimed as a
pass. The earlier first focused slice passed at
`fe943420-002b-4304-b0fb-c44606333199`.

Independent uncapped Sol Extra High review by c17_ast_construction_review
completed cleanly against base ee2b571281344a8f61be8e68de3428d0a22b8798:
no substantiated production defect or required proof gap. This used an existing
reviewer context for a new scope, not a newly spawned agent. The reviewer
inspected the entire scoped production/test matrix and the relevant constructor,
context, graph and Bazel dependencies without edits or rerunning tests. No
proposed finding required rejection or repair; redundant branch localization
was not promoted to a requirement. Source remained frozen during review and
the final gates. Numeric flow and update representability remain 03C.
