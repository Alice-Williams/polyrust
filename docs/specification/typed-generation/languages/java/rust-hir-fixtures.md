# Java admitted Rust-source fixture matrix

All target names below are relative to `//experiments/rustc-frontend:`. The
authoritative tests run in the Linux DevContainer. Each corpus contains the
eleven checked-in boundary seeds plus every integer from -4096 to 4096: 8,204
input rows, including deliberate duplicate seed values.

## Entry fixtures

| Rust source under `fixtures/` | Java generation target | Native Rust reference | Differential test | Inputs x outputs | Structural/metadata evidence |
| --- | --- | --- | --- | --- | --- |
| `model.rs` | `generate_java_model` | `rust_reference` | `java_source_model_test` | 8,204 x 1 | A: package, scopes, concrete records |
| `alternate.rs` | `generate_java_alternate` | `rust_alternate` | `java_source_alternate_test` | 8,204 x 1 | A: package, scopes |
| `scopes.rs` | `generate_java_scopes` | `rust_scopes` | `java_source_scopes_test` | 8,204 x 1 | A: package, lexical scope bindings |
| `mapping_inventory.rs` | `generate_java_mapping_inventory` | `rust_mapping_inventory` | `java_source_mapping_inventory_test` | 8,204 x 1 | A: borrow depth, field order |
| `documentation.rs` | `generate_java_documentation` | `rust_documentation` | `java_source_documentation_test` | 8,204 x 1 | A: resolved source/doc provenance |
| `direct_calls.rs` | `generate_java_direct_calls` | `rust_direct_calls` | `java_source_direct_calls_test` | 8,204 x 1 | A: field/call/comparison evaluation order |
| `boolean_order.rs` | `generate_java_boolean_order` | `rust_boolean_order` | `java_source_boolean_order_test` | 8,204 x 1 | A: six typed operators; native test: 24 truth cases |

A denotes `java_source_ast_test`, which runs each listed source through its
compiler-backed probe and requires byte equality with the production adapter.
These entry examples expose the selected entry operation rather than a public
package export facade; facade privacy is covered by the package rows below.

Each generation artifact is `generated/java/<fixture>/Generated.java` under
the package's Bazel output directory. Every test compares that exact artifact
with two independent adapter invocations, compiles Java 21 with warnings as
errors, and checks all 8,204 rows against native Rust and generated C at O0/O2.
The documentation and direct-call targets explicitly declare their included
Markdown and out-of-line Rust inputs. Boolean ordering additionally checks all
six predicates on all four boolean operand pairs against independent truth.

`java_source_ast_test` retains the compiler-backed typed-tree assertions for
scopes, documentation, field initialization order, shared borrow depth, call
argument order, comparison order and distinct source identities. The existing
C native matrices retain GCC/Zig and ASan/UBSan coverage; compiler-negative and
typed capability/registry tests remain enabled.

## Public packages and crate graph

| Rust source | Java bundle target | Native reference | Native test | Inputs x outputs | Private/metadata evidence |
| --- | --- | --- | --- | --- | --- |
| `fixtures/public_package.rs` | `generated_java_public_package_bundle` | `rust_public_package` | `java_fixture_public_package_test` | 8,204 x 8 | A: constructor/helper/record privacy; native test: complete export-map equality, aliases and module cycle |
| `fixtures/java_same_spelling.rs` | `generated_java_same_spelling_bundle` | `rust_same_spelling` | `java_fixture_same_spelling_test` | 8,204 x 2 | A: distinct typed callables; native test: complete export-map equality, distinct IDs/targets; no private source declarations |
| `fixtures/crate_proof_{leaf,left,right,root}.rs` | `generated_java_crate_proof` | `rust_crate_proof` | `java_graph_native_test` | 8,204 x 16 | Native test: complete independent private/public manifest oracle, mutation rejection, all private functions/records denied |

The first two tests consume actual single-owner Java/C bundles and compare
8,204 rows against independently compiled Rust, Java 21, GCC 14.2 O0/O2 and
Zig O0/O2. Their expected mathematical results are checked independently too.
They compare the complete C/Java source export-binding maps and function IDs;
explicit assertions cover the public alias chain, self-referential module
alias and two same-spelled functions with different source/target identities.
Independent exact fixture binding-key lists also reject a common-mode extra
alias; mutation checks inject one into every exported module to prove rejection.
C bundle targets are `generated_c_public_package_bundle` and
`generated_c_same_spelling_bundle`. Existing `public_package_smoke_test` retains
full-corpus C sanitizer, include-order and private-access checks.

`java_source_ast_test` separately proves that the public-package constructor,
private helper and private record reject Java consumers, with direct compiler
observations of public and private callable identities. It also checks every
boolean branch combination. The diamond test compares the complete emitted
public/private metadata inventory against independent compiler/typed-certificate
observations, mutation-tests that oracle, and rejects consumers of every private
function and record. It compiles four owners separately and checks 131,264
results, exact Bazel/CLI/probe bytes, reordered records and relocated inputs.
`java_bundle_publication_test` retains atomic publication/failure/race tests.

## Deliberately excluded fixtures

`export_graph.rs` is a compiler metadata probe, not an admitted generated Java
program. It includes public record exports, a foreign standard-library re-export
and an exported macro outside this closed public-package subset. Negative and
metadata-only fixtures remain valuable tests but are not presented as translated
examples. This matrix is bounded execution evidence, not a universal equivalence
theorem or support for arbitrary Rust programs.

## Inspectable examples

The ignored host directory `generated/m35-java-fixtures-preview/` contains the
seven entry outputs under `entries/`, plus `public_package/` and `same_spelling/`
production bundle directories. Each copy is verified byte-for-byte against its
declared Bazel artifact. The four-crate production example remains in
`generated/m35-java-production-preview/`. Generated output is never committed.

From `/workspace` in the Linux container, these commands build, copy and verify
every listed Java example. `mktemp` selects a fresh ignored destination and the
existing previews are not overwritten. No test-only probe payloads are copied.

```sh
set -eu
set --
for fixture in model alternate scopes mapping_inventory documentation direct_calls boolean_order; do
  set -- "$@" "//experiments/rustc-frontend:generate_java_$fixture"
done
bazelisk build "$@" \
  //experiments/rustc-frontend:generated_java_public_package_bundle \
  //experiments/rustc-frontend:generated_java_same_spelling_bundle \
  //experiments/rustc-frontend:generated_java_crate_proof
mkdir -p generated
example=$(mktemp -d generated/m35-java-examples.XXXXXX)
artifact=bazel-bin/experiments/rustc-frontend
for fixture in model alternate scopes mapping_inventory documentation direct_calls boolean_order; do
  mkdir -p "$example/entries/$fixture"
  cp "$artifact/generated/java/$fixture/Generated.java" "$example/entries/$fixture/Generated.java"
  cmp "$artifact/generated/java/$fixture/Generated.java" "$example/entries/$fixture/Generated.java"
done
for fixture in public_package same_spelling; do
  cp -R "$artifact/generated_java_${fixture}_bundle.bundle" "$example/$fixture"
  diff -rq "$artifact/generated_java_${fixture}_bundle.bundle" "$example/$fixture"
done
cp -R "$artifact/generated_java_crate_proof.bundle" "$example/diamond"
diff -rq "$artifact/generated_java_crate_proof.bundle" "$example/diamond"
printf 'Verified examples: %s\n' "$example"
bazelisk test //experiments/rustc-frontend:all //:release_gate
```

The full migration gate `6445dc5b-0311-4696-961a-12529ee432ef` passed all 373
tests across 487 targets. The seven entry files and both three-file fixture
bundles were individually byte-verified against the actual artifacts; the
separate diamond preview contains nine previously byte-verified payloads.
This is working-tree evidence; isolated commit-tree proof is a separate gate.
