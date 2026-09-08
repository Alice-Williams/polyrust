# C17 exact counted-while grammar

- Status: normative migration target; owners M34A-11-02B/02C/02D/04

## One initial form

BoundedLoop contains its exact registered identity, a counted-progress record,
an explicit call-free Bool condition and a body block. Counted progress contains
the exact counter and bound local references plus the closed Step::One variant.
This record names evidence to be checked; it is not proof of progress and cannot
instruct the renderer to add missing expressions or statements.

The only initially admitted form counts upward from zero with a Size counter
and an immutable Size bound snapshot. The ordinary enclosing AST declares the
counter initialized by the exact Size zero literal and the const bound with its
actual initializer, both dominating entry. The counter has no intervening
mutation before first entry. Their registered lexical owners are the loop's
containing scope. The loop body has its own registered child scope.

The condition is the actual tree Numeric(Bool,
Less(Read(counter), Read(bound))). The body contains ordinary assignment AST
counter = Add(Read(counter), Size(1)) exactly once on each reachable backedge,
including every Continue targeting this loop. There is no extra update in the
renderer or a loop epilogue skipped by Continue. Early Break, Return or legal
cleanup exits leave the loop and need no update. They retain all cleanup rules.

The counter cannot otherwise be assigned, aliased through its address or
modified by a nested loop/call. The bound is immutable and cannot escape through
a writable alias. The verifier derives these facts from actual declarations,
places, call summaries, control paths and registered owners, not from a supplied
safe/progress flag. Multiple branch-local updates are legal only when every
continuing path executes exactly one and no path executes two.

Step occurrences are identified by walking the actual Assign nodes and their
authenticated counter references, with verifier-owned structural program points
for path counting. The verifier inventories every write to that counter, not
only writes listed by the progress record. There is no caller-supplied step-ID
list that can hide an extra write, omit a branch or manufacture dominance.

At the update, counter < bound <= SIZE_MAX proves counter + 1 cannot overflow.
After the update, counter <= bound and the next condition determines whether
another iteration occurs. Zero bound executes no body; SIZE_MAX is a valid
representable bound, although native tests must not require SIZE_MAX iterations.
No generic semantic iteration cap is introduced.

## Formatting and lowering boundary

The renderer prints only while, the supplied condition and supplied body. It
does not create initialization, comparisons, increments, branches, labels or
cleanup. Assignment expressions/increment operators remain excluded from the
general expression grammar; the explicit body update is an ordinary Assign.

Reverse scans and variable-width algorithms use a separate derived index under
this bounded visit counter. All index updates/branches and range checks exist
in the AST before certification. Iterative traversal engines use checked finite
work/visit bounds and explicit early exits; their node/work accounting is not
replaced by a renderer-chosen loop bound. Additional loop forms require an
explicit closed grammar extension with independent proofs.

## Required proof

- 02B implements all payloads and exact local type/reference checks; identity-
  only 02A registrations are not already complete loop ASTs.
- 02C verifies structural occurrence, containing/body scopes, declaration
  dominance and actual innermost Break/Continue targets across nested switches.
- 02D derives fixed initialization, exact condition, immutable bound, no counter
  escape/foreign mutation, once-per-backedge progress and update range facts.
- Reject changed initialization, wrong comparison/direction/bound/counter,
  zero/two/wrong steps, Continue before update, nested-loop counter substitution,
  counter/bound aliases, missing update paths and incompatible local owners.
- Accept zero/one/multiple iterations, branch-local single updates, early exits
  and nested switches/loops. Step and range boundary arithmetic is tested
  directly at SIZE_MAX-1/SIZE_MAX without an impractical native iteration run.
- 04's two-compiler/optimizer oracle executes finite exact-count and Continue
  fixtures and proves the canonical renderer emits only the supplied tree.
