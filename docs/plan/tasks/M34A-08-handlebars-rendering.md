# M34A-08 — Add resolved-only strict Handlebars rendering

- Status: planned
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
