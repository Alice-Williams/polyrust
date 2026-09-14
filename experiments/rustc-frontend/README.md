# Rust compiler frontend proof

This isolated experiment accepts real Rust source through rustc's parser, type
checker and borrow checker. After successful analysis it reads the compiler's
HIR and type-check results, preserving structured branches in generated C17.
No custom Rust parser, textual compiler-dump parser or borrow checker exists.

The first fixture moves a non-Copy Ticket, borrows it and returns a conditional
result. A second fixture changes fields, constants and branch structure; a
third checks nested lexical blocks. A fourth exercises the complete eight-owner
mapping inventory, with compiler-backed assertions over the resulting C AST.
A fifth preserves ordered module/type/field/function doc attributes, including
declared included text and hostile C comment contents. Its compiler probe checks
typed owner placement and three-render determinism. Separate regressions exercise
shared metadata for a 256-field record, extraction/resource limits and declared
compiler file/environment dependencies.
A sixth fixture retains a finite compiler-resolved public export graph: aliases,
type/value/macro namespaces, cyclic module aliases, dependency identities and a
declared path-selected module. Its tests also check Rust privacy rejection and
unchanged output on errors. Retained exports are metadata, not a claim that the
current single-unit C output exposes every Rust API item.
Boundary tests reject foreign ABIs and
unstable input features, including when RUSTC_BOOTSTRAP is set by the caller.
The same 8,204 inputs are run against each native Rust program and C at O0/O2.
Invalid moves, escaping/conflicting borrows and type errors must produce Rust
diagnostics and no C artifact. Valid unsupported Rust gets an admission error.

## Run in the Linux development container

```sh
bazelisk test //experiments/rustc-frontend:proof_test \
  //experiments/rustc-frontend:alternate_proof_test \
  //experiments/rustc-frontend:scopes_proof_test \
  //experiments/rustc-frontend:boundary_test \
  //experiments/rustc-frontend:provenance_test \
  //experiments/rustc-frontend:rustfmt_test \
  //experiments/rustc-frontend:reference_clippy_test
bazelisk test //experiments/rustc-frontend:capability_contract_test \
  //experiments/rustc-frontend:mapping_inventory_ast_test \
  //experiments/rustc-frontend:mapping_inventory_proof_test \
  //experiments/rustc-frontend:model_native_matrix_test \
  //experiments/rustc-frontend:alternate_native_matrix_test \
  //experiments/rustc-frontend:scopes_native_matrix_test \
  //experiments/rustc-frontend:mapping_inventory_native_matrix_test
bazelisk build //experiments/rustc-frontend:generate
bazelisk test //experiments/rustc-frontend:documentation_test
bazelisk test //experiments/rustc-frontend:export_graph_test
```

Bazel output is `bazel-bin/experiments/rustc-frontend/generated/model.c`.
For a visible, ignored copy in the bind-mounted workspace:

```sh
mkdir -p /workspace/experiments/rustc-frontend/output
bazelisk run //experiments/rustc-frontend:adapter -- \
  /workspace/experiments/rustc-frontend/fixtures/model.rs \
  /workspace/experiments/rustc-frontend/output/model.c
```

The adapter entry is named `score`, with exact signature `fn(i32) -> i32`.
C exports `int32_t poly_score(int32_t)`. This deliberately narrow harness is
not a production language plugin. Generated artifacts are not committed.

## Public API package example

Package mode maps the complete admitted public function graph rather than one
selected entry. Aliases share one implementation; private helpers and concrete
record layouts remain in the implementation. Its first fixture has four public
functions, four private helpers, and an alias-only public module with doc attributes.

```sh
bazelisk test //experiments/rustc-frontend:public_package_smoke_test \
  //experiments/rustc-frontend:public_package_manifest_test \
  //experiments/rustc-frontend:public_package_negative_test
bazelisk build //experiments/rustc-frontend:generate_public_package
```

The declared tree artifact is
`bazel-bin/experiments/rustc-frontend/generate_public_package.package/`.
A tested, byte-identical ignored copy is available in
`experiments/rustc-frontend/output/crate-identity/default/`: a crate-identity-named
public header and implementation, plus `api.json`. The manifest retains the
finite module/alias graph and exact linker symbols; it is metadata, not a
deserializable compiler or rendering certificate.

The package oracle compares 8,204 native Rust inputs against separately compiled
C implementations/consumers in eight GCC/Zig/ASan/UBSan O0/O2 configurations.
Each runs three header include orders, duplicate inclusion and a 1 MiB stack
limit. Exact public symbols and rejection of private helper calls are checked.

For a new destination, use `adapter INPUT.rs OUTPUT_DIRECTORY --package`, with
any declared `--input PATH` pairs last. Existing destinations are rejected.
Current package APIs are nongeneric ordinary Rust functions over i32/bool;
unsupported public types, macros, foreign APIs and other unmapped categories
diagnose instead of disappearing from the generated interface.

### Explicit crate identity

```sh
bazelisk test //experiments/rustc-frontend:compiler_configuration_test \
  //experiments/rustc-frontend:crate_identity_test
bazelisk run //experiments/rustc-frontend:adapter -- \
  /workspace/experiments/rustc-frontend/fixtures/crate_identity.rs \
  /workspace/experiments/rustc-frontend/output/my-new-package \
  --package --crate-name identity_fixture --crate-key polyrust.my-package
```

Supply name and key together. The key is explicit stable build identity passed
to rustc's metadata disambiguation; it is not inferred from an absolute path.
Both compiler crate and declaration identities enter each package symbol name.
This changes experimental package symbol spelling; the selected `poly_score`
ABI is unchanged. This standalone invocation has no foreign dependency graph;
use the checked bundle operation below for admitted cross-crate calls.

The tested declared `generate_identity_a` and `generate_identity_b` artifacts
have visible ignored copies under `output/crate-identity/a` and `b`. The older
`output/public-package` copy is retained as historical M35-01D-03C evidence;
use `output/crate-identity/default` for current default package output.

Additional compiler inputs must be passed explicitly as `--input PATH` after
the source and output arguments, and declared in the Bazel action. For example,
the documentation fixture requires `--input fixtures/documentation.md` when
running from this experiment directory. Readable but undeclared include files,
out-of-line modules and environment-dependent attributes are rejected before
writing output. This is an input-dependency check, not a compiler OS sandbox.

## Checked crate bundles

The declared-source driver now checks every reachable crate against its loaded
metadata, binds foreign calls to exact certified C APIs, and emits one owning
header/source pair per crate. Imports are separate from owned definitions in
version-two member manifests. Unsupported foreign signatures and public foreign
re-exports diagnose; this is not a general translator for arbitrary Rust crates.

```sh
bazelisk test //experiments/rustc-frontend:c_graph_test \
  //experiments/rustc-frontend:c_bundle_action_test \
  //experiments/rustc-frontend:directory_publication_test
bazelisk build //experiments/rustc-frontend:generated_root_bundle
```

The declared artifact is
bazel-bin/experiments/rustc-frontend/generated_root_bundle.bundle.
An ignored copy of its actual ten generated files is available in
output/crate-bundle/m35-04c. bundle.json identifies the root and member manifests;
the member files retain crate-qualified identities, public/private linkage and
typed dependency-derived includes. No source or metadata artifact is copied
into a consumer implementation. A manifest is descriptive, not proof input.

The low-level operation is adapter --bundle OUTPUT followed by the closed
--root/--crate/--input/--dependency records documented in the
[driver specification](../../docs/specification/typed-generation/languages/c/rust-hir-dependency-driver.md),
or one response file. --check-crates uses the same graph without publication.
Publication refuses existing destinations atomically, including racing empty
directories. The Linux launcher supplies the first-party no-replace helper;
source inputs cannot select it.

### Four-crate native proof (M35-01D-04D)

The leaf/left/right/root source diamond is also built as four independent native
Rust libraries. Its generated C implementations are separately compiled and
linked, with 8,204 inputs and sixteen results per input compared against Rust in
eight GCC/Zig/ASan/UBSan O0/O2 profiles and three header include orders. A nested
call pair is deliberately noncommutative and nonconstant, and both chosen
branches are observable. Tests reconcile all external symbols with public
bindings and all owned public/private functions at O0, check exact fixture
import signatures, reject both private helpers and the private record through
public headers, and check every fixture doc comment's exact placement.

```sh
bazelisk test //experiments/rustc-frontend:crate_native_test \
  //experiments/rustc-frontend:crate_proof_clippy_test
bazelisk build //experiments/rustc-frontend:generated_crate_proof
```

The declared artifact is
`bazel-bin/experiments/rustc-frontend/generated_crate_proof.bundle`.
The ignored workspace example is `output/crate-bundle/m35-04d`, containing
`bundle.json` plus one header, implementation and member manifest per crate
(thirteen files). From the repository root inside the container, produce a
fresh example and verify its bytes as follows; choose a new destination if it
already exists:

```sh
artifact=bazel-bin/experiments/rustc-frontend/generated_crate_proof.bundle
example=experiments/rustc-frontend/output/crate-bundle/m35-04d-new
test ! -e "$example"
mkdir "$example"
cp "$artifact/"* "$example/"
diff -rq "$artifact" "$example"
```

These are proofs for the admitted no-heap subset, not support for arbitrary
Rust crates. The task record retains the gate IDs and independent review results.

## Java production bundles

The same real four-crate Rust source graph now generates a production Java
bundle through the existing typed Java AST, certifier and structural renderer.
Each owner has its own canonical crate namespace and `Generated.java`; exact
dependency certificates produce qualified calls, not copied foreign bodies.
One descriptive API JSON file per owner and `bundle.json` retain source aliases,
visibility, docs, private declarations and direct used-owner edges. JSON never
reconstructs compilation or callable authority.

From the repository root in the Linux development container:

```sh
bazelisk build //experiments/rustc-frontend:generated_java_crate_proof
bazelisk test //experiments/rustc-frontend:java_graph_native_test \
  //experiments/rustc-frontend:java_bundle_publication_test \
  //experiments/rustc-frontend:java_graph_check_test
```

The declared artifact is
`bazel-bin/experiments/rustc-frontend/generated_java_crate_proof.bundle`.
The host-visible ignored copy is `generated/m35-java-production-preview/`, with
an explanation in `generated/m35-java-production-preview.md`. Its nine payload
files were compared byte-for-byte with the actual Bazel artifact. To create
another copy, choose a fresh destination (the command does not overwrite it):

```sh
artifact=bazel-bin/experiments/rustc-frontend/generated_java_crate_proof.bundle
example=generated/m35-java-production-preview-new
test ! -e "$example"
cp -R "$artifact" "$example"
diff -rq "$artifact" "$example"
```

The production CLI is `java_graph_adapter --bundle DESTINATION` followed by the
same `--root`/`--crate`/`--dependency` graph records as `--check-crates`; one
bounded `@response-file` is also accepted. The `rust_java_bundle` Bazel rule
supplies these records from the declared `RustSourceCrateInfo` provider.
Destination parents must already exist. Only a new destination is accepted;
all owners are checked, bounded and rendered before one atomic no-replace
publication. Check mode validates but never renders or publishes target files.

The oracle compiles each owner separately with pinned Java 21
`--release 21 -encoding UTF-8 -Xlint:all -Werror -implicit:none -sourcepath ""`,
then checks 131,264 results against native Rust and rejects private consumers.
Record reordering and physical input relocation preserve every source/JSON byte.
Filesystem tests cover late/existing targets, failed members, unused declared
owners, helper failures and a race with one complete winner. The supported
subset remains explicitly limited as described below; this is not arbitrary
Rust-to-Java translation.

## Toolchain and admitted shape

The [fixture matrix](../../docs/specification/typed-generation/languages/java/rust-hir-fixtures.md) lists every admitted Java generation
target, native reference, full-corpus test and host-visible example location,
and distinguishes metadata-only/unsupported fixtures.

Rust 1.98.0 and its checksum-pinned rustc-dev component come from Bazel.
The adapter's build alone sets `RUSTC_BOOTSTRAP=polyrust_rustc_frontend` to
access unstable compiler APIs on that exact pinned release; input compilation
does not receive this exemption. Clippy compiles the adapter with warnings
denied. The dedicated Bazel Rustfmt toolchain checks all Rust sources.
Generated C uses the existing hermetic C toolchain.

Admission covers i32/bool, nonempty scalar-field structs without custom Drop,
shared references, scalar constants, moves/copies, aggregate construction,
comparisons and boolean branches. Functions have plain immutable parameters,
initialized plain let bindings and a tail result; tail if/else and nested
blocks are supported. Other HIR forms, overloaded operations and unimplemented
compiler adjustments reject. Built-in shared-reference dereferencing uses
the compiler's adjustment metadata. C has structured if/else, not goto labels.
Representation identities come from compiler declarations, not name strings.
C includes are derived from admitted types.

Custom record representations (including packed/aligned forms) reject before
C registration. Alias and associated-type paths in the crate's HIR type
surfaces and constructor paths reject until alias-use provenance is mapped;
compiler normalization
must not silently erase that obligation. An unused alias declaration alone
does not constitute an alias use.

The compiler-backed provenance probe inspects the real typed C tree: two
same-named records in different modules, field/initializer ownership,
declared versus effective visibility, declaration identities, source lines
and resolved docs. It is separate from the registry tests using manual metadata.
It also checks the actual tree's type-derived C header and nominal-definition
requirements against the retained frozen registry, including repeatability.

During M35-01B, every admitted entry lowers into the existing backend-c
registry and AST and passes its context, constant/layout, sequencing, numeric,
index and storage checks. Compiler declaration identity, source location,
module, declared/effective visibility and resolved documentation accompany its
registrations. Shared projection, verification, linking, post-link checking and
resource admission then produce RenderReadyPackage<CDialect>, consumed by
CStructuralRenderer. No miniature model or alternate renderer remains. Typed
platform assertions derive their own standard-header requirements and reject
incompatible C layouts at native compilation. Eight executable capability-owned
mapping slots use a consuming builder and compiler-negative registration tests;
source inputs do not contain target AST types. The native matrix executes each
certified artifact with pinned GCC/Zig and GCC ASan/UBSan at O0/O2. Single-unit
documentation placement and one-crate public-header/source packages are
implemented. Separate dependency-crate linking has a native differential gate;
Java now has a corresponding compiler-backed production bundle path; final
C/Java no-heap migration and isolated commit-tree proof are complete. The
separate C and Java checkpoints are committed and pushed; E05 records their
exact tree/commit IDs and Linux/Bazel proof. Heap/drop support remains M35-02.

HIR is rustc's higher-level AST after macro expansion and name resolution;
it does not alone certify typing or borrowing. The adapter requires full
compiler analysis to succeed before reading it. Some Rust forms, including
for loops, are already desugared in HIR and need explicit future mappings.

There is no heap allocation, custom destructor, standard-library mapping,
unsafe, FFI or unwind support. This proof does not establish heap cleanup,
universal safety, full Rust support or production C compliance. See
[the experiment specification](../../docs/specification/rustc-frontend-proof.md)
and [M35](../../docs/plan/milestones/M35-rustc-frontend-proof.md).

M35-02A adds a backend-independent `owned_probe` executable, observing pinned
rustc drop-elaborated MIR after successful analysis. Its `owned_probe_test` also
depends on C/Java adapters for rejection controls. It tests moves, conditional and
partial drops, early returns, shadowing and a same-spelled counterfeit Box.
It does not enable heap translation. See the [typed observation evidence](../../docs/specification/typed-generation/languages/c/rust-drop-observations.md)
and [ordered owned-value plan](../../docs/plan/tasks/M35-02-rustc-owned-values.md).

M35-02B-01 adds `owned_construction_test` and its separately cached compiler-
negative contracts. It authenticates individual `Box<i32>` constructor inputs
through compiler items and invokes a typed registered mapping in a compiler-only
proof consumer. This does not enable C/Java heap output or establish HIR/MIR
variable correspondence. See [constructor capability specification](../../docs/specification/typed-generation/languages/c/rust-owned-construction.md).

M35-02B-02 adds `owned_linear_test`, `owned_linear_private_test` and a dedicated
format target. They establish a closed correspondence for one Box<i32>, whole-
value moves and a final scalar read/drop, retaining canonical HIR bindings and
authenticated MIR places/locations. Private mutations reject incorrect producer,
move, read, drop and control-flow relations. This remains compiler-only evidence,
not C/Java heap admission. See [linear correspondence specification](../../docs/specification/typed-generation/languages/c/rust-linear-owned-correspondence.md).

M35-02B-03A adds the separately selected tail-scope reader and `owned_scope_*`
proof targets. Canonical HIR containment certifies block parents and binding,
read and owner/drop scopes, including empty wrappers and ownership moved inward.
The one-owner MIR relation is reused; the root-only entry remains restricted.
This does not admit sibling exits, branches or multiple owners and does not
enable heap generation. See [tail-scope specification](../../docs/specification/typed-generation/languages/c/rust-owned-tail-scopes.md).
