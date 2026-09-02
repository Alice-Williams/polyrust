# M34A-08 — Add resolved-only strict Handlebars rendering

- Status: complete
- Depends on: M34A-07

## Goal

Use per-language templates solely to spell already resolved grammar.

## Definition of done

- A pinned and audited `handlebars-rust` dependency is declared through Bazel
  and documented.
- Renderer traits accept only resolved packages/files and convert them to
  private typed render views.
- Template selection uses dialect-owned closed `TemplateId` enums with
  exhaustive registration and no wildcard arms.
- Registries enable strict mode, embed/pin every certified template, reject
  missing/duplicate templates and fields, and expose no scripting or semantic
  helper.
- Templates contain reusable grammar skeletons only; feature/runtime-helper
  implementations and portable-operation branches are rejected by policy.
- Precedence, parentheses, identifier/literal escaping, names, dependencies,
  helpers, and file placement are resolved before template invocation.
- Renderer modules cannot import CoreIR/capability APIs or inspect unresolved
  symbols.
- Deterministic newline/encoding/formatting policy is enforced.

## Tests

- `bazel test //crates/codegen:handlebars_renderer_test --nocache_test_results --test_output=errors`
- `bazel test //crates/codegen:renderer_compile_fail_test --nocache_test_results --test_output=errors`
- `bazel test //tools/policy:template_policy_test --nocache_test_results --test_output=errors`
- Missing/extra fields, duplicate/missing registration, forbidden helper,
  precedence, escaping, newline, and three-render determinism fixtures.

## Commit gate

Commit and push `M34A-08: add strict resolved rendering` only after dependency
licence checks, focused tests, Buildifier, Rustfmt, and Clippy pass in the dev
container.

## Exit evidence

- `handlebars` 6.4.4 is pinned exactly through Cargo and Bazel with default
  features disabled. Its complete normal dependency closure was audited as
  permissively licensed; the release allowlist now keys every package by both
  crate name and version so parallel versions cannot be concealed.
- `LinkerDialect` performs an explicit unresolved-to-resolved item/module
  rewrite. The linker stores those resolved values, rederives them during
  verification, and exposes no production accessor for its unresolved package,
  symbol-reference table, helper IDs, forward-declaration IDs, or whole-package
  debug representation.
- `ResolvedTemplateRenderer` accepts only verified `LinkedTargetPackage` and
  `LinkedFile` values and constructs a private serializable file view. A
  compile-fail test proves unresolved target AST cannot enter the certified
  renderer API.
- The shared `CertifiedTemplateEngine` enables strict mode, disables development
  mode and HTML escaping, registers no custom helpers, validates exact closed
  template registration and exact top-level view fields, and rejects missing
  partials, arbitrary helpers, invalid fields, triple braces, and non-canonical
  control bytes.
- The template source policy rejects semantic names/branches, static dependency
  directives, and custom helpers. Its Rust scan independently prevents the
  renderer module from importing CoreIR/checker/capability types or naming
  unresolved target packages/symbols.
- Tests cover missing/extra fields, missing/duplicate registration, missing
  partials, forbidden helpers, full precedence pairs and associativity sides,
  identifier/literal/documentation escaping, canonical LF/final-newline policy,
  comment sanitization, and three-render determinism. An integrated linked
  fixture proves runtime and user items pass through the same certified
  declaration template.
- In the Linux development container, all three named milestone gates passed
  uncached. Rustfmt, Clippy, Buildifier, documentation, typed-generation,
  template, release dependency, release fault-injection, and repository source
  policies passed. The complete tracked-scope graph passed uncached, 266 of 266
  tests. The frozen untracked M34-03 `stdlib-abs` package was the only excluded
  Bazel package and was not modified.
