# M34A-11-02B — C expressions, declarations and files

- Status: in-progress
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

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02B; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
