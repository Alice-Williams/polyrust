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

### Review repair obligations

The uncapped review of 82a77fb found five production issues and an incomplete
required mutation matrix. All findings are accepted: allocation storage and
actual read/write/index/member operations require complete object types;
origin/file roles need structural definition-owner checks; initialization must
respect conditional evaluation; exact array paths must include every admitted
integer literal category and registered enumerator value. Scope/control/alias
and child-reconstruction mutations must be broadened before closing this task.

Completeness traverses all syntax, including unreachable branches. Ordinary
incomplete pointers and address cancellation remain legal; array indexing needs
complete elements even when only forming an address. SizeOf/AlignOf target
completeness remains an explicit 02D obligation. Separate closed visitor modes
keep lexical/type/completeness walks exhaustive while initialization prunes only
proven-unselected short-circuit/conditional expression children. Unknown values
retain both possibilities. Exact constant leaves do not evaluate unchecked
arithmetic or assume numeric narrowing preserves nonzero; those are 02D facts.

The origin matrix is specified in c/symbols-and-files.md. It checks registration
ownership and actual definition/body placement, not files merely referencing a
symbol. It does not authenticate caller-presented Core provenance or replace
the later linked dependency and mapping-certificate obligations.

The first new completeness controls reproduced acceptance of invalid inputs in
f4b01237-07e5-474f-b299-59840ee359c3 (expected-red, not a passing gate).
Focused container contextual tests passed after the first repairs in
b7c8f545-f4ae-423a-a21c-39ad1d5f4c79 and after the origin/scope/conditional
matrix additions in 82bf77f3-8e8d-4553-ae62-6e2409132484. Broader controls,
full gates and a fresh independent repair review remain required.

The first broad repair gate, 6edeabd5-30b5-4ac9-bbe4-664774f639f7, stopped
on three Clippy cloned-reference-to-slice findings in new initializer/declaration
tests. Replace those clones with borrowed one-element slices, without lint
suppression. Its skipped tests and unrun release/conformance stages are not
passing evidence; rerun the complete gate after repair.

The repair proof matrix is split into test-only modules rather than expanding
the verifier files. contextual_completeness covers allocation storage and
incomplete dereference/index uses with complete/address-cancellation controls;
contextual_conditional covers selected/skipped/unknown expression paths and
ensures skipped evaluation cannot bypass lexical/type checking;
contextual_arrays covers every integer literal category and enumerator index
against an unwritten sibling. contextual_origins exercises every origin row
and file role, private-header definition-family laundering and production
references from tests with body-file propagation.

contextual_scope_mutations checks sibling swaps, duplicate/missing/wrong-parent
scopes, local placement, roots, foreign functions and parameter count/order/
owner substitutions. contextual_control_mutations checks duplicate/deleted/
crossed/reowned loops and switches plus nested-loop break/continue targets.
contextual_aliases distinguishes mutual alias-hidden by-value cycles and
incomplete storage from legal recursive pointers. contextual_transfer inspects
an actual loop/Continue graph and seeds a previous-iteration initialization fact
to prove declaration transfer kills it unless the declaration initializes anew;
this narrow graph control is not a complete program/progress certificate.

contextual_value_variants traverses every value/place kind under a valid parent,
all operators, known constants and numeric conversions; it corrupts descendant
value caches and place caches independently. contextual_callable_variants adds
both adapter conversions and direct/indirect void-call argument/brand mutations
under labels. contextual_initializer_variants mutates all five initializer
categories and their stored types. contextual_declaration_variants checks all
six declaration categories, enum projection deletion and file-object initializer/
extern-linkage checking. These local reconstruction fixtures deliberately do not
claim flow/ownership/ABI safety for their null or incomplete registration inputs.

### Repair checkpoint evidence (fresh review in progress)

All five reported production issues have dedicated repairs and controls. The
expanded C unit binary reports 159 passing tests; the largest contextual
production module remains 263 physical lines. No dependency or lint suppression
was added, and all new mutation fixtures are excluded from library source globs.

| Gate | Invocation | Result |
| --- | --- | --- |
| Every tracked Bazel rule, including Rust/Bazel linters and policies | a5f14d82-a462-40cb-b9cf-9cc666e200f1 | 439 rules; all 314 test targets pass, 48 executed |
| Cached release | f695fa66-2234-46d8-a30a-38243372261f | All 251 test targets pass |
| Eight-target conformance and deterministic manifests | 5135fa3a-0135-46a8-814a-7498f58fc7dc | 50 cases and one portable test; evaluator and all eight targets agree; repeated manifests byte-identical |

The commands/container/cache policy are unchanged from the implementation
checkpoint below. The user's untracked stdlib-abs example remains untouched.
These existing native outputs are regression evidence, not proof of a migrated
C renderer. The exact earlier implementation SHA 82a77fb also has successful
hosted CI run 34272699543; that is not hosted evidence for this new repair.

A fresh Sol Extra High reviewer is auditing the full contextual implementation
and expanded matrix without a finding cap. An initial enum-placement concern
was withdrawn after checking the composed entry point: local reconstruction
requires enumeration's file to equal its canonical owner file, then verifies
the declaration belongs to its containing source file. Either tamper form is
rejected before the origin-role pass; unlike aggregate definitions, enum
definitions cannot be relocated. No production workaround is needed. The review
remains in progress and this task cannot close until its final findings are
evaluated and any accepted repairs pass the required gates.

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

## Second contextual repair review

The uncapped review of 66ef4554 accepted three further defects: joins lost the
common initialized union object when branches wrote different members; moved
definitions could hide missing declarations in their registered owner files;
and expression-only pointer-to-array types escaped element completeness checks.
All three have dedicated repairs before closing 02C. The first two reproduced in
b7bbda02-0402-4d50-93fc-cf4655cd674f and passed the full C unit target after repair
in b32f691d-0dc8-40e1-a45f-5daa430ccbda. The last reproduced in
d377ea29-c68f-49e4-bf26-dd926b536681. These expected-red runs are not passing gates.
An earlier test-fixture build, dc088869-807b-469f-91a9-2811fdb5f76b, had two
incorrect constructor calls; those were corrected before the red reproduction.

contextual_union_joins exercises an unknown input branch, union/struct/array
shapes and missing-branch/unwritten-sibling controls. The join derives finite
ancestor candidates and retains only coverage independently established by both
predecessors. contextual_owner_files tests functions, objects, complete structs,
complete unions and incomplete tags, with missing/present owner declarations.
Definitions remain independently required where the registry promises them.
contextual_completeness now tests expression-only pointer-to-array types with
plain-pointer, complete-element and unreachable-syntax controls.

The incomplete function-pointer parameter finding is rejected and withdrawn:
C permits incomplete aggregate types in mere prototypes, while actual function
definitions require complete parameters. See WG14's
[C draft, 6.7.6.3](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1547.pdf).
The composed checker already requires complete actual definition parameters;
indirect calls cannot substitute an unproved prototype for a registered callable
contract. A positive AST regression and the existing two-compiler native ABI
probe now retain an incomplete by-value parameter/return prototype explicitly.
No production restriction was added for this withdrawn finding.

Hosted CI 34280162332 completed successfully for exact
66ef4554f959f93d70ae1298efb3266dd5a25602. It is not evidence for the new repairs.
The repaired C unit binary has 163 passing tests. Its native incomplete-prototype
control passes the existing ABI probes under both pinned compilers at O0/O2.
Fresh review of the repaired source is still required; 02C remains open.

| Gate | Invocation | Result |
| --- | --- | --- |
| Every tracked Bazel rule, including Rust/Bazel linters and native ABI probes | 8402f2a3-a95b-418e-b43b-880236918589 | 439 rules; all 314 test targets pass, 49 executed |
| Cached release | a5c9aba1-1ab3-4a23-b3bc-77eb89084257 | All 251 test targets pass |
| Eight-target conformance and deterministic manifests | 046b75a5-22ae-41e0-ab5e-1014fd22af61 | 50 cases and one portable test; all eight targets agree; repeated manifests byte-identical |

## Final production review and proof-matrix closure

A fresh uncapped Sol Extra High review of cf807c92342c034ea49f8ca384c1ef49c752025a
found no production acceptance/rejection defect, certificate bypass or raw-source
path. It identified six required proof groups, accepted as test gaps rather than
reasons to change working production checks:

1. Same-typed nonconstant file initializer replacement after construction.
2. Explicit forwarding of SizeOf/AlignOf operand completeness, constant arithmetic
   safety and assertion truth from diagnostic 02C to mandatory 02D.
3. Global/function-definition completeness and actual Object/Aggregate origins.
4. Reads of address-forming pointer/index operands and unproved call effects.
5. Promoted duplicates across switch arms and fallthrough in the default arm.
6. Nonvoid branch/loop return paths and exact cleanup-label occurrence.

contextual_declaration_variants now replaces scalar and nested array initializer
leaves with same-typed global reads and requires ExpectedStaticInitializer.
contextual_safety_boundary explicitly demonstrates that incomplete SizeOf/AlignOf,
a false assertion and an unsafe but category-valid constant operation are not
rejected by 02C; those test values must never receive a final certificate.
contextual_definition_completeness checks global definitions and actual function
parameter/return types with complete/incomplete and alias controls.
contextual_definition_origins checks Object/Struct/Union and member ownership in
both generated/runtime directions, and same-registry typedef relocation.
contextual_address_operands covers pointer/index operand initialization and
address-passing to a call without a proved initialization effect.
contextual_control_edges covers cross-arm promoted duplicates, default fallthrough
and deleted/duplicated cleanup labels. contextual_return_paths checks an unknown
Boolean input's two branches and a loop's zero-iteration function exit, each with
a matching positive return control.

Do not add artificial production restrictions for unreachable isolated cases.
Typedef/Enum relocation is rejected by canonical owner and source-file
reconstruction before the origin pass. An incomplete aggregate assignment is
already rejected by CStatements::assign's mutability construction; its negative
control asserts IncompleteAggregate there and its complete control passes the
full contextual entry point. Likewise an incomplete Member owner cannot carry
the required actual complete member inventory. Existing earlier rejection is
valid evidence; a redundant later guard need not become a new public escape.

The first expanded unit run 99661c97-d790-4fda-a0aa-b02cf2acf781 failed because
the write fixture incorrectly unwrapped that existing constructor rejection;
this was a test assumption error, not a newly discovered production hole.
After correcting the expected boundary and completing the matrix, all 173 C unit
tests passed in 43c888b5-5bcd-45a6-ad25-52db8c2e95d7. The complete gates below
also pass. A fresh final review of the proof additions remains required before
closing 02C. Production verifier logic is unchanged from reviewed cf807c9;
new fixtures stay test-only and every contextual production file is at most
263 physical lines.

| Gate | Invocation | Result |
| --- | --- | --- |
| Every tracked Bazel rule, Rust/Bazel linters and policies | 7c76f622-e723-448e-8cb4-b5aaffb784c6 | 439 rules; all 314 test targets pass, 5 executed |
| Cached release | a3dc4b77-0302-40d1-ab3c-d83ef5692abf | All 251 test targets pass |
| Eight-target conformance and deterministic manifests | 9b4d50e5-10a5-44a3-9d59-ffdc400844c7 | 50 cases and one portable test; all eight targets agree; repeated manifests byte-identical |

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02C; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
