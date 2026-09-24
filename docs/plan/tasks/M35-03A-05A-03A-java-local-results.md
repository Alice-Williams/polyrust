# M35-03A-05A-03A — Local Java scalar-result declarations

- Status: complete
- Parent: [Java result transport](M35-03A-05A-03-java-results.md)
- Depends on: [C result transport](M35-03A-05A-02-c-results.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-scalar-results.md)

## Contract

The first independently gated prerequisite is
[03A-01 synthesized component identity](M35-03A-05A-03A-01-java-components.md).
It is complete with all 1,041 release/lint targets and independent review passing.
It establishes component references, not a closed Result family or source
admission. The closed-family checkpoint below completes this local parent.

The remaining local work was completed in
[03A-02 closed families](M35-03A-05A-03A-02-java-local-families.md): exact family
certification, generated interface conversion, guarded observations and native
boundary/resource evidence. All 1,041 release/lint tests pass, including 459 Java
unit cases and the separate native partitions; independent broad GPT-6-SOL
review is clean. Both review-requested boundary tests were added. Existing
generated outputs and 45 unrelated WIP files are unchanged. This does not widen
dependency publication or admit Rust Result source programs; 03B/03C and 05A-04
remain open.

Build one ordinary sealed interface and two immutable variants through the
existing typed Java declaration tree. The success record stores one primitive
int; the error variant stores no payload. Preserve exact declaration identities,
permits/implements relationships, constructors and component identity. Do not
invent Rust declaration IDs for synthesized target helpers or classify them as
legacy runtime members. Add an explicit typed synthesized-component origin if
the existing Core/RustSource/Runtime vocabulary cannot represent this ownership.

This is target-level transport, not proof that a compiler Result instance was
mapped correctly. Keep compiler admission closed. Start with local declarations
and scalar-facing entry points; do not widen dependency publication until 03B.
Do not weaken the existing private source-record verifier into a general class
allowlist. Result-family verification has its own focused module and inventory.

Use typed constructors and guarded pattern bindings for observations. Materialize
the scrutinee once. An error cannot expose a success component; same-shaped
families remain distinct. No Runtime.Result, raw Object transport, null sentinel,
unchecked cast or exception-as-Err. Null arriving from foreign Java is outside
the Rust value domain and follows the existing public boundary policy.

## Definition of done and tests

- Independently certified local families compile under Java21 with strict lint
  and run under normal and interpreted execution. Cover both tags, zero, signed
  extrema, construction, copies, calls, returns and fallback.
- Wrong family/variant/constructor/component, extra subtype, changed permits or
  implements, mutable payload, unguarded access and unsupported payload shapes
  reject before rendering. Synthesized origins confer no unchecked authority.
- Strict dependency publication rejects newly introduced unsupported public
  nominal signatures until the next checkpoint establishes import authority.
- Node, nesting, declaration, method/JVM and source-output limits remain enforced.
  Rendered code is structural and adds no runtime file or source-text template.
- Preserve existing outputs/WIP; pass full Linux Bazel release/lint and fresh
  GPT-6-SOL review before a separate milestone commit/push.
