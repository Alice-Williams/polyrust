# Certified Java Result nominal imports

- Status: implemented and verified by checkpoint 03B
- Task: [M35-03A-05A-03B](../../../../plan/tasks/M35-03A-05A-03B-java-result-imports.md)
- Parent contract: [scalar results](rust-scalar-results.md)

## Authority and identity

The source-owner dependency API remains the publication boundary. A local
JavaScalarResultFamily is evidence about declarations in one immutable Java
certificate, not a dependency owner, Rust type identity or import permission.
Dependency publication validates any descriptive family selection against the
same containing certificate before allocating its existing JavaDependencyPackage
authority. Failure is atomic and produces no exported handles.

An exported nominal identity is the original package authority, original family
identity and an enum role: interface, success or error. Paths and simple names
are derived from the original resolved declaration inventory, never supplied as
authority. Independent certification of equal text remains a different identity;
clones of one authority remain equal. A relay retains the defining authority and
path. It must not reconstruct the family or mint a replacement owner.

Constructor and accessor handles retain that same nominal authority and their
exact roles/signatures. Success construction takes one int; error construction
takes no parameters; the success accessor returns int. The interface has neither
a constructor nor payload accessor. There is no public direct-field handle for
the private record component. All enclosing declarations must be public before
their nested nominal types can be exported.

The owner authority stores validated layouts, not exported handles pointing back
to itself. Derived handles reference that authority. This prevents strong Arc
cycles when exported function signatures mention their owner's result types.

Publication also supports an owner containing only selected public families.
Such an owner must retain the existing explicit JavaSourcePackage root/export
metadata in its canonical certificate; a package name alone is not provenance.
An empty function/constant binding set is then permitted only because a nonempty
exact selected public-family inventory exists. Do not invent Rust bindings or
declaration IDs for these synthesized types. An entirely empty API still rejects.
Treat each selected interface/success/error triple as one exact inventory unit:
unselected extra public adapter declarations reject, and private source records
continue through their existing separate strict verifier.

## Language vocabulary and linking

Keep the closed JDK/legacy-standard symbol enumerations as their existing leaf
catalogues. Introduce explicit language-owned wrapper enums for referenced types,
constructors and methods, distinguishing standard symbols from certified
dependency symbols. The shared TypedAstDialect/LinkerDialect known-symbol slots
carry those enums. Do not add a second string-based foreign-type renderer or a
copied generated declaration. Existing convenience constructors may wrap standard
symbols without changing their generated text.

Local definitions retain GeneratedTypeId; foreign references retain authenticated
nominal handles. Public signature projection must translate the former to the
original owner's exported identities, then bind those identities in the consumer
scope. Producer arena indices must never become consumer declaration indices.
Exact signature comparison uses original nominal identities, not names or
structural similarity, including parameters, returns and receivers.

The imported variant-to-interface relation requires the same consumer scope,
original family authority and correct enum roles. Calls, checked casts and type
patterns may use that relation. Exact-type local initializers, assignments,
returns and conditional branches use an explicit typed upcast to the interface;
this increment does not broaden their existing general subtype inference.
Reference equality remains exact-type rather than introducing a one-directional
comparison rule. Source-owner bodies admit only the specific safe upcast and
guarded observation forms, not general Object downcasts.

Implement three explicit signature phases: the certified producer declaration
signature, the post-authority exported signature over original nominal handles,
and its checked consumer-bound signature. The current scalar-only direct copy of
JavaDependencyFunction.signature is insufficient for nominals. A same-numbered
GeneratedTypeId from a different arena must never compare as a matching import.

Use the existing shared known-type/constructor/method catalogue contracts for
symbol discovery and resolution. A type appearing only in a parameter, return,
local or pattern must discover its dependency. Constructors and accessor calls
discover both the exact member and its owner. Import membership never grants
permission to declare the foreign type or extend its sealed family. The ordinary
renderer emits only the certified resolved name and typed syntax.

## Consumer scope and closure

The consuming JavaDependencyScope builder registers original families and their
selected nominal/member uses; freezing prevents later mutation. A reference used
in a file must be present in that file's frozen scope. Importing a function also
registers the nominal types required by its signature, preserving the original
owner through relays. This registration must be atomic on conflict or capacity
failure. Merely holding an exported handle does not register it in a consumer.

Registration incrementally enforces 100,000 distinct bindings, 1,024 original
owners, 64 MiB of qualified-name bytes and 65,535 bytes per qualified name.
Repeated registrations in the same scope do not recharge those budgets. An
owner's retained closure is traversed once per builder; repeated imports must
not repeatedly walk the complete graph. Certification independently checks the
frozen scope, including unused type/member-only registrations. Opaque imported
type/accessor handles and bound callable signatures use shared immutable storage
so nominal metadata does not inflate every AST expression or statement.

Apply the existing one-authority-per-owner and consumer/owner namespace-overlap
checks to type-only and member-only dependencies, including their transitive
closure. Reexports retain original identity without making all registered or
private types public. Reject additional foreign subtype declarations and mixed
same-shaped families before rendering.

## Verification and resource audit

The implementation must update these boundaries together:

- Type equality, arity, assignment, conditional joins, arrays/generics policy,
  instanceof/pattern compatibility, checked upcasts and null operations.
- Constructor/member signature checks, record accessor resolution, visibility,
  sealed inheritance restrictions and local-versus-foreign declaration ownership.
- Source inventory/descriptions, public signature projection, import discovery,
  frozen membership, dependency catalogue and original-owner closure.
- Source-size reservations, node/depth limits, declaration/binding/name/owner
  budgets and JVM type descriptors, constants and method/code reservations.

Every unsupported operation remains a diagnostic. Closed Result publication
requires the narrower body profile as well as valid Java and canonical variants.
Retain bounded call-graph checks and reject unadmitted effects/expressions; pure
flags are descriptive, not evidence. Compiler source metadata is not relaxed by
this target-level work and no synthetic Rust declaration IDs are invented.

## Required evidence

Compile producer, relay and consumer independently using pinned Java21 strict
lint, then run normal and interpreted modes. Include both variants, signed
extrema, zero, construction, copy, call/return, pattern payload and null policy.
Verify deterministic text, no copied family/runtime file, and original declaring
class paths through the relay.

Positive type-only imports must work. Negative tests cover absent scope, wrong
scope, wrong authentic owner, same-shaped family substitution, wrong member or
constructor, altered signatures, private enclosing types, extra sealed subtype,
mixed certificates, namespace overlap, closure conflicts and atomic failure.
Test exact and one-over resource limits for dependencies without callable uses;
compare emitted classfile measurements against certified reservations. Preserve
existing outputs and complete the full release/lint and independent-review gate.
