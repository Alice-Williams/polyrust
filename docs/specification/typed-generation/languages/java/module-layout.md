# Java module ownership

- Status: normative for M34A-10Z and subsequent Java work

The root files are entry points, explicit public re-exports, or orchestration.
They are not places to accumulate unrelated helpers and fixtures.

| Module | Responsibility | Must not own |
| --- | --- | --- |
| `capabilities/` | One executable mapping per portable capability, with typed inputs and results | An independent second implementation of the same mapping |
| `preflight/` | Dynamic admission and support certificates | Rendering or portable semantic approximations |
| `lower/` | Walk verified CoreIR, allocate symbols, assemble mapping inputs and evaluation plans | Raw Java text or unchecked input admission |
| `ast/` | Java node models, structural checks, member identity, lexical flow, and conformance | CoreIR lowering or text generation |
| `dialect/` | Closed Java catalogues, invocation signatures, target registration and linker integration | Portable program interpretation |
| `runtime/` | Typed runtime AST construction by runtime family | Static import strings or rendering |
| `resources/` | Post-certification target capacities, descriptor accounting and compiler-admission budgets | Syntax admission, rendering or portable arity caps |
| `render/` | Structural printing of certified declarations, statements, expressions, names and syntax | New checks, lowering decisions or runtime semantics |
| `tests/` | Focused fixtures, mutations and compiler-oracle tests | Production library sources |

Within the runtime, declaration builders own metadata, member builders own
method/field construction, expression builders own expressions, and statement
builders own statements. Equality-bearing record assembly belongs to equality,
not to a nominally generic declaration utility.

Within the AST, model modules are separate from checks. Lexical and constructor
flow state is private to the AST subsystem; extraction must not make it public.
Private child modules explicitly re-export only the intended existing API.
Imports name actual dependencies rather than sharing a parent wildcard scope.
Helpers stay file-local unless a sibling implementation or focused test needs
them; sibling access does not justify public crate API access.

Production files aim below 500 lines and must remain below 1,000. A cohesive
check may slightly exceed the soft aim; numbered chunks or arbitrary splits
are not a substitute for responsibility boundaries.

## Build and policy boundaries

Bazel excludes `src/tests/**/*.rs` from the production Rust library and includes
those files in the Rust test target. Rendering policy recursively covers every
renderer child; typed-source policy recursively covers production modules.
Discovery failures, including partial results from `find`, fail closed.

Rust modules in a single crate share a compilation action. This layout improves
navigation and keeps fixture-only changes out of the production source set;
it does not promise per-module compiled artifact caching. A future crate split
must demonstrate an acyclic dependency boundary and useful independent Bazel
actions before it is adopted.
