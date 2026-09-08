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

## Tests and proof

- Every constructor/category has a positive unit test and an invalid shape/category rejection; no untested Other variant.
- Declarator/call argument, initializer shape, qualification, value/effect and place-mutation matrices.
- Explicit void versus nonvoid direct/indirect calls; alias-expanded signature
  checks; nonempty aggregate definitions; known constant/enumerator references.
- Actual C int comparison/logical results versus explicit _Bool conversion;
  array Read rejection and first-element address bounds; qualifier/provenance
  preserving object/void allocation conversions. ByteArray/F64Bits are expanded
  mapping inputs, never final renderer literals.
- Compile-fail public API controls and full cached tracked/release/eight-target gates.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02B; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
