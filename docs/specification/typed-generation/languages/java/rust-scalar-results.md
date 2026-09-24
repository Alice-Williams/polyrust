# Rust scalar results in Java21

- Status: typed component foundation complete; complete family/imports still planned
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

Use separately compiled original producer/relay/consumer packages under strict
Java21 compilation and normal/interpreted execution. Pristine value controls and
test-only native call observers must distinguish correct selected-arm execution
from value-preserving eager-arm, duplicate-scrutinee and reordered-call faults.
Exercise both variants throughout the input corpus, not only the first failing
input. External negative consumers prove subtype closure and private boundaries.
Null policy, JVM limits and source-size accounting remain part of the proof.
