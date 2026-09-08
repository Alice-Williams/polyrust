# M34A-11-02C — C contextual scope and control-flow verification

- Status: in-progress
- Depends on: M34A-11-02B

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

## Definition of done

- Verify namespace/linkage/file-role legality, complete object requirements, prototype/definition agreement, exact member/call types and return coverage.
- Independently rederive file-initializer and static-assertion constant-expression
  categories from their actual trees. Forward complete-object obligations for
  SizeOf/AlignOf and constant arithmetic/assertion truth to the 02D proof stage.
- Verify initialization on all reachable paths, duplicate switch constants after promotion, exhaustive non-fallthrough arms, bounded loop targets, labels and forward cleanup exits.
- Keep verifier evidence private and derived from actual registrations and AST. Unknown target input is fallible; target verification cannot certify legacy source.

## Implementation sequence and module contracts

Planning may proceed while the immutable 02B repair is reviewed; implementation
starts only after that review closes. Follow Java's separation of structural
type, lexical and completion checks, with C-specific explicit control edges.
This stage is diagnostic/contextual checking, not an alternative certificate
pipeline. Stage 02D must still discharge ownership/range/call-effect obligations;
03 binds the frozen registry into the shared dialect and rechecks linked facts.

| Order / module family | Responsibility | Must not infer |
| --- | --- | --- |
| 1: contextual type rechecking | Authenticate actual references and reconstruct every value/place/call/initializer relationship from children; compare the derived result with cached fields | Cached type or registry brand alone is not a contextual proof |
| 2: complete objects and inventories | Walk registered complete-by-value dependencies; compare actual file/declaration/definition/member/parameter/scope/control occurrences against the authoritative registry | Pointer recursion is not a by-value cycle; deleting a projection cannot delete a registration obligation |
| 3: lexical structure | Track exact function, root/child scope, visible declarations, control stack and forward cleanup labels | Matching names do not establish identity; a branch's terminated path is not a continuing path |
| 4: control-flow graph | Derive private typed program points and actual edges for branches, loop tests, break/continue, return, scope exit and cleanup jumps | Progress metadata does not create loop initialization, updates or edges |
| 5: definite initialization | Worklist analysis with intersection over reachable predecessors; retain local/member/constant-array coverage and precise writes | A member write does not initialize its siblings; taking an address or calling an unproved contract does not initialize storage |
| 6: contextual constants and integration | Normalize switch cases to the actual promoted scalar; recheck initializer/assertion categories and package-local linkage | Constant-expression shape is not arithmetic safety or assertion truth; final allocated names/order belong to 03 |

Use cohesive modules under ast/contextual with an explicit root entry point,
not a large verifier file or numbered fragments. Reuse the authoritative scalar,
signature and constructor relations while independently walking actual children.
Do not create a second signature inventory, caller-populated proof state or
public verified/render-ready constructor. Diagnostics may be public; any
intermediate graph/analysis evidence remains private and immutable to callers.

The lexical walk validates structure even in unreachable code. A local becomes
visible at its declaration before checking its initializer, but a self-read
still needs initialization. Every block and loop/switch/label identity has one
actual occurrence. Break uses the innermost loop or switch; Continue ignores
switches and uses the innermost loop. A cleanup destination must be later in
structural order and in the same or an ancestor scope. It cannot require a
declaration skipped by the jump. Destruction/ownership state is rechecked in 02D.

The graph contains only edges derived from the AST. Return has no fallthrough
edge. A loop retains its zero-iteration exit. Switch arms have explicit exits;
a reachable implicit arm fallthrough is rejected. A reachable nonvoid function
fallthrough is rejected. Private typed program-point indices are traversal
implementation details, never input-supplied evidence or declaration identities.

Initialization distinguishes reading a value, writing a place and forming an
address with a closed access enum. Address formation checks all pointer/index
operands without pretending to read or initialize the addressed aggregate.
Declaration entry clears state from any earlier loop iteration. Joins consider
only reachable incoming edges; an exited branch cannot poison a later read.
Partial aggregate writes retain their exact registered member paths. Array
coverage must not enumerate a potentially enormous declared bound: use actual
initialized paths/whole-initializer coverage and checked cardinalities.
Heap/prefix, active-union, callable-effect and borrow/lifecycle facts remain
02D obligations; initializing a pointer slot does not prove its pointee is Live.
The shared final verifier must compose these obligations rather than treating
02C success as complete source validity.

Before name allocation, check structural origin/role/linkage and type ownership,
not candidate-spelling uniqueness. Stage 03 allocates ordinary/tag/member/label
names by identity and checks actual linked collisions and private-layout leaks.
Likewise, 02C independently checks constant categories; 02D evaluates actual
constant operations, assertion truth and safe arithmetic.

## Tests and proof

- Positive/rejected mutation for every contextual rule, including crossed scopes and aliases, missing/mutually deleted registration evidence and duplicate promoted switch values.
- Control-flow tests for branch initialization joins, unreachable exits, wrong break/continue targets and labels bypassing declarations.
- Exact counted-loop containing/body scopes and counter/bound declaration
  dominance, including Continue across nested switches; follow c/counted-loops.md.
- Every Switch carries its exact registration. Reject duplicate statement
  occurrences, wrong function/scope, crossed switch IDs and Break naming an
  outer loop/switch. Continue targets the innermost loop across nested switches.
- Block/root scope identities match actual lexical parent/function ownership
  with exactly one occurrence. Reject swapped siblings, duplicated/absent scopes,
  wrong parents and local declarations in another scope; accept dominating
  ancestor reads and legal distinct shadow bindings.
- Full Rust/lint/compile-fail and cached tracked/release/eight-target gates.

## Contextual implementation checkpoint (review pending)

The C-specific implementation follows the planned small-module split:
local child-based reconstruction; authoritative typed registration views;
complete-by-value object graphs; lexical visibility and cleanup checks;
private actual-AST control-flow points; definite-initialization storage paths;
and promoted switch-case equality. The largest contextual production module
is 263 physical lines. Test-only corruption fixtures remain excluded from the
Bazel rust_library source glob. No new dependency or lint suppression was added.

Diagnostic entry points compose local, package, lexical and flow checks and
return no certificate. The shared CDialect/verify/link/certify integration,
ownership/range/call-effect proof, resolved names/includes, resource certification
and native new-renderer proof remain the explicitly later stages. C compliance
remains open/Fail; the legacy generator is unchanged by this checkpoint.

The private graph retains each executable control/transfer's actual statement
origin, containing scope and derived successors for 02D. Initialization uses
reachable predecessor intersections, kills local state at declaration/scope
exit, distinguishes Address from Read/Write, and retains exact member and
constant-index paths. Variable-index member reads cannot hide local-array
initialization obligations. Whole-array facts avoid enumerating a huge bound;
the huge-bound unit control does not claim target object-size/resource validity.

Focused container Bazel C unit/rustdoc controls passed through
c6e409ec-21b7-4e0a-807a-f671c5d0525b. One earlier focused run,
e6331462-3735-4d08-b3a3-a051e0370868, rejected a test fixture's reserved
default identifier before reaching its intended control assertion; the fixture
names were corrected and 6f98cb05-e6d8-4ee7-a45a-01111bb0ca92 passed.
The initial full gate 0cc8916f-9626-4f16-89e5-0faded8e3843 stopped on
three Clippy findings (one large member-path enum and two collapsible matches).
Those were repaired by boxing the member payload and using match guards.
Skipped tests in that failed run are not passing evidence.

The complete local checkpoint is green:

| Gate | Invocation | Result |
| --- | --- | --- |
| Every tracked Bazel rule, including Rust Clippy/rustfmt, Buildifier and policies | 77686192-40a7-4290-a619-880180ef3551 | 439 rules; all 314 test targets pass, 48 executed |
| Cached release | 4d053dc1-fddf-4943-8028-ebc6e6ef6c61 | All 251 test targets pass |
| Eight-target conformance and deterministic manifests | d0ef7c30-f272-4627-9574-fd2f851fa84b | 50 cases and one portable test; evaluator and all eight targets agree; repeated manifests byte-identical |

Commands use polyrust-dev-step0 at /workspace and
`bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch`.
The tracked gate enumerates every rule in tracked BUILD.bazel packages and
runs test --test_output=errors --noshow_progress; release uses test
//:release_gate; conformance uses run //crates/conformance:polyrust-conformance
-- --all-targets --determinism. Normal action/test caches remain enabled.
The user's untracked stdlib-abs example is excluded and untouched.
The C unit binary reports 137 passing tests. Existing native outputs are
regression evidence, not native proof of the future migrated C renderer.

A fresh Sol Extra High reviewer is auditing the complete contextual delta,
without a numerical finding cap. Findings must be evaluated and repaired or
explicitly justified before this task closes and 02D implementation begins.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02C; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
