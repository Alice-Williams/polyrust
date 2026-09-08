# C17 module ownership

- Status: normative for M34A-11
- Reference structure: the completed Java backend, not Java grammar or runtime code

## Production modules

| Module | Owns | Must not own |
| --- | --- | --- |
| `ast/` | C types, declarators, expressions, statements, declarations, lexical/type checks | CoreIR decisions or rendered source |
| `capabilities/` | One executable mapping per supported portable capability and its exact plan | Independent duplicate mappings |
| `preflight/` | Dynamic admission from the same registered mapping slots | Rendering or support guessed from target strings |
| `lower/` | Verified CoreIR traversal, symbol requests, sequencing and mapping input assembly | Operation semantics implemented outside their owning mapping |
| `dialect/` | C catalogues, reference identity, includes, namespaces and placement | Portable evaluation |
| `ownership/` | Borrow/owner/allocator/initialization/cleanup proof states | Textual cleanup snippets |
| `runtime/` | Structural scalar, text, bytes, collection and lifecycle implementations | Embedded C/header source |
| `resources/` | Linked target capacities and checked size/layout accounting | Syntax repair or arbitrary portable arity caps |
| `render/` | Certified C grammar spelling, declarator parentheses and formatting | Includes discovery, ownership decisions or compiler processes |
| `tests/` | Focused Rust tests, mutations and compiler harnesses | Production library inputs |

Root files are entry points and explicit re-exports. C does not acquire Java
classes, inheritance, exceptions, packages, JVM limits or JDK catalogue types
through copying. Shared phase machinery is reused; C grammar is independently
modelled with closed enums and private invariant-bearing wrappers.

The C declaration registry is authoritative and frozen only after its complete
declaration/helper inventory is known. The later CDialect/shared unresolved
package owns that immutable payload; shared GeneratedType/Callable/Value/File
registrations are a checked projection, with bidirectional identity bindings,
not a second independently authored source of signatures. No mutable registry
view survives verification. Ordinary Rust equality authenticates references;
address-free canonical inventory projections determine stable ordering and
phase evidence. Ephemeral registry authentication never names an output symbol.

Stage 02A admits generated registrations only. Stage 02B introduces distinct
closed known-object/constant identities and their actual AST type categories;
stage 03 completes the authoritative header/completeness/layout metadata and
callable signatures/contracts, with shared binding and dependency resolution.
Known references cannot be fabricated by assigning library spellings to
generated keys or by attaching caller-supplied type/effect flags.

## Size and build policy

Aim below 500 lines and split before 1,000. No numbered fragments or wildcard
parent imports. A cohesive test corpus may exceed the soft aim, but production
code and fixture ownership remain separate.

Bazel excludes `src/tests/**/*.rs` from the production library and includes
them explicitly in its Rust test target. Module splitting alone does not create
separate compilation actions. Extract a crate only at an acyclic, stable
dependency boundary with demonstrated cache benefit.

The old `generator.rs`, raw runtime files and fragment adapter are migration
debt. They remain the explicitly legacy route until atomic cutover; new code
cannot call them as a fallback, parse their output or wrap it in a certificate.
No new empty directories or nominal modules count as an implemented layer.
