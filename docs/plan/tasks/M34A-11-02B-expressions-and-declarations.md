# M34A-11-02B — C expressions, declarations and files

- Status: planned
- Depends on: M34A-11-02A, M34A-11-00R, M34A-10AB

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

## Tests and proof

- Every constructor/category has a positive unit test and an invalid shape/category rejection; no untested Other variant.
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

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02B; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
