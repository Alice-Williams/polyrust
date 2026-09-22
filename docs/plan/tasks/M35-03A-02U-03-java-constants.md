# M35-03A-02U-03 — Java finite-constant foundation

- Status: planned
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [C foundation](M35-03A-02U-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-finite-f64-constants.md)

## Contract

Extend exact public static final source-constant certification to primitive
Double and finite F64 literals, preserving owner/registration/provenance and
resource checks. No source admission or renderer/helper changes.

## Definition of done and tests

Private-reader matrices and public certificate tests reject wrong primitive,
boxed type, initializer, visibility/modifiers, resolved owner and import
authority. Separately compiled rendered producers/consumers agree with the
independent bit oracle under strict Java21 normal and interpreted execution.
Compiling zero-sign, f32-rounding and wrong-value controls are detected.
Source bounds and old constant tests pass. Full gate and fresh review pass;
preserve old output/WIP and commit/push this foundation separately.
