# Rust HIR direct calls and same-crate helpers

- Status: same-crate mapping implemented; checked foreign-call extension in M35-01D-04C
- Parent: [HIR mappings](rust-hir-mappings.md)
- Proof boundary: [native call stack](call-stack-resources.md)

## Source identity and signature registration

The selected experiment entry remains fn(i32) -> i32 for the native differential
oracle. This harness signature is not a general function-arity restriction.
It must be externally reachable in Rust; selection never promotes a private
Rust function into a public target API.
Reachable ordinary same-crate functions may have zero or more i32/bool
parameters and an i32/bool result. Rust analysis must succeed first.

Collect callees from resolved HIR/type-check facts. Register every admitted
signature before lowering bodies, keyed by compiler declaration identity.
Identical helper names in different modules must remain distinct. Legacy
single-crate entry points reject foreign calls. The checked graph extension
below admits only source-authenticated C dependency witnesses. Reject indirect
calls, methods, generic instantiations and unsupported signature
or adjustment shapes until their dedicated mappings exist. Do not infer a
callable from its printed name or copy its source body into the caller.

DirectCalls owns session-bound source input and an executable registered C
mapping. Its associated context and output retain the existing Reader and C
expression types. Missing, duplicate and wrong input/context/output bindings
must fail Rust compilation, not become runtime support flags.
FunctionSignatures separately maps compiler identity and signature facts to
CFunctionType through a stored binding. EntrySignatures retains the selected
native harness ABI check. All ten capability slots must be supplied exactly
once before the complete binding set can be built.

The discovery pass is finite and diagnostic-bounded: at most 4,096 functions,
100,000 visited expressions across their bodies, and 128 expression nesting
levels. These are verifier budgets, not Rust language or generic AST limits.

## Checked foreign-call extension

The graph check operation `adapter --check-crates RECORDS_OR_RESPONSE_FILE`
uses the [checked source driver](rust-hir-dependency-driver.md) before invoking
C lowering. It certifies all declared owning packages in memory and publishes
no generated output. Bundle publication is a subsequent task.

Resolve calls to an explicit `Local(LocalDefId)` or `Foreign(DefId)` variant.
Only local definitions enter the body traversal queue. Keep foreign witnesses
and consumer-branded CFunctionRefs in inventories separate from owned functions;
the combined inventory remains bounded by 4096 callable identities. Sort foreign
registrations by stable declaration identity, not ephemeral compiler numbering.

Resolve a foreign DefId's CrateNum through the private invocation-scoped
CheckedDependencies view. Its owning result must be the exact CDependencyApi
constructed from that crate's same-analysis checked C package. Compare the
compiler owner identity, look up the exact StableCrateId/DefPathHash declaration,
and compare the FunctionSignatures mapping to the actual witness signature
before CRegistry::import_function. FunctionSignatures now consumes DefId for
both local and foreign ordinary scalar functions, without adding a second
signature implementation or a new independent Supports flag.

Imported functions never acquire local prototypes, parameters or definitions.
Argument sequencing and call construction remain the existing DirectCalls
mapping. Projection, all verification passes, shared linking and complete
transitive resource certification remain mandatory. Alias spelling, descriptive
JSON and a scalar signature alone cannot create a foreign witness.

The same-crate rules below describe the original closed family. Its foreign
extension requires the [certificate-backed effect and resource protocol](rust-hir-crate-dependencies.md);
unknown external functions and unsupported source shapes still diagnose.

## Evaluation and representation

Use CFunctionRef, CCallableKind::Direct and the checked CExpressions call
constructors. The target registry authenticates owner, signature and arity.
Lower arguments in Rust source order and materialize call results in typed,
scope-owned evaluation temporaries. Any required argument temporaries precede
the call. The renderer must receive already-sequenced expressions, not decide
an evaluation order or invent a declaration.

Statement preludes stay at the original evaluation site. In particular, an if
condition's prelude executes before the branch, and a branch's prelude stays in
that branch. Struct fields are evaluated in source order before the existing
member-index arrangement. No temporary may cross a lexical lifetime boundary.

## Closed effect evidence

A declared scalar signature is not an effect summary. Derive private evidence
from the actual registered definitions and their complete call graph. The first
closed family has scalar parameters/results, local-only storage, structured
control and calls only to other members of the same admitted family. Unknown
operations, global accesses, foreign/indirect calls and pointer parameters are
not members merely because the function returns a scalar.

Storage checking may treat a call result as initialized only when its direct
target has this derived evidence and its arguments have been checked. Every
callee body still passes the ordinary context, numeric, initialization and
storage passes. Evidence is immutable and private; callers cannot register a
pure flag or use the summary as a rendering certificate. Conservative numeric
result ranges remain conservative: purity does not prove a returned number's
range or allocation-size history.

## Linkage and graph admission

Each retained function has exactly one primary prototype before use and one
definition. Their registered owner, signature and linkage agree. An admitted
externally reachable API is external; private/restricted helpers are internal
and render with static linkage. The shared callable visibility is derived from
that checked linkage, not independently set to public.

Build the call graph from actual target call nodes. Reject unresolved edges and
cycles, including self-recursion, before certification. Compute the worst live
call path by checked arithmetic over a finite DAG. Repeated sequential calls do
not accumulate frames; nested callers do. Account for each function's automatic
objects, by-value results/arguments, expression temporaries and conservative
compiler/instrumentation overhead. Existing source/AST capacity checks remain.

The no-call resource model must not silently certify calls. Native frame
reports and controlled-stack tests under both pinned compilers, O0/O2 and
ASan/UBSan must justify the extended conservative bounds. Production generation
does not invoke a C compiler to compensate for missing proof.

## Required proof

- Scalar signatures with zero, one and several arguments, including bool.
- Same-spelled helpers in different modules, a diamond and nested calls.
- Exact typed argument/prelude ordering and branch-local placement.
- Compiler-resolved aliases call the original declaration exactly once.
- Unchanged reconstruction succeeds before owner/signature/linkage/edge
  mutation controls are counted as negative proof.
- Missing/duplicate definitions and prototypes, indirect/foreign calls,
  recursive components and unsupported effect bodies reject.
- Exact computed stack-budget boundary, one-over and arithmetic-overflow tests;
  native call-path measurements must not exceed the certified bound.
- Rust/C differential execution and native symbol-table visibility checks.

Public headers and independently compiled consumers remain M35-01D-03. Separate
crate linking remains M35-01D-04; this step does not merge dependency bodies or
claim a complete translation of every public item in the metadata inventory.
