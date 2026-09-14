# M35-01D-04C-02 — Declared crate graph and compiler driver

- Status: complete
- Depends on: [M35-01D-04C-01](M35-01D-04C-01-metadata-agreement.md)
- Parent: [M35-01D-04C](M35-01D-04C-rust-crate-dependency-driver.md)

## Definition of done

Implement in three checkpoints: the independently cached closed configuration
and crate graph; declared metadata actions; then the checked multi-crate driver.
Keep parsing/graph validation free of compiler internals and target backends.
Metadata emission and source checking consume the same configuration types.

The configuration checkpoint uses private-field `CrateDescription`,
`InputMapping` and `CrateGraph` types in the standalone
`portable_rustc_configuration` Bazel library. The graph is root-reachable,
dependency-first and deterministic; aliases are separate from defining keys.
Logical file mappings are explicit. This validates descriptors only: filesystem
canonicalization, compiler authentication and target certificates remain later
checks and cannot be minted by these configuration types.

Introduce a bounded typed crate configuration/graph with exact dependency alias
edges and explicit source, doc and metadata inputs. Reject duplicates, missing
nodes, cycles, unsupported configuration and budget excess before compilation.
Bazel owns each metadata action and supplies a typed provider for its declared
configuration/inputs. Use the same fixed compiler arguments as checked analysis.
Include metadata output mode and explicit stable logical source-path mappings:
04C-01 shows both affect the compiler content hash. Physical sandbox relocation
must not alter the declared logical names; undeclared/out-of-map inputs diagnose.

Reconstruct dependencies in topological order in the current driver process;
retain owned compiler agreement evidence, never TyCtxt references across
invocations. Provide a same-analysis callback for the target certificates added
by 04C-03; this checkpoint does not mint those certificates. Verify each consumer-loaded dependency against
its checked source evidence using the pinned protocol established in 04C-01.
No ambient library discovery or caller-written signature/hash summaries.

## Tests and proof

- Pure graph/configuration tests cover boundaries, aliases, missing/duplicate
  identities, cycles, options and deterministic ordering.
- Real compiler tests reject stale metadata, body/doc/config substitutions and
  undeclared inputs even when public names/signatures still match.
- Independent declared metadata/source/doc changes invalidate Bazel actions.
- Existing single-crate/default and compiler-negative tests remain unchanged.
- Linux Bazel tests, compiler Clippy, formatting, buildifier and docs pass.

## Configuration checkpoint evidence

- Shared configuration extraction: all 48 frontend tests passed in
  `a869c5b6-893c-4322-8a77-e70b36c099a9`.
- Typed descriptor/DAG builders: all seven focused test/lint targets passed in
  `6aeed47d-ae2d-4f9c-9fb0-11b7fa726ff6`, including a real 1024-crate
  chain/one-over, diamonds, aliases, collisions and input/path budgets.
- Closed record parser: all seven focused targets passed in
  `a5038ef6-0f8f-4ec7-b67f-34699f16ad7e`, with 11 public graph/parser
  cases and two internal checked-budget cases. Legacy configuration tests pass.
- See the [driver specification](../../specification/typed-generation/languages/c/rust-hir-dependency-driver.md)
  for exact descriptor guarantees and the remaining filesystem/compiler stages.
- Full integration gate `6ef6140a-e75f-4748-9ffb-22754b905883` passed
  all 315 tests across 357 targets. Fresh broad read-only Sol Extra High review
  found no actionable core errors in the typed graph or closed parser.
- Metadata actions, compiler evidence and C foreign-call integration are not yet
  enabled by this completed configuration-only checkpoint.

## Filesystem resolution checkpoint

`ResolvedInputs` resolves declared regular files, rejects canonical aliases and
invalid remapping delimiters after symlink resolution, and rechecks the resolved
path-byte budget. It derives fixed metadata-mode arguments and one stable
physical-to-logical mapping per declared file. It performs no compilation or
publication and carries no source-content proof. Existing rustc read tracking
remains mandatory; graph-wide metadata/output ownership checks follow.

The first filesystem gate `844d9500-66d2-4604-9db5-12f368b47f82`
passed all seven targets, including four real Linux filesystem tests and
graph/budget/Clippy/format/buildifier/docs checks. The real compiler probe is now
being joined to this resolver to prove per-file metadata hash agreement.

The integrated per-file compiler probe passed in
`25438710-37ea-4f68-9ab8-c787d177f67e`: root/module/doc relocation,
input-order independence, changed contents and undeclared module rejection.
Full gate `6ea364ab-3172-4d71-8cbd-fdfdc43bcd8c` passed all 316 tests
across 358 targets. Fresh read-only Sol Extra High review found no actionable
core errors. Filesystem resolution/remapping is complete at its stated boundary.

## Metadata action checkpoint (complete)

Add an isolated metadata emitter with no backend dependency, using the shared
graph/input configuration and existing post-analysis declared-read verification.
Its first action admits leaf crates only; graphs needing foreign metadata
diagnose until exact dependency artifact staging is added. Use an explicit
`RustSourceCrateInfo` Bazel provider, declared per-file logical names and a
separate metadata action whose inputs include every source/doc artifact.

Publish a compiler-produced regular nonempty metadata file through atomic
no-replace hard-link creation from a private sibling stage. Existing files,
directories and symlinks must remain untouched, including a destination appearing
after staging begins. Unsupported filesystems diagnose rather than falling back
to an overwriting rename. Clean only the known staged file and its empty private
directory. This is I/O protection, not target-certificate authority.

Require actual Bazel-versus-direct metadata equality and loaded compiler
identity, relocated inputs, invalid/undeclared-source rejection, malformed option
rejection and no output/staging leaks before extending dependency metadata.

The first action test exposed rustc's raw working directory in metadata bytes
despite identical source mappings. Added working-directory normalization before
the exact per-file remaps, retaining the post-analysis undeclared-read guard.
The strict Bazel/direct/relocated byte-equality assertion now passes; it was not
weakened to name/signature agreement. Focused invocation
`e33bac26-890b-4ba9-a5bc-0f9696dd5f5b` passed all ten targets, including
the metadata action/probe, five no-replace publication tests, source-resolution
tests, compiler/Rust Clippy, formatting, buildifier and documentation checks.
Full integration `854f2d80-10f0-41d0-a615-907f3c4a72c0` passed all
319 tests. Independent review then found a real textual-prefix ordering defect:
logical-name ordering could put a shorter physical prefix after a longer one.
Accepted the finding; order every remap (including cwd) by increasing physical
prefix length. Real compiler byte-equality tests cover overlapping file names
and a working directory itself prefixed by an input file. This repair must pass
the expanded integration and fresh review before checkpoint closure.

## Declared dependency metadata extension (complete)

The isolated emitter now snapshots the exact declared metadata closure in its
private stage and exposes only direct alias edges via `--extern`. Bazel providers
carry explicit immutable graph records and reject conflicting defining keys;
each action reads only its source/docs and declared dependency artifacts.
Transitive sources remain descriptors for the later checked-source driver.

Resource caps and canonical/inode alias rejection are specified in the driver
contract. Separate filesystem tests cover snapshots, direct aliases, source and
artifact aliasing, missing/empty/directory/oversized artifacts, and owned cleanup.
The actual three-crate Bazel action test compares direct/relocated metadata bytes,
rejects stale transitive metadata and accidental transitive source imports, and
accepts equal crate names with distinct explicit keys. Focused gate
`ce16f229-87e8-4051-838a-71524adc3706` passed all seven test/lint targets.
Full integration `4d6c41af-3a78-49f4-81e5-b8efb1646fb6` passed all 322
tests. Fresh review identified three accepted contract gaps: source-to-source
hardlink aliases, the Bazel rule's inability to express repeated aliases for
one target, and graph limits exceeding OS argv capacity. Fixed all three with
inode uniqueness, an alias-keyed target dictionary and bounded verbatim response
files. Added a real repeated-alias Bazel action, hardlink rejection and a valid
multi-megabyte response-file test. Focused repair gate
`8b98e82a-f0bb-4ff6-8ec1-a56e44bfe0ef` passed all ten targets.
Expanded integration and another fresh review remain required. This
does not complete source-to-loaded-metadata authentication or enable foreign C
calls; those joins remain required.

The expanded gate `31dd0738-2217-47c3-80cd-1e1846e82d5e` passed all
325 tests across 372 targets. A further actual compiler test emits metadata from
an 800-input, >2 MiB response file (`ebba0d34-0bf8-4179-ac79-d1992f0a6d28`).
Review raised a possible legacy `extern crate` transitive-visibility bypass;
the pinned compiler rejects that syntax as well as extern-prelude access with
the existing dependency-only search path. Added and retained the regression in
`c93ed1e2-d594-4385-a083-7ed03c11d63c`; no source relaxation or workaround.

The next independent review found a genuine response framing defect: LF inside
an unvalidated Bazel attribute could become an extra input record before Rust
field validation. Accepted and fixed it by checking every serialized field,
including transitive provider records, for ASCII controls before serialization.
Six actual Bazel analysis-failure tests cover input logical names, root logical
names, crate names/keys, aliases and tabs. Intentional malformed fixture targets
are manual-only; their rejection tests are normal wildcard/CI tests. This does
not disable any successful-build or runtime test. Focused gate
`d12ddfa8-6973-4616-bfe2-c5fc3a1c1c12` passed all nine targets, including
normal generation, closure loading and buildifier. A new full gate and fresh
review remain required.

Full integration `c4d00c25-8ec5-4fe9-ab48-502e26ca7946` passed all 331
tests across 378 targets. Fresh broad read-only Sol Extra High review found no
actionable core defects in framing, graph/response budgets, resolved identities,
exact staging, declared reads or no-replace publication. The final expanded
gate `a7bab790-6742-4221-b874-17e259c927be` also passed all 331 tests,
including the exact loaded-artifact probe and narrowed adapter action inputs.
The declared-metadata checkpoint is complete; source authentication remains next.

## Checked driver preparation

Derive each compiler invocation's exact rooted subgraph from the typed graph,
retaining descriptor/alias equality and excluding sibling metadata. Cover
diamonds, nested roots, unknown roots and the full 1024-node chain without
recursion; reuse the graph constructor's checked ordering and budgets.
The four-target graph/lint/format/docs gate passed in
`40496797-9fee-4a4d-b782-52a017e92808`.

Unused explicit dependencies also need compiler identity checks. An isolated
probe showed that mutating rustc's crate store in `after_expansion` is too late:
the pinned compiler has already frozen it. Do not use that approach. The typed
compiler configuration hook can instead request resolution using `ExternEntry`
`force` before initialization, preserving explicit paths and direct-only source
visibility. The probe verifies unused metadata identity, equal source-analysis
and emitted hashes, and invalid/missing unused artifacts. Four focused targets
passed in `e6aa8494-8e0b-405e-99c0-fbcb2eb29f1d`.

This is probe evidence only. Before production use, share the fixed setting
between metadata emission and checked source analysis; retain typed compiler
identity/hash evidence and verify source-to-loaded-metadata agreement. No input
language feature exemption or arbitrary compiler flag channel is introduced.

The follow-on alias probe found that force-only loads do not populate rustc's
source-use alias lookup table. Therefore an unused dependency must be joined
through its exact compiler-reported metadata artifact path as well as its typed
crate/content identity. A used alias additionally supplies the resolved alias
identity. Do not infer a declared alias from mere membership in the loaded set.
The production join must compare against the driver's exact staged path for
that defining key, rejecting coordinated/swapped descriptor artifacts.
The exact-artifact probe, historical compiler agreement, capability contracts,
format/buildifier and docs all passed (14 targets) in
`e7a3fca9-85fb-41a8-90b4-3cdf9732de3d`.

The C adapter's Bazel inputs now name only its actual module trees, excluding
standalone metadata tools and the separately cached configuration library source.
Compiler-library dependency edges remain explicit. Metadata-only edits no longer
invalidate unrelated C adapter compilation through a catch-all source glob.

## Checked-source implementation (complete)

The separate `source_checker` executable now rechecks the graph dependency-first,
retains private typed `StableCrateId`/`Svh` evidence, and compares exact loaded
artifact paths and used-alias resolutions. Metadata emission shares its fixed
pre-initialization dependency-loading setting. Both use the same configuration
library and exact dependency staging, but their Bazel compilation actions remain
independent. Scratch stages have no publication operation.

Actual compiler cases cover unused transitive dependencies, repeated and used
aliases, diamonds, metadata swaps, stale bodies/docs/configuration, physical
relocation, undeclared documentation and graph-wide inode overlap. The focused
11-target gate `3011da97-f118-4527-a38b-fa231069da56` passed.
The new provider-based source-check action additionally declares every graph
source/doc and dependency metadata input; it uses the same control-safe response
serializer as metadata generation. Its report is descriptive, not proof authority.
Full integration and fresh independent review remain required before closure.

Full integration `a885701d-bd57-4b42-854e-cba813bd05ad` passed all
332 tests across 382 targets, including declared source-check actions, negative
framing tests, compiler Clippy, Rust formatting, buildifier and existing C/Java
gates. The source-authentication test additionally checks its private scratch
directory is empty after every successful and failed invocation. Fresh review
is in progress; this task remains open until its findings are evaluated.

The reviewer found that the callback received all previously checked results,
including unrelated siblings absent from its compiler subgraph. Accepted this
scope defect and replaced the callback argument with a structurally filtered
dependency-only view. An actual compiler callback-contract executable now checks
exact dependency membership, owner/result association and once-only topological
invocation across the positive fixtures, including the diamond. All runtime
checks passed in `496f2bbe-1839-450d-9d8e-ea05f5f1980c`; only Bazel
list indentation needed formatting before the next full gate.

Independent filesystem tests additionally cover exact 256 MiB metadata limits,
one-byte excess, per-file/nonregular rejection, graph-wide inode overlap, root
metadata non-consumption and canonical symlinked toolchain allowlists. Their
five-target test/Clippy/format/buildifier/docs gate passed in
`8aa41407-884a-4665-b494-1cdd5854668f`.

Full scope-repair gate `30faa713-c7b5-4caa-a0df-f71b217dabf3` passed
all 334 tests across 385 targets. Review coverage does not include a real
1024-invocation compiler run or injected cleanup syscall failures: graph limits
are tested independently, and cleanup guarantees remain owned/nonrecursive
best-effort cleanup rather than immunity to filesystem failure. A further
end-to-end regression covers same-name/distinct-key crates sharing source,
including swapped metadata rejection. A fresh independent review is required.

The same-name/distinct-key regression and documentation gate passed in
`a001f967-dc5b-4932-a235-3dbab7cff1a4`. Read-only Bazel action queries
confirmed whole-graph source checking tracks all three source roots plus its
two dependency metadata artifacts (`93149e92-42e5-4a93-a9bc-466aae87492f`),
while the root metadata action tracks only its own source plus those metadata
artifacts (`45266bfd-e560-449e-aaad-3799f6c8ac76`). These are actual
declared action inputs, not an inference from a source-file split.

Final full gate `2690b192-1819-4334-bd60-e7f0e95f28c9` passed all
334 tests across 385 targets, including the final same-name/shared-source
regression. A fresh broad read-only Sol Extra High reviewer found no actionable
core defects in the repaired checked-source/metadata/Bazel boundary. The
configuration, declared metadata and checked-source driver checkpoints are
complete. C foreign bindings, target certificates and bundle publication remain
04C-03; compiler agreement alone grants no target-generation authority.

## Scope boundary

Successful compiler evidence alone does not allow foreign C calls or publish
partial output. The full C import and publication join is 04C-03.
