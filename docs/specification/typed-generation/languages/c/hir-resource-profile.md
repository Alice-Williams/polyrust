# C HIR scalar resource profiles

- Status: implemented closed-profile admission; integrated release proof in progress
- Owners: M35-01B (no-call baseline), M35-01D-02 (direct calls), M35-01D-03B (paired packages)
- Complements: [platform proof](platform-and-proof.md) and
  [call-stack resources](call-stack-resources.md)

These profiles apply only after the closed scalar/record/shared-local grammar
and all existing C checks pass. The baseline has no generated calls. The
direct-call extension below admits an acyclic graph of defined scalar callees.
Neither admits heap allocations, callbacks, loops, recursive records or arbitrary
foreign callees. The bounded standard-call extension below is separately measured.
Extending the grammar requires extending the resource model before
certification, not reusing a narrower profile's assumptions.

## Measurement authority

Reuse the exact bounded grammar walk and the existing registered-layout engine.
Count nodes, maximum visited depth, parameters, aggregate fields, final linked
identifier bytes, normalized comment bytes, automatic object bytes and value
object bytes. Pointer slots count their actual pinned pointer layout, not the
size of their pointee. Sum all scopes/functions conservatively; never assume
that lexical siblings cause a compiler to reuse automatic storage.

Automatic/value byte sums are not frame sizes. Before admission, derive and
measure a conservative frame estimate that also covers alignment, spills,
temporaries and supported sanitizer redzones. Because the baseline emits no
calls, it needs no generated call-path summation; instrumentation and the
documented native entry headroom still require their own evidence.

The source-byte estimate charges every visited node for two maximum-length
resolved identifiers, punctuation and maximum indentation, and adds normalized
comment bytes and six bytes per static-assertion diagnostic byte (the maximum
`[0xNN]` display expansion). Each of the ten required typed platform assertions
pays for its item and eight child/type nodes. Signature parameter declarators
are counted explicitly because
they are not expression-tree children. Assert that this estimate dominates
actual formatting throughout native, adversarial and capacity fixtures.
Checked arithmetic failures are resource diagnostics, never wrapped sizes.

## Native probes

These test inputs exercise the selected admission policy:

| Dimension | Candidate probe |
| --- | --- |
| Parameters | 127 initialized scalar parameters, all explicitly consumed |
| Fields | 256 scalar fields, all initialized, last field read |
| Block nesting | 45 nested lexical blocks with a returned scalar |
| Linked identifier | 256 bytes after generated-prefix allocation |
| Documentation | 1 MiB of normalized non-executable comment bytes |
| Combined | 127 parameters, 256 fields, 40 blocks, 256-byte linked names, 1 MiB comment |
| Scalar storage | 499 simultaneously live scalar locals; 500 rejects the node budget |
| Aggregate storage | 27 simultaneously live 256-field records; 28 rejects the frame budget |
| Expression nesting | 91 nested conversions; 92 rejects the nesting budget |

The initial combined probe passed with a maximum reported frame of 1,264 bytes
and a conservative source-byte estimate of 3,614,652 bytes. Evidence:
Linux/Bazel invocation `ed489cd8-85b6-4017-83b8-1eb3b0a4ec58`. All native runs
used a controlled 1 MiB process stack. GCC sidecars use -fstack-usage; the pinned
Zig frontend additionally needs an explicit -stack-usage-file output path.
Missing sidecars or missing generated-function rows fail the oracle.

## Admission implementation

The enforced resource policy selects 4,096 visited/declarator nodes,
96 maximum visited nesting, 127 parameters, 256 fields, 256 bytes per linked
identifier, 1 MiB normalized comments, 8 MiB source-byte estimate, and 1 MiB
native-frame estimate. Each assertion message is limited to 4,095 normalized
display bytes. Original bytes remain in CAssertDiagnostic; unsafe bytes display
as `[0xNN]`, avoiding numeric escapes rejected by the pinned Clang in unevaluated
messages. The length limit prevents strict GCC overlength-string failures and
counts expansion, not just input bytes. For the one-file profile the source bound
is also the package bound; paired files additionally require summed package
budgets, as specified below. These are target policy budgets, not limits on Rust syntax or
claims that larger C programs cannot compile.

The frame estimate is `4096 + 16*(automatic_bytes + value_bytes)
+ 256*automatic_objects + 64*nodes`, using checked arithmetic throughout.
All scopes/functions are summed, so no stack-slot reuse is assumed. The layout
factor covers by-value storage/copies and spills; per-object/per-node/fixed
allowances cover alignment, instrumentation and compiler temporaries. This is
a conservative pinned-compiler admission model to validate with frame reports,
not a formal theorem about arbitrary compiler versions. Actual-AST tests cover
many simultaneous locals/copies and expression-depth limits on both sides of
admission. Platform assertions count toward the same budget: 499 scalar locals
fit at 4,092 nodes; 500 use 4,100 and must fail certification.

The iterative grammar guard runs before recursive contextual passes. Statement
and value/place reconstruction are iterative, as is eager scalar storage
evaluation; derived Clone/PartialEq and other existing passes
remain recursive within the admitted depth. Do not advertise the complete
standalone C AST API as supporting arbitrary-depth host-stack-independent work.

Each native probe uses the real C registry/AST, shared projection, linker,
resource certification and public certified renderer. Compile and execute
under the declared Zig SDK and GCC 14.2.0 at
O0/O2, and separately under GCC ASan/UBSan at both optimization levels.
Record exact measurement values and compiler frame reports before selecting
limits. Then add actual-AST one-over rejections and arithmetic/missing-count
mutations. Test composed pressure, not only one dimension at a time.

The shared resource adapter rejects measured policy violations with
source-attributed TargetResourceLimit diagnostics; it does not repair syntax
or grant an unchecked bypass. Only shared post-link and resource certification
issues RenderReadyPackage<CDialect>. Native compiler success is test evidence;
production rendering never runs a compiler. Final integrated proof and review
are recorded in M35-01B; implementation status is not milestone completion.

## Defined scalar direct-call extension

M35-01D-02 extends target admission, not the completed Rust frontend surface.
Its [direct-call contract](rust-hir-direct-calls.md) is normative. Only resolved
direct calls with scalar `I32`, `Int` or `Bool` parameters and results are
admitted. Every target requires an earlier primary prototype and one matching
definition in the flattened unit. Private definitions and prototypes carry
internal linkage; shared callable visibility derives from the definition.
Unreferenced internal functions fail the strict warning-clean profile.

Storage checking derives closed-call evidence from actual definitions, using
a private reverse-topological closure. Evidence is keyed by the exact
registered function identity, not names or a caller-supplied purity flag.
Missing bodies, indirect/foreign calls, cycles, pointer signatures and global
effects cannot acquire that evidence; all ordinary callee numeric, scope and
storage checks still run. This is not an interprocedural result-range proof.

Resource measurement partitions the same AST walk by defining function and
collects exact direct edges. Each frame uses the baseline checked formula on
its own body plus a conservative copy of all non-body syntax charges. Parameter
storage, locals in every branch, return values and argument expression values
are included; no compiler slot reuse is assumed. For each function, its live
path bound is its frame bound plus the maximum bound of any direct callee.
Sequential or repeated calls do not add mutually non-live callee frames.
Admission uses the maximum path over all functions, including disconnected
components. Missing edge targets, mismatched inventories, cycles or arithmetic
overflow fail closed. The whole-unit node/source limits remain unchanged.
If the graph has no edges, retain the older, stricter whole-unit frame sum.

Native evidence must check each generated function's reported frame against
its individual estimate and independently sum reports along actual fixture
paths. A missing generated frame report remains a failure. Test-only,
externally observable function-address declarations retain optimized helper
bodies using their exact typed prototypes; these declarations are not part of
production generated output. Narrowly recognized compiler `.constprop`, `.isra`
and `.cold` derivatives are checked against their owner's frame estimate; add
the sum of every derivative frame to the maximum original source-graph path.
Unknown extra reports fail rather than being ignored. This deliberately charges
even mutually non-live derivative fragments. Native symbol tables must show external entry
and local helper definitions. Probe runs disable inlining and sibling-call
optimization to exercise live call frames, compile the certified bytes at
O0/O2 with both pinned compilers and ASan/UBSan, and execute under the same
controlled 1 MiB process stack. The launcher must fail if setting that limit
fails or its effective readback is not 1,024 KiB; failure injections cover both
conditions. This remains bounded empirical evidence for
the pinned toolchains, not a proof for arbitrary native compilers.

Required cases include chains, diamonds, repeated calls, composed locals,
the last admitted/first rejected chain, zero/three/127-argument calls, and all
three admitted scalar types. Test both sides of the 127-parameter policy
boundary separately from the unrestricted typed constructor. Record completed
gate evidence in M35-01D-02; do not claim HIR call support before its mapping,
compiler contracts and differential tests are complete.

## Public header/implementation extension

M35-01D-03B applies the same closed grammar to one authenticated public header
and one sibling implementation. Measure the complete registry/source inventory;
neither file is an independently certifiable fragment. Track per-file syntax,
docs, identifiers and storage, plus aggregate node/comment/source budgets.
Definitions determine frame ownership even when their primary function
declaration belongs to the header. Header syntax is charged conservatively to
real function frames; the header itself has zero executable frames. Reconstruct
one complete direct-call graph, preserving the existing no-edge bound.

Typed generated includes and guards add source bytes and guard identifiers
participate in identifier limits. Production resource certification first checks
per-file and aggregate policy, then checks exact structural formatter lengths
against each file estimate and their sum. Overflow or an underestimated output
is a diagnostic, not a certificate. Actual-AST tests hit the aggregate comment
and node limits exactly and one over while each individual file still fits.
Independent native consumers and retained-frame evidence follow the
[public-package contract](rust-hir-public-packages.md). Actual Rust package-mode
mapping and manifest parity remain subsequent M35-01D-03C/D obligations.


## Bounded standard truncation extension

M35-03A-02M-01 adds only the catalogue-owned FloatTruncate call. Its existing
typed signature and exact dependency traversal remain mandatory. The catalogue
supplies a nonzero 64 KiB native stack reserve selected after guarded-stack and
watermark measurements of the actual rendered three-package call path, including
GCC14/Zig O0/O2 and GCC ASan/UBSan. Observed total worker use was 6,264..9,152 bytes;
generated function reports were 8..48 bytes. Guard-page faults and a one-byte
allowance are negative controls. No arbitrary external call obtains a bound.

The per-function estimate adds the maximum known-call reserve in that body,
then ordinary direct/imported call paths compose the resulting bounds. Charging
a library reserve and an ordinary callee on disjoint paths is conservative.
Sequential standard calls share one reserve. For the older no-direct-edge
whole-unit policy, add the maximum known-call reserve to the whole-unit estimate.
All arithmetic remains checked. Neither the 1 MiB budget nor its complete-path
meaning is weakened. Actual-AST chain boundaries, exact/one-over budget controls
and missing/one-byte-cost mutations keep this extra charge observable.

See [truncation mapping](rust-floating-truncation.md) for the supported-platform
boundary. This empirical reserve requires renewed native evidence if the
supported library/runtime, compiler or instrumentation profile changes.
