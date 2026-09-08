# M34A-11-02B — C expressions, declarations and files

- Status: complete
- Depends on: M34A-11-02A, M34A-11-01S, M34A-11-00R, M34A-10AB

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

## Definition of done

- Implement the closed expression/place/initializer/statement/declaration/file categories from grammar-inventory.md in focused modules.
- Keep void call effects separate from values, exact direct/indirect signatures, explicit pointer/numeric conversions, and registered member/parameter/local origins.
- Represent comments and Discard without raw executable text. Deliberately invalid native fixtures stay outside production ASTs and manifests. Do not expose verified/render-ready packages yet.
- Define automatic local declarations with registered type/owner and optional
  initializer; reject local static storage and const-without-initializer.
- Every Block carries its exact registered scope; Switch carries its exact
  registered control identity. Function body roots are explicit, not inferred
  from local spellings or statement positions.
- Implement the complete counted-loop payload in c/counted-loops.md: identity,
  counter/bound/Step::One references, explicit condition and body. No implicit
  initialization/update expansion and no claim that progress metadata is proof.

## Tests and proof

- Every constructor/category has a positive unit test and an invalid shape/category rejection; no untested Other variant.
- All closed signed/unsigned literal payloads retain their exact AST scalar
  identity, including Int/I32 and U64/Size despite ABI compatibility. Rust-width
  payloads reject out-of-range construction; zero/one Size literals compose the
  exact counted-loop form without an unmodelled token or implicit conversion.
- Declarator/call argument, initializer shape, qualification, value/effect and place-mutation matrices.
- Explicit void versus nonvoid direct/indirect calls; alias-expanded signature
  checks; nonempty aggregate definitions; known constant/enumerator references.
- Introduce closed KnownObject/known-constant identities required by the
  grammar, distinct from generated aliases. Accept borrowed FILE pointers and
  complete MaxAlign use; reject by-value/array/SizeOf/AlignOf FILE and generated
  name substitutions. Stage 03 completes catalogue metadata and linking.
- Closed PointerTest nullness for object/void/function pointers and SameSlot for
  exact owning-slot address types. Reject ordinary pointer Binary comparisons,
  two live-handle equality, mismatched slot types and implicit pointer truthiness.
- Actual C int comparison/logical results versus explicit _Bool conversion;
  array Read rejection and first-element address bounds; qualifier/provenance
  preserving object/void allocation conversions. ByteArray/F64Bits are expanded
  mapping inputs, never final renderer literals.
- Compile-fail public API controls and full cached tracked/release/eight-target gates.
- Retain a registered parameter's owner-signature alias provenance even when its
  local type is canonical (optional 02A review hardening, exercised here).

## Implementation sequence and module boundaries

This slice starts only after 00R/02A review closure. Follow Java's division
between structural models and constructors, not its Java-specific grammar.
Keep each cohesive production module below the size policy; no numbered
fragments or parallel string-producing path.

The pure measured scalar ABI calculations are extracted into the independently
reviewed [01S foundation](M34A-11-01S-scalar-abi-model.md). This slice consumes
those rules for operator signatures and contextual expressions; the extraction
does not waive its contract-review prerequisite or expose a certificate.

| Order / module family | Implementation | Required controls |
| --- | --- | --- |
| 1: type compatibility and operator signatures | Implement the pinned ABI's scalar identity, integer promotions and usual arithmetic conversions; retain registered nominal/alias provenance separately from C compatibility | Exhaust all 13 by 13 scalar pairs; Int/I32 and U64/Size compatibility; plain-char distinction; aliases, nested qualifiers and exact callback prototypes |
| 2: expression/place models and constructors | Private typed values and places, exact-width literals, actual enum-constant type, reads/addresses, unary/binary/conditional expressions and explicit conversions | Every closed variant; array-read and void-value negatives; const member propagation; same-layout wrong nominal; nested-pointer qualifier rejection |
| 3: callable/value-effect construction | Derive generated direct signatures and preserve indirect contract provenance; never accept caller-authored effect facts | Arity and each argument/result category; same-prototype wrong contract; separate void effect and nonvoid value; high-arity positive |
| 4: initializers and statements | Exact aggregate/array shape, local declarations and assignments; structural blocks, loops, switches and cleanup identities | Missing/duplicate/wrong-owner fields, wrong union member, const assignment, uninitialized const declaration, wrong branch/return category and crossed control identities |
| 5: declarations and file models | Derive declaration payloads from authoritative registrations; separate definitions, linkage/storage contexts and normalized comments | Wrong file role, duplicate/missing parameter bindings, invalid storage combinations, escaped adversarial comments and public API compile-fail controls |

Construction proves local shape/type relationships only. Each node retains the
actual references and children needed for independent contextual rechecking;
cached result types are not verifier evidence. Completeness, lexical dominance,
initialization, return coverage and case joins belong to 02C. Range, pointer
provenance, callable effects and sequencing belong to 02D. Catalogued dependency
resolution, actual file ordering and shared dialect binding belong to 03.
No constructor here yields a verified or render-ready package. The closed known
reference categories must not be simulated with generated names while their
catalogue owner is pending.

## Construction checkpoint (review pending)

The implementation now has separate model/construction modules for expressions,
places, calls, initializers, ordinary/control statements, declarations and files.
Library objects/constants, scalar/operator rules, comments and constant-expression
categories are separate cohesive modules. Heavy conversion/loop payloads are boxed;
lint allowances do not hide oversized enum variants. No new dependency was added.

Local constructors retain actual registry origins, children, scopes, parameters,
callable identities and initializer inventories. Source files are unresolved:
there is no public include/guard/raw-source input, verifier certificate or renderer.
02C/02D must independently rederive cached types and verify contextual facts,
including assertion truth, constant arithmetic safety and actual indirect-call
provenance. A successful shape constructor is not evidence that those stages ran.

Focused container Bazel C unit/rustdoc invocations passed through
874545c2-991d-4efb-94c2-80161681007b. The first full tracked gate
4a856908-a835-4eec-82e5-84b64549efd0 found Clippy enum-size/style findings;
its skipped tests are not passing evidence. Full checkpoint and independent
review evidence must be recorded before completion.

The subsequent full local construction checkpoint is green:

| Gate | Invocation | Result |
| --- | --- | --- |
| Every rule in tracked Bazel packages, including Rust Clippy/rustfmt, Buildifier and policy checks | 0aee23f8-6ff8-44e8-a82b-cb54a55b0094 | 439 rules; 314 test targets passed (48 executed, remaining results cached) |
| Cached release gate | 24d460cd-f9f3-42d3-b2ac-8c8959e9929f | 251 test targets passed |
| Eight-target conformance and deterministic manifests | 52ca8f57-ee98-4f34-8c42-d690c0f08fda | 50 cases and one portable test; evaluator and all eight targets agree; repeated manifests byte-identical |

Commands ran in polyrust-dev-step0 at /workspace through
`bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch`.
The tracked gate queries `kind(rule, set(...))` from host `git ls-files '*BUILD.bazel'`
and passes every returned label to `test --test_output=errors --noshow_progress`.
Release uses `test //:release_gate`; conformance uses
`run //crates/conformance:polyrust-conformance -- --all-targets --determinism`.
The user's untracked stdlib-abs example is excluded and untouched. C's unit
binary reports 91 passing tests; rustdoc controls also pass. The AST module
size audit has a maximum of 251 lines, with no new lint suppression.

These are construction and existing-output regression results, not native
proof of the new C renderer (which is still a later stage). Independent review
remains required before this task is complete or 02C implementation begins.

## Independent construction review and repair

A fresh Sol Extra High reviewer audited the complete immutable construction
checkpoint 2a21a145a5e1ad0d1dd5964393ebc7c00673dc47 against e47bcd9,
including every changed production module, surrounding type/registry foundations,
focused tests and the normative C contracts. The review was read-only, uncapped,
and did not treat deferred 02C/02D/03/04 work as already implemented.

All three findings were independently evaluated and accepted:

| Finding | Disposition and regression evidence |
| --- | --- |
| P1: SameSlot accepted const object/void pointees as owning slots | Require unqualified slot and effective pointee; pointer_qualification tests equal-type borrowed slots, qualified nested arrays, and mutable object/void/array controls |
| P2: AddConst accepted no-ops and rejected immediate array qualification | Require exactly Unqualified to Const through array layers, retaining bounds and deeper pointer qualification; tests cover scalar, void, pointer, nested-array transitions, no-ops, reversal, wrong bounds/types and unsafe nested-pointer changes |
| P2: promised constructor matrix incomplete | Add exhaustive operator construction/payload checks, unsigned/enumerator cases and foreign enum rejection, loop break and nested-block owner checks, every mutable place's exact assignment/const/type controls, and every definition origin/placement/linkage combination |

The main-agent handoff check found an additional implementation mismatch with
03's existing opaque-layout contract: aggregate completion incorrectly required
the declaration owner's file. Completion now retains the original nominal owner
while allowing public-header to corresponding implementation/private-header and
private-header to implementation placement. A full six-role matrix rejects
unrelated same-role files, direct generated/runtime crossover and test-owned
production completion. Same-file tag completion remains legal. Actual dependency
direction, public layout leakage and linked ordering remain independent 03 checks.

No finding was rejected. The review's optional alias-spelling surface expansion
is not required by the closed grammar: canonical element-qualified arrays remain
constructible and no certificate or renderer is exposed here. This does not waive
future lowering evidence if such a spelling-preserving alias becomes necessary.

Focused container Bazel invocation 1c5222de-d650-4f2a-9ece-0982abc3c30e
passed the C unit and rustdoc/compile-fail targets after these repairs.
The full repair checkpoint passed all local gates:

| Gate | Invocation | Result |
| --- | --- | --- |
| All tracked Bazel rules, Rust Clippy/rustfmt, Buildifier and policies | f607e314-8839-4643-bdd2-366ef24cda30 | 439 rules; all 314 test targets pass, 46 executed |
| Cached release | 1c086fd6-6e44-4f94-8e35-f617912d3815 | All 251 test targets pass |
| Eight-target conformance/determinism | fa62e696-9f98-4727-9fb8-98aea4775b3d | 50 cases and one portable test; evaluator and eight outputs agree; repeated manifests byte-identical |

The C unit binary now has 100 passing tests. These commands use the same
container, Bazel root and normal caches documented above. No new dependency or
lint suppression was introduced. A fresh independent review remains required
before 02B closes. Hosted CI run 34257746538 completed successfully for the
earlier construction SHA 2a21a14; that is not hosted proof of this repair delta.

## Second independent construction review

A fresh Sol Extra High reviewer audited immutable
d58484c6078c44cbff8f0e308eb6abb3e3b3dfe8, the repair delta and all affected
constructors/registry/contracts. It reported four findings, all accepted:

| Finding | Disposition |
| --- | --- |
| P1: matching FILE** still admitted as owning slots | The main agent raised this edge case and the reviewer independently confirmed it. Require the direct effective object target to be storable; reject direct/typedef FILE**, retain FILE*** storage-of-borrow positive |
| P2: intermediate-const slot guard lacked an independent control | Add equal matching T* const* operands, so type mismatch cannot mask removal of the qualifier guard |
| P2: declaration/file branch coverage missing | Add Union forward/complete declarations, incomplete/wrong-file enums, wrong-file prototypes/object declarations, exact declaration payloads and Definition file-grouping positives/negatives |
| P2: retained-child/call/static-initializer evidence incomplete | Assert actual place origins/path children, all literal categories, value/conditional/layout/pointer/conversion children, callable/effect/indirect nonvoid payloads, initializer aliases/elements/members, ordinary statement/file payloads; add arithmetic/address/nested static-initializer positives and dynamic/contaminated-child negatives |

The FILE regression was run before the fix:
9bf85711-0825-4b2e-9051-1cc9a6d5d7bf failed the single selected test because
SameSlot returned Ok for matching FILE**. This is expected reproduction evidence,
not a passing gate. After the storage-category fix and first evidence expansion,
ed7dbcfa-75b2-41e0-85b2-7736dd8b27ef passed C unit and rustdoc/compile-fail tests.
Additional exact absent-initializer/void-return/switch-break and alias controls
were then added. The complete second repair checkpoint passed:

| Gate | Invocation | Result |
| --- | --- | --- |
| All tracked rules including Rust/Bazel lint and policies | ff10016c-044c-4b7d-84f3-2f233d872348 | 439 rules; 314 test targets pass, 47 executed |
| Cached release | 0e281b3e-24aa-469d-86fa-cbc4da1c3918 | 251 test targets pass |
| Eight-target conformance/determinism | 3c0a7d3e-3a7a-4bfa-a127-9ea06c00d017 | 50 cases and one portable test; evaluator/eight targets agree; repeated manifests byte-identical |

The C unit binary reports 109 passing tests. The same documented container,
Bazel output root and normal caches were used. Hosted CI run 34261776669 passed
all eight jobs on the preceding d58484c repair; it is not hosted proof of this
new delta. A fresh independent review remains required before 02B completion.

The reviewer found no other production defect and confirmed the corrected
AddConst and aggregate-placement relations. No finding was rejected or
reclassified as an optional feature. No 02C/02D/03/04 obligation is being
represented as construction proof, and no new language feature was added.

## Third independent construction review

A fresh Sol Extra High reviewer audited immutable 9485acc633d17bf0eeaeb599694b63a261521bf6
and found one remaining evidence gap: the nonempty function-definition test
checked storage but not the retained parameter vector. Accepted: the test now
compares the entire Function payload, including both distinct registered
parameters in order, exact function, linkage and body. No production code
changed. The reviewer found no other substantive issue in its bounded pass.

The repair passed the same container commands and normal caches:

| Gate | Invocation | Result |
| --- | --- | --- |
| All tracked rules and Rust/Bazel lint | e04aa53d-b380-4553-a004-8a6232f2992a | 439 rules; all 314 test targets pass, four executed |
| Cached release | 29b1ead0-3bb4-48a3-aa3e-04854a726c6f | All 251 test targets pass |
| Eight-target conformance/determinism | 4f2f51f6-6685-40c9-a8d7-31cdb75aa9f2 | 50 cases and one portable test; all eight targets agree; repeated manifests byte-identical |

A fresh Sol Extra High reviewer audited the final immutable repair
98bac7e7d993a3093d696bc99afdb924a722fc15 and returned PASS with no substantive
findings. It confirmed the two separately registered parameter identities,
indices, owner/type equality and independent expected vector make the assertion
non-vacuous; dropping, reordering or duplicating parameters fails. No finding
was rejected and the scope of later contextual/proof stages is unchanged.
Final documentation/Buildifier invocation 233a81a3-b202-46a2-b734-92cd7560a478
passed. An earlier invocation 738c1189-0870-4c25-a269-5f21aa638ecf used a
nonexistent tools package and ran no tests; it is not passing evidence.
02B is complete. This closes construction only, not C migration or certification.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02B; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
