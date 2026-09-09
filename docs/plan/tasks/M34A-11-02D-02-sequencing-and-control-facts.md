# M34A-11-02D-02 — C sequencing and typed control-flow facts

- Status: complete
- Depends on: M34A-11-02D-01

## Goal

Make actual evaluation and control edges available to the safety analyses without
caller-supplied dominance or positional guesses about graph successors.

## Definition of done

The exact contract is specified in
[sequencing and control facts](../../specification/typed-generation/languages/c/sequencing-and-control-facts.md).
02D-01 closes at 9acf7baaa6de3f7e35bccf861464f6ad2c5a694b after its clean
independent review and full cached gates. Implement this slice in order:

1. Split the graph into immutable model, private structural builder and scope
   crossing derivation; replace positional successors with closed typed edges.
2. Adapt initialization to point destinations and edge-derived scope exits,
   retaining existing local-declaration and missing-return checks.
3. Enforce all-syntax call-root legality using the shared expression/place
   traversal and exhaustive actual graph-action/initializer dispatch.
4. Compose a private borrowed ContextFacts boundary over context, constants,
   sequencing and actual per-function graphs; expose diagnostics, not facts.
5. Add root/hidden-call and edge/identity/scope matrices, privacy compile-fail
   tests, all cached gates and independent uncapped review before closure.


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

## Implementation and evidence

The graph now separates its immutable model, structural builder and scope-edge
derivation. Closed meanings retain actual predicate polarity, switch case lists,
loop/backedge, Break/Continue/Return and cleanup identities. Program-point and
graph fields are private; read accessors expose no mutation. Return has a
separate destination and cannot become ordinary FunctionEnd fallthrough.

Initialization applies scope exits on each outgoing edge, including Continue
and cleanup jumps that skip ScopeExit nodes. A root ScopeExit still exits its
root scope even though FunctionEnd is a root-owned structural landmark.
Sequencing consumes every actual graph action, including unreachable nodes and
labelled children, with Discard distinct from predicate Read. Its expression/
place visitor never skips unselected syntax. ContextFacts borrows the same
checked registry/files and retains immutable per-function graphs; the public
diagnostic returns no evidence or rendering certificate.

Seven focused test modules cover root positions, nested/wrapped/sibling/
argument/callable-operand calls, all nonlocal assignment categories, aggregate
initializers, hidden pointer/index/member operands, conditions, labels and
unreachable code. Edge tests reorder successor storage without changing
semantic interpretation and inspect exact crossed-scope/target identities.
The prior Continue initialization regression now checks edge-based scope
invalidation directly. Context and false-assertion controls prove the composed
diagnostic does not bypass prerequisites. The C unit suite contains 219 tests.

Passing cached Linux Dev Container evidence on the reviewed worktree:

- Focused C units, Rustdoc compile-fail, Clippy, Buildifier and documentation:
  `39b8abae-27fa-45cc-a5c1-56088ac0b5fa` (all five targets pass).
- Full tracked gate: `61f04b36-5020-476e-85a5-41820c243ccf`
  (445 rules; all 320 test targets pass, 45 executed).
- Release gate: `e3e4e799-923d-4918-b380-033d9bed7f6b`
  (all 257 test targets pass).
- Eight-target conformance: `6e253852-44e1-4c57-b71d-0fa06a0f2fdf`
  (50 cases and one portable test agree with the evaluator; repeated manifests
  are byte-identical).

Earlier invocation `04841677-9546-48b7-86a6-2e8105e2ce32` had one test-only
fixture failure: it attempted to register the reserved keyword default as a
scope identifier. The fixture now uses fallback; no production keyword rule
was weakened. That invocation is not claimed as an overall pass.

Independent uncapped Sol Extra High review by c17_ast_construction_review
completed cleanly: no substantiated production defect or required proof gap.
This was a new independent review scope using an existing reviewer context,
not a newly spawned agent. The reviewer inspected all scoped production/test
files, immutable boundaries, edge semantics, scope invalidation, all-syntax
sequencing, existing regression matrices and Bazel inclusion without editing
or rerunning tools. No proposed finding required rejection or a repair loop.
Redundant branch-isolation cross-products were not promoted to requirements.

Base checkpoint 9acf7ba has successful hosted CI run 34294051318. That result
is not attributed to this checkpoint's later commit. Local gates and clean
independent review close this slice; range/ownership/call-summary and renderer/
mapping obligations are not attributed to these diagnostic checks.
