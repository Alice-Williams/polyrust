# Rust scalar results in Java21

- Status: local component/family foundation complete; imports and source admission planned
- Contract: [shared](../../rust-scalar-results.md)

Use an ordinary source-derived sealed result interface with an immutable
success record carrying primitive int and a payload-free error implementation.
Register the exact permits/implements, constructors, fields and dependencies
through typed Java AST APIs. The error implementation represents only the
admitted opaque standard error observations, not a general singleton type.

Each canonical source instantiation owns distinct target symbols. Do not share
Runtime.Result, use raw Object/null as a variant, throw to represent Err, or
add unchecked payload casts. Match lowering uses certified variant knowledge
and evaluates the scrutinee once and only the selected arm. Foreign null is not
a Rust result value and must follow the established public boundary policy.

Incidental Java object allocation is not support for Rust-owned heap payloads.
Prove closed variants, immutable exact payloads, original import authority,
private implementation details, JVM capacity and strict separately compiled
normal/interpreted consumers before source admission.

## Layer responsibilities

The local target AST retains exact generated declaration identities, permits,
implements, constructors and success-component ownership. Synthesized result
components need a typed origin tied to their actual owner, not invented Rust
declaration IDs or legacy runtime-member identities. The target verifier checks
the complete closed family and every use against those identities. A verified
target family alone is not a compiler-authenticated Rust Result instantiation.

Dependency publication derives opaque nominal/member/constructor witnesses from
the complete original certificate and exact public signatures. A consumer scope
authenticates foreign references separately from locally owned declarations.
Relays preserve original identity and defining package paths. Private or unused
foreign registrations are not public type exports. Same names/shapes do not
authorize substitutions or additional sealed subtypes.

Language-owned referenced-type enums distinguish standard-library declarations
from certified dependencies. Structural symbol discovery and the existing shared
catalogue/import policies determine names and imports; the renderer never
reconstructs foreign declarations or interprets source-like strings. Audit all
affected constructor, pattern, assignment, field, visibility and capacity checks.

The dependency/body verifier admits only the explicitly specified closed family;
it does not turn the existing private source-record rule into an arbitrary-class
allowlist. Unsupported signatures remain diagnostics until their ownership and
resource obligations are implemented. Compiler source admission follows only
after local transport, imported transport and measured execution tests pass.

## Required execution evidence

The component foundation uses `JavaSynthesizedField { owner, role }` with the
enum role `ScalarResultPayload` and exact primitive-int storage. The owner is an
ordinary generated record registered as a synthesized interface adapter. Field
declarations, direct references and accessors authenticate that same owner/role,
name and type; structural name-only reads cannot substitute for it. Constructor
assignment and cross-nest access remain subject to normal final-field/privacy
checks. This role is descriptive storage metadata, not source provenance or a
certificate of a complete closed Result family. The family verifier must still
check the entire declaration set, canonical constructors and absence of custom
accessor implementations before claiming Result observations.

`JavaScalarResultFamily` is the narrower local declaration proof. It is derived
only from an immutable `RenderReadyPackage<JavaDialect>` and a descriptive
selection of three generated type identities; the proof retains that original
certificate. The same top-level nest must contain an empty sealed interface,
one single-int success record and one zero-field error record. Their identities
must be distinct, their synthesized adapter origins exact, and their
permits/implements inventory complete. The variants have only canonical
constructors with matching explicit visibility; success assigns the final input
directly and error has no initialization. Custom accessors or extra members are
valid in general Java but cannot acquire this narrower proof.

Generated adapter coercions are separate from checked Core implementation
witnesses. They authenticate the source record, destination empty sealed
interface, actual local implements edge and both synthesized adapter origins.
The renderer retains the checked reference upcast to preserve the interface's
static type in pattern contexts. This is not an unchecked payload cast. Normal
lexical checks continue to reject a success-pattern binding used outside its
guard. Public reference entry points reject foreign null through the typed JDK
non-null operation, not by treating it as a Result error variant.

This family proof covers declarations, not arbitrary facade-method semantics or
Rust source equivalence. All uses retain ordinary Java syntax/scope verification;
the compiler lowering and the later bounded dependency-body profile must impose
the source operation/evaluation contract. Local family proofs do not export
nominal dependency handles or bypass the existing source-facade verifier.

Use separately compiled original producer/relay/consumer packages under strict
Java21 compilation and normal/interpreted execution. Pristine value controls and
test-only native call observers must distinguish correct selected-arm execution
from value-preserving eager-arm, duplicate-scrutinee and reordered-call faults.
Exercise both variants throughout the input corpus, not only the first failing
input. External negative consumers prove subtype closure and private boundaries.
Null policy, JVM limits and source-size accounting remain part of the proof.
