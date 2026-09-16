# M35-03A-02F-02B-05A — Publish certified owned constants in bundles

- Status: complete
- Parent: [multi-crate constants](M35-03A-02F-02B-05-constant-bundles.md)
- Depends on: M35-03A-02F-02B-04

## Contract

Extend existing C and Java bundle projection/publication to retain each owner's
certified scalar constants. This checkpoint publishes constants-only and mixed
owners, including separately declared unused dependency owners. It does not yet
admit Rust body reads or re-exports of foreign module constants.

C manifest schema 4 adds owned constant descriptions to the existing bundle
function/import records; schema 2 remains byte-compatible for function-only
owners, standalone schemas 1/3 remain unchanged. The bundle index stays schema 1
because its member-reference structure does not change. Check external constant
symbols together with functions during whole-bundle collision preflight.

Java owner schema 2 adds constant declaration descriptions with exact primitive
type/value and readonly semantics; schema 1 remains unchanged for owners without
constants. Serialize bool as JSON bool and signed values as canonical decimal
strings. Reconcile descriptions with JavaDependencyConstant identities, paths,
types and values; retain exact certificate authority. Reserve all metadata bytes
through the existing shared counting/encoding traversal before rendering.

## Definition of done and tests

- Constants-only and mixed Rust source owners publish complete runtime-free C
  and Java bundles; finite aliases/docs and unused owners remain present.
- Separate native consumers read exact bool/i32/i64 values, including signed
  boundaries and values beyond double precision. Compare with independent Rust
  execution/oracles; C uses GCC and Zig at O0/O2, Java uses strict Java 21.
- Metadata JSON is parsed and values/types/paths/exports are checked against
  actual certificate/compiler probes, not guessed target symbol spellings.
- Mutated owner descriptions, missing/duplicate declarations, incomplete owner
  graphs, external symbol collisions and undersized reservations reject before
  publication. Existing output preservation and source-only inventories hold.
- Foreign public constant body reads/re-exports continue to reject until 05B/05C.
  Existing standalone and function-bundle tests remain enabled.
- Focused/unit/native tests, full Linux Bazel/release/lint gate, independent review,
  inspectable ignored generated examples and scoped commit/push pass.

## Implementation locations

- C: api_manifest serialization, output bundle preflight and focused probes.
- Java: java_bundle projection, constant serialization helper, exact owner checks,
  dedicated fixtures/tests; no changes to generic rendering or unchecked inputs.
- Separate Bazel targets for bundle metadata contracts and native integration.

## Implementation and verification

C bundle owner schema 4 and Java owner schema 2 now carry complete owned scalar
constant descriptions. Old function-only schemas and standalone constant schemas
are unchanged. C preflight includes constant and function definitions in one
external-symbol set. Java projection reconciles each constant description with its
original certificate witness; both sink passes serialize exact signed decimal
strings and JSON bools with bounded reservation.

The two-owner compiler fixture combines a constants-only dependency and a mixed
root. The dependency is declared but unused by Rust body code, proving complete
owner retention without pretending foreign constant reads work. Native tests
compile/link both owners into each consumer environment and check all exact
values, aliases, docs, public readonlyness and the absence of runtime files.
Compilable value mutants fail independent Rust/numeric truth; javac consumers are
recompiled for each producer variant.

Unit tests cover lossless scalar reservation/encoding, undersized buffers,
unsupported literal kinds, missing/duplicate/replaced source descriptions,
changed type/value witnesses, owner inventories, deterministic owner order and
old/new schema coexistence. A compiler probe reconstructs C manifests with the
complete function/constant compiler inventory. Whole-bundle mutation probes
reject missing roots, wrong owner keys, swapped manifests and injected duplicate
external constant symbols.

The atomic publication target checks complete source-only file inventories,
record-order determinism, empty/existing directory/file/symlink preservation,
missing dependency owners and collision rejection. The existing 80 standalone
constant admission/atomic tests remain enabled, including foreign read rejection.

Initial focused runs exposed a test-only nonexistent Java literal variant and an
old C import probe's function-only reconstruction. Both were corrected without
weakening or disabling tests. The initial native bundle proof already passed;
the full corrected tree subsequently passed all gates below.

- Exact candidate: 700a1cafbaf8e99e0ef974434f5276c5e24be68b; 2,618 archived
  Git blobs and executable modes verified before the Linux test run.
- Command: `bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test
  //... //:release_gate --noshow_progress --noverbose_failures
  --test_output=errors --test_summary=terse --keep_going`.
- Result: 1,047 targets, 718/718 tests passed; 38 executed, 680 cached;
  173.311 seconds. Invocation: 25e1bed5-4d5d-4ced-9a3a-f6b12da5d38a.
- Actual C/Java source bundles were exported and inspected under ignored host
  generated/examples/owned-constant-bundles-700a1ca: seven C files and five Java
  files, two crate boundaries, exact versioned values and preserved docs.
- First independent Sol Extra High review rechecked the import-probe fix and
  reported no remaining actionable core error. A fresh independent Sol Extra High
  reviewer audited all 23 staged files, the original authority chains and actual
  exported examples in tree 700a1ca and reported no correctness, regression or
  required-proof defects. Completion documentation is included in the final
  isolated Linux gate before the scoped commit/push.
- Twenty pre-existing unrelated file hashes remain unchanged; no generated output
  or unrelated C ownership edits are staged.

This checkpoint does not enable source-level foreign public constant reads or
foreign re-exports and does not complete child05 or overall runtime retirement.
