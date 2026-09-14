# M35-01B — Lower Rust HIR into the existing C AST

- Status: complete
- Depends on: completed [M35-01](M35-01-rustc-adapter.md)
- Contract: [C HIR lowering](../../specification/typed-generation/languages/c/rust-hir-lowering.md)

## Goal

Replace the experiment's miniature C model with the decided backend-c types
and certification pipeline for the explicitly admitted no-heap subset.

## Completion

The earlier intermediate evidence below is historical. Executable mappings
closed in M35-01B-02; the remaining public-header and separate-crate boundary
obligations now close with [M35-01D-04D](M35-01D-04D-separate-crate-proof.md).
Final Linux/Bazel gate 5630aeea-7116-4b63-bfcf-5a07969cb6a2 passed all
343 tests, including strict native Rust/C parity, private consumers, compiler
provenance, existing resource/certification guards and all historical Java tests.
Fresh independent C closure review is clean. This is complete no-heap existing-
type integration; heap cleanup and the Java retrofit are separate tasks.

## Definition of done

- Isolate unstable compiler inspection in the compiler/C bridge.
- Use CObjectType, CValue, CPlace, CInitializer, CStatement, CBlock and
  registered declaration/member/local/file references; no duplicate target AST.
- Add authenticated typed compiler-source provenance to the existing origin
  model; never fabricate a CoreDeclaration or use a spelling as identity.
- Implement the exact first-subset mapping table with capability-owned typed
  inputs, executable registrations and explicit unsupported diagnostics.
- Preserve nested scopes, compiler adjustments, field identity and sequencing.
- Carry source-file/module ownership and resolved visibility from the start;
  the crate/API boundary proof is completed in M35-01D, not discarded here.
- Connect unresolved files to shared verification/linking/post-link/resource
  certification and RenderReadyPackage<CDialect>; no renderer bypass.
- Finish the necessary C dialect/linker/certifier/renderer work for this
  closed subset before claiming integrated generation. Their current absence
  is not permission to keep the miniature renderer as a production fallback.
- Do not claim complete portable Supports slots or remove legacy ownership
  obligations until their separate replacement evidence is complete.

## Tests and proof

- Compile-fail wrong-category/registry/owner/scope/signature/certificate cases.
- Exact mapped AST assertions for all initial types and expressions.
- Native tests for shadowing, source/declaration field order, nested shared
  references, unused bindings and all admitted branch forms.
- No-goto and scope-preservation mutations; unknown-adjustment negatives.
- C17 O0/O2 Rust parity, public-header consumer, pinned ABI checks and
  ASan/UBSan for admitted pointer mappings.
- Type-derived include matrices and three-render byte determinism.
- Full existing-type integration, Rustfmt, Clippy, Buildifier, docs and release
  checks pass in the container; independent review and documented evidence.

## Work organization

The remaining executable-mapping refactor follows
[M35-01B-02](M35-01B-02-source-capabilities.md), after the certified-output
checkpoint below. It does not reopen or replace the decided C AST.

M35-01B-02 is now locally complete: full release/compiler invocation
`defdfe9a-3c06-4ba9-9521-ca1b0ea33871` passed all 280 tests after independent
review and the accepted source-scope abstraction repair. All eight owner
bindings, compile-fail contracts and four compiler-generated native/sanitizer
matrices are covered. The remaining public-header consumer obligation closes
with the M35-01D API integration; it is not proved by the current .c-only harness.
Documentation work may proceed from this certified capability checkpoint while
that boundary proof keeps the parent integration milestone open.

### Current intermediate checkpoint

The isolated adapter now declares the existing backend-c/codegen libraries as
Bazel dependencies and lowers the admitted HIR into their actual registry,
types, places, initializers and scope-owned statements. It runs the existing C
context, constant/layout, sequencing, numeric, index and storage checks. Source
origins retain compiler declaration hashes, module, span, visibility and docs.
No new parser or borrow checker is involved.

The first broad local gate (before follow-up admission repairs), invocation
`eea872dd-2e7a-48ca-98f3-6af9309b732b`, passed 265 release/prototype checks.
Clippy-driven fixes share both source metadata and C declaration keys via
immutable Arc payloads; registry branding and value-based key equality remain.

Review found custom record representations were admitted without a mapping,
alias normalization discarded required provenance, and manual metadata tests
did not exercise compiler extraction. All three findings are accepted. New
boundary fixtures reject packed/aligned records and alias type uses without
opening outputs. A separate compiler-backed provenance target checks the
actual C tree's identities, module, visibility, location, docs and member owners.
Post-repair Linux/Bazel invocation `01d7b735-b952-49ba-9fbf-1e14513ccb39`
passed all 266 checks, including the alias-constructor regression. Independent
scoped re-review passed with no remaining substantiated core defect in this
intermediate bridge. This verdict explicitly excludes the newer dependency
collector and the unfinished production certification/linking work.

The next integration slice adds a typed C dependency catalogue and exhaustive
structural traversal. It derives headers/library requirements and authenticated
nominal/alias/function/object references, distinguishing declaration-only use
from complete-type use. It does not render or certify a package. The compiler
bridge now retains its frozen registry alongside the source tree so downstream
passes can authenticate it rather than reconstruct signatures from strings.

Dependency integration Linux/Bazel invocation
`0a94f9c1-b085-40ab-8a5c-8749a6c13391` passed all 266 checks, including the new
unit matrices and compiler-backed dependency assertions. Scoped dependency
review is separate from the already-passed compiler bridge review.

The shared target-origin enum now also has a dedicated RustSource variant,
retaining the same immutable metadata used by C. This prepares the checked
registry projection and Java retrofit without fabricating CoreIR provenance;
it does not authenticate caller-constructed metadata or complete the projection.

Shared-origin compatibility gate `f10d6235-bc87-4974-995d-da2a549b88e5`
passed 266 checks. Subsequent dependency review found indirect calls incorrectly
collecting aliases from their proof-only witness signature. This finding is
accepted: discovery now uses the actual function-pointer signature, preserving
its declared aliases and upgrading by-value result requirements at invocation.
The dedicated distinct-alias regression passed with the C suite in invocation
`c2ac2176-1e9d-4971-8d13-78ca336bd238`. Full post-repair invocation
`06d0f465-5edc-4bdc-9b54-b1b2a361a213` passed all 266 targets, including Rustfmt,
Clippy, Buildifier, documentation, native language/real-world regressions and
the compiler-backed proofs. Tests remained enabled; no commits or pushes were
made during this intermediate slice. Final independent scoped review passed
for the dependency collector and shared Rust-source metadata addition, with no
remaining substantiated core defects. The review covered every current C AST
variant and the indirect-call repair; it did not certify the still-unimplemented
renderer, linker, visibility/resource policy or public-header consumer path.

Linux/Bazel invocation `906fe8b6-d4ce-42a3-b631-7f2fa9a75d3a` passed eight
targets, including the existing C/shared-codegen unit suites and all compiler
prototype regressions. This is intermediate evidence, not milestone completion:
the original miniature renderer still produces the experimental C artifact.
Replacing it with the shared certified path, completing executable capability
slots, dedicated mapping/visibility/resource proof and Java retrofit remain.

Split source admission, provenance, types, expressions, control and tests into
focused modules. Keep capability mappings in one file per capability. Use
separate Bazel targets for compiler bridge versus reusable backend AST;
merely splitting Rust files is not an independent cache boundary.

## Commit gate

### Shared projection and private structural-spelling checkpoint

The existing frozen C registry now projects into the shared package with exact
bidirectional bindings. The shared package hook reconstructs and compares all
registrations, signatures, origins, files, groups and typed payloads, and runs
again at linking and post-link verification. Mutation tests reject omitted or
extra callables, changed signatures/types/origins, missing groups, renamed files
and foreign registries. Real compiler-produced C trees now pass this shared path.
Independent scoped review found no substantiated core defect in this projection.

The next private spelling slice covers the closed scalar/record/shared-reference
profile with linker-owned identifier maps and typed import formatting. It is
not a publicly callable raw renderer and cannot issue a certificate. GCC 14.2.0
O0/O2 tests execute scalar and nested-const-reference record output, with exact
formatting and three-render checks. Explicit unused-parameter/local Discard
operations are introduced in lowering rather than invented by formatting.

Full Linux/Bazel invocation `76915a2c-13aa-4a83-ba74-68ccc514a0d8` passed all
266 targets, including the compiler proofs, all existing release regressions,
Rustfmt, Clippy, Buildifier, docs and dependency-source policy. Earlier failures
in import-renderer policy placement and an overly restrictive provenance-fixture
local count were corrected and retested; no tests were disabled. Structural
renderer review and measured resource/native-boundary proof remain in progress.
The experimental miniature renderer is still active: this is not completion of
M35-01B, and neither C certification nor Java retrofit is claimed complete.

Spelling review accepted and repaired two findings: definition-local parameter
constness now comes from CParameterRef (prototypes still use signature types),
and the structural import-lint boundary uses the absolute external trait path
rather than accepting a locally counterfeited same-name trait. Invocation
`6c36a73a-0e96-4c0c-a909-b0c0c88c33e7` passed the expanded six-comparison,
integer-boundary and nested-reference matrix under pinned Zig/GCC at O0/O2,
plus GCC ASan/UBSan, source policy/failure injection and Buildifier. Scoped
independent static re-review passed with no remaining core spelling defect.

Reviewer feedback about concatenated directive literals also added flat
string-literal concat lint coverage. Arbitrary runtime and mixed-literal Rust
computation analysis was not accepted as a core renderer defect: the scanner
is explicitly a lexical supplementary guard, not the typed-generation proof.
Its scope is documented without claiming universal constant evaluation.

The next resource slice reuses the same bounded profile walk and existing
checked C layout engine to measure shape, automatic/value storage and a
conservative source-byte bound. This alone is not a stack/compiler-capacity
proof. Select and probe numeric limits, test their one-over rejections, and
verify source-size/frame estimates before enabling resource certification.
No measured-only report may mint RenderReadyPackage.
The [no-call resource profile](../../specification/typed-generation/languages/c/hir-resource-profile.md)
records the candidate probe inventory and required evidence before activation.

The candidate block-nesting probe found a host-stack overflow in contextual
statement reconstruction. Reconstruction now uses an explicit postorder work
stack, preserving the same constructors and exact final structural comparison.
Label placement also walks label chains iteratively. Independent Sol Extra High
review found no remaining core defect under the closed profile's iterative
depth/node admission guard. Derived Clone/PartialEq and other contextual passes
remain recursive within that bound; arbitrary-depth standalone legacy checking
is not claimed stack-independent. The explicit 2 MiB Rust-thread regression
passes without raising the process stack setting or skipping checks.

Native capacity invocation `ed489cd8-85b6-4017-83b8-1eb3b0a4ec58` passed the full
C unit suite, including all five individual probes and one combined-pressure
fixture (127 parameters, 256 fields, 40 nested blocks, 256-byte linked names,
1 MiB comment). Every fixture executes with a controlled 1 MiB native stack
under pinned GCC and Zig at O0/O2 and GCC ASan/UBSan at O0/O2. The combined
fixture's largest reported frame is 1,264 bytes (GCC ASan O0); its conservative
source-byte estimate is 3,614,652 bytes. Zig requires an explicit Clang
stack-usage-file output; the oracle fails if either compiler omits its report.
These measurements do not yet complete resource certification. Candidate
source-attributed limits, expression-depth and many-local/frame-boundary probes
and their independent review remain in progress. No migration push was made.

Follow-up invocation `2ad4b4e4-974b-4ecb-a5bb-4a8543fdd2a8` passed all 603 C
tests plus Clippy, Buildifier and both source-policy gates. The candidate
expression-depth probe exposed a second host-stack issue: values/places now
reconstruct iteratively, and eager scalar storage evaluation uses an explicit
work stack while conditional/short-circuit/pointer semantics remain unchanged.
Independent scoped reviews passed both repairs. Review also found missing/late
prototypes, late record declarations and unused bindings could violate strict
native warnings; the closed profile now checks actual ordered declarations and
typed binding uses rather than silently inventing renderer fixes.

Every native frame report is now checked against its own source-derived bound,
not just the global budget. Probes include 500 scalar locals and 27 simultaneous
256-field record copies. The 28-copy actual AST measures 1,058,816 estimated
frame bytes and is rejected by the 1,048,576-byte candidate budget. Checked
arithmetic and exact one-over tests cover every policy category. Independent
resource review found no remaining core defect in this candidate model, while
explicitly distinguishing empirical pinned-compiler allowances from a universal
compiler theorem. Resource enforcement, required typed platform assertions and
the public certified-renderer/frontend switch remain unfinished.

### Certified output checkpoint

Linux/Bazel invocation `8887f5cb-62b5-497e-b3ce-52eaed65804f` passed all 266
release/compiler integration targets after activating resource certification
and CStructuralRenderer. The frontend now returns RenderReadyPackage<CDialect>
and has no miniature AST/renderer or alternate output path. All three compiler
fixtures pass 8,204-input Rust/C parity at O0/O2, deterministic output, borrow/
type/admission negatives, provenance and unchanged-output checks. Independent
Sol Extra High review found no remaining core defect in this frontend switch.

The earlier C-only gate `9ddeac14-81d2-4bac-8649-f82f03f02249` passed 607 tests
and all four lint/policy targets. The final gate adds hostile diagnostic and
message-capacity regressions. Typed layout assertions use the existing C AST
and derive Stdint/Stddef through type references. Their 90-node cost changes
the scalar-storage boundary to 499 admitted locals (4,092 nodes), with 500
rejected (4,100). No capacity was raised to accommodate an old fixture.

Adversarial native test `83bf8f18-75d5-4705-a297-a159ae805be2` exposed pinned
Clang's rejection of numeric escapes in unevaluated static-assert messages.
Review also found oversized diagnostic strings could pass certification but
fail strict GCC. Both defects are accepted and repaired: CAssertDiagnostic
retains original bytes and constructor-normalized printable presentation,
with a 4,095-display-byte resource limit. Native tests cover all 256 byte values,
preprocessing hazards, 4,095-byte ASCII and 4,092-byte expanded positives;
4,096/4,098-byte actual AST negatives reject before certification. Deliberately
wrong platform assertions fail both compilers. The final full gate proves the
repairs without suppressing warnings or skipping existing tests.

Independent Sol Extra High re-review of the activated platform/resource/public
renderer boundary is clean after both repairs. It confirms that private
diagnostic presentation, reference-derived imports, checked source accounting,
post-link verification and resource checks operate on the same immutable
payload. The accepted documentation clarification distinguishes ten mechanical
layout guards from the separate mandatory full pinned-ABI probe contract.

Visible ignored examples are refreshed from the certified Bazel outputs in
experiments/rustc-frontend/output/{model,alternate,scopes}.c. This checkpoint
does not complete M35-01B: executable capability-owned mapping slots and the
remaining integration/boundary proof still need work. Java retrofit remains
pending. No commits or pushes were made during the migration.

Commit this completed integration separately from owned-allocation work. Hold
pushes until the complete C/Java migration gate is green, per the latest user
instruction. Keep all existing tests enabled throughout the migration.
