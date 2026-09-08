# C17 native call-stack admission

- Status: normative migration target; measurements required in M34A-11-04
- Complements [runtime traversal](runtime-traversal.md), which bounds data-depth
  independently of native call depth

## Supported invocation environment

The initial Linux x86_64 execution profile requires an 8 MiB or larger native
thread stack and at least 4 MiB of unused stack headroom at each generated public
entry. Custom allocator callbacks must not re-enter any generated API and must
use no more than 512 KiB of additional stack, including their callees. Valid
callback code/context lifetimes and normal return remain required. Arbitrary
foreign recursion, asynchronous signal-handler stack consumption and entering
with an already exhausted stack are not safety guarantees of generated code.

The selected admission policy reserves at most 1 MiB for the complete generated
native call path, including linked runtime helpers and the bounded overhead of
catalogued library calls. The remaining headroom is a safety margin, not another
budget available to generated automatic objects. These are chosen policy limits,
not measurements already established by the existing scalar ABI probes.

## Linked accounting

The post-link resource checker derives the actual native call graph from direct
references and every possible exact closed interface-table target. It accounts
for simultaneously live automatic locals/arrays, by-value argument/return
storage, expression temporaries, alignment, calling-convention overhead and
conservative compiler spill/instrumentation allowances. A flat per-function
statement count is not a stack proof. Sum conservative frame bounds along the
worst reachable native path; repeated loop iterations do not multiply live
stack, but nested calls do. Apply checked arithmetic throughout.

Checked v0 rejects user recursion but permits arbitrarily long finite call DAGs;
such a DAG can exceed this target budget. Reject it with a source-attributed
target resource diagnostic before constructing RenderReadyPackage. Do not add a
portable function-arity or call-depth type restriction. Large individual frames
are rejected through the same accounting, even in a one-function package.

Logical lifecycle specialization graphs may have cycles. Their iterative work
engine must nevertheless produce a bounded native call graph; a task edge is
not permission to recurse through a vtable or public traversal driver. Reject
an unbounded native call component rather than assuming the logical fixed-point
summary bounds its stack. Known library functions require catalogue-owned
conservative bounds under the supported runtime profile. Unknown external calls
cannot be assigned an invented zero cost.

## Required evidence before admission

Stage 04 must establish conservative frame/call rules under both pinned
compilers, O0/O2 and the supported sanitizer configuration before exposing the
C resource adapter. Native oracle frame reports and controlled-stack/watermark
experiments must cover every admitted automatic-storage and call category,
including worst-case interface dispatch and catalogued runtime calls. If a
conservative rule cannot be justified, do not claim this profile certified;
revise its implementation or explicitly narrow the target profile and document
the supporting evidence. Compiler invocations remain test oracles, not a
production generation dependency.

Tests include long acyclic portable call chains, large automatic arrays and
many locals, exact computed budget/one-over rejection, missing-edge and
underestimated-frame mutations, wrong interface-target accounting, and illegal
callback re-entry plans. Boundary-admitted programs must execute under the
stated headroom profile without stack exhaustion under every supported build
configuration. Deep finite owned values separately exercise the iterative
traversal engine; a small native stack must not become a value-depth cap.

Record measured bounds, compiler flags, instrumentation assumptions and exact
test invocation evidence in Stage 04. Raising a budget or changing compiler,
sanitizer or runtime versions requires rerunning the corresponding probes.
