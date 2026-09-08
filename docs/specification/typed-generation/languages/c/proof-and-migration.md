# C17 proof and migration protocol

- Status: normative for M34A-11

The [exact platform/command/target inventory](platform-and-proof.md) specifies
the admitted build environment, isolated negative shape, no-vacuity controls
and historical membership. Planned target names are not passing evidence.

## Gates

1. Rust compile-fail: wrong AST category, invalid fixed signature, missing/
   duplicate/wrong-dialect mapping, crossed brands and every proof-wrapper bypass.
2. C AST tests: every constructor and contextual rule has positive and negative
   cases. Mutation suites cover prototypes, declarators, qualifiers, references,
   linkage, namespaces, initialization, lifetimes, cleanup and plan authenticity.
3. Certified compiler oracle: deterministic structured mutations span each
   category; every admitted package renders without repair and compiles under
   the pinned C17 toolchain. Negative fixtures must actually fail compilation.
4. Native semantics: separate public-header consumers, all portable capabilities,
   Unicode/UTF-8/embedded-zero data, arithmetic/shift/F64 edge matrices,
   interface/empty-interface/composition and lifecycle fixtures.
5. Fault injection: fail each allocation in nested construction/clone/dispatch
   and every early exit; track outstanding allocations and initialized prefixes.
   Pair these with GCC 14.2.0 ASan (including leaks) and UBSan at multiple
   optimization levels. The fault allocator itself is tested for correctness.
6. Determinism and integration: three-generation AST/manifest equality,
   evaluator and all eight native outputs, every historical C port, source
   policy, Rustfmt, Clippy, Buildifier, docs, the full tracked Bazel rule graph
   and release suite, then hosted CI.

Use the Linux development container and existing pinned Zig C17/GCC toolchains.
Required strict diagnostics include C17, Wall, Wextra, Wpedantic, Werror,
strict prototypes and missing prototypes for generated public definitions.
Any compiler-specific flag is tested with its pinned compiler; no silent
diagnostic disable is allowed merely to get a fixture through.

Native compilation proves type/grammar acceptance, not all memory safety.
Sanitizers and finite fuzzing supplement the typed checker, not replace it or
establish a universal theorem. Each accepted review issue needs a reproducible
counterexample and regression; reject false positives with explicit evidence.

## Checkpoint policy

Implement in the task order under M34A-11. Keep the legacy route explicit until
the new pipeline can replace it atomically, and never certify legacy fragments.
Each completed slice records exact test targets/results and is committed and
pushed separately. Normal Bazel action/test caching remains enabled.
Do not run cargo/bazel/native compilers on Windows or pass credentials to the
container. Leave the unrelated untracked stdlib-abs work untouched.

After a fully integrated immutable push, request a fresh uncapped Sol Extra High
read-only review, evaluate every finding, repair core errors and repeat. Optional
new syntax/features are not correctness blockers. Wait for that exact commit's
CI at coarse five-minute intervals. Mark C Pass only after all required evidence,
legacy deletion and a clean fresh review; then tell the user it is ready for
their review before proceeding to another language.

## Standards and oracle sources

- [WG14 public C11 draft N1570](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf)
  supplies background clauses for namespaces, declarators, expressions and
  library contracts. It is not mislabelled as the C17 standard.
- [Clang C status](https://clang.llvm.org/c_status.html) identifies C17 mode.
- [Clang compiler manual](https://clang.llvm.org/docs/UsersManual.html) documents
  standard selection and pedantic diagnostics; the repository pins the actual SDK.
- [GCC 14.2 instrumentation](https://gcc.gnu.org/onlinedocs/gcc-14.2.0/gcc/Instrumentation-Options.html)
  documents the sanitizer oracle, not a replacement for the ownership verifier.
