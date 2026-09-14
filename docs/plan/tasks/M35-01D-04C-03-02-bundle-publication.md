# M35-01D-04C-03-02 — Imported inventories and atomic bundle publication

- Status: complete
- Depends on: [M35-01D-04C-03-01](M35-01D-04C-03-01-compiler-foreign-bindings.md)
- Parent: [M35-01D-04C-03](M35-01D-04C-03-foreign-bindings.md)

## Outcome

Publish a complete, bounded, separately owned C crate bundle only after every
source and target obligation succeeds. Preserve the existing one-crate layout
and new-directory no-replace publication guarantees.

The old single-crate publication implementation uses an absence check followed
by std::fs::rename. That preserves ordinary preexisting destinations but is not
an atomic no-replace primitive for a racing empty directory. Do not copy that
gap into the bundle path or claim its race is already tested.

## Implementation

1. Extend descriptive API inventories with separate imported function records,
   retaining their compiler declaration identity and actual dependency owner,
   public header and linked symbol. Reconstruct both directions from compiler
   bindings and certified target packages. JSON never becomes proof input.
2. Define a collision-free flat bundle of crate-qualified header/source pairs
   and separately named per-crate manifests. A root index identifies the root
   and owning members without changing Rust public/private visibility.
3. Validate the complete bundle's ownership, path/count/byte limits, exact
   membership and single-definition policy before rendering or publication.
   Reuse typed imports and the structural renderer; do not rewrite generated
   text or synthesize dependency includes.
4. Render all members before a new-output atomic publication. Preserve existing
   outputs on failure, clean only owned staging files and avoid recursive
   deletion. Keep the single-crate three-file guard in its existing path.
   On Linux, prove an actual no-replace rename operation and reject unsupported
   filesystems rather than fall back to overwrite-capable rename. Keep platform
   publication separate from language lowering and preserve Rust's unsafe-code
   prohibition and the no-new-third-party-dependency preference.
5. Connect the explicit Bazel provider graph to the C bundle action with every
   source/doc/metadata input declared. Generated output remains untracked.

## Tests and proof

- Actual two-crate, transitive and diamond output has one implementation per
  owner, exact imported/owned inventories, correct public headers and no copied
  dependency bodies or private foreign definitions.
- Record order and physical relocation do not alter generated logical output.
  Repeated aliases do not duplicate symbols, headers, packages or files.
- Missing/extra/wrong-owner/wrong-signature/import-manifest mutations reject.
- Failures in source checking, lowering, certification, rendering, collision
  checks and publication leave absent output absent and existing output intact.
- A destination appearing after staging, including an empty directory, must
  survive unchanged. Test the platform primitive independently before wiring it
  into the bundle publisher; syscall failure must not trigger an unsafe fallback.
- Existing single-crate output and all release/frontend/C/shared/Java,
  lint/format/policy gates pass, followed by fresh broad independent review.
- Record native Rust-versus-C differential closure separately under 04D rather
  than claiming this publication gate proves equivalence on its own.

## Definition of done

All output and rejection contracts above have tests and recorded gate/review
evidence; parent 04C-03 is complete only with both child tasks complete.
No generated artifact, cache or compiler proof serialization is committed.

## Inventory implementation evidence (in progress)

The certificate-only import view and exact compiler/target reconstruction passed
all nine focused C graph/backend/docs targets in
`0feb3806-e7ee-45a4-a0bf-41f3f329853e`. The backend test checks exact
certificate retention, consumer branding, once-per-package imports across two
units, empty inventories and separation from local definitions, including
registered-but-unused imports. Version-two serialization and real-compiler
typed inventory mutations are the next validation increment. The standalone
version-one format remains unchanged and refuses imported inventories.

At that inventory checkpoint publication was not yet implemented. The subsequent
implementation and proof increments are recorded below; full integration and
fresh bundle review remain required.

The five-target import/compiler/native standalone/format/docs gate passed in
`473afdf4-a8b6-4796-9e22-594a153580cc`. Fresh Sol Extra High review of
the view, compiler binding retention, exact reconstruction, schema compatibility,
encoded bounds and typed mutation tests found no actionable findings. Parsed
schema-v2 assertions will be included with actual bundle output tests.

## Platform experiment evidence

Initial independent test preparation (while 03-01 was reviewed) isolated a first-party
Linux C probe around renameat2 with the no-replace flag. It is not wired into the
Rust driver at that point and added no third-party dependency or Rust unsafe block. Its tests
exercise complete-directory visibility, late existing empty/nonempty/file/link
destinations, failed parents and two concurrent publishers. Production wiring
and the remaining bundle contracts are still planned.

## Bundle implementation evidence (in progress)

The --bundle operation now retains an opaque checked graph, validates exact
member/manifest/import authority and global ownership/byte budgets, renders all
members structurally, and publishes exactly 3*N+1 files. The first generated
bundle/native single-crate/platform/lint/docs gate passed all five targets in
`f70fdb4d-096e-41c6-a82c-1e272fd4fbc6`.

The Linux helper is now wired through an explicit Bazel runtime dependency,
including its runfiles. The ordinary single-crate publisher also uses the same
atomic no-replace operation while retaining its three-file guard. Unsupported
kernel/filesystem operations fail without an overwrite fallback. Staging is
owner-only and cleans explicitly owned files without recursive traversal.

Six focused targets passed in `fbabe22b-f4ec-4271-828a-bacb59d8e1a4`:
actual transitive and repeated-alias Bazel tree artifacts; parsed version-two
manifests and exact file/header ownership; reordered and physically relocated
diamond input byte equality; existing directory/file/link preservation; actual
Rust publisher late empty/nonempty/file/link races; helper execution/failure and
partial-write cleanup; plus Clippy/Bazel format/docs. The native platform test
also retains its two-concurrent-publishers/one-winner assertion.

Further typed compiler contracts corrupt private bundle invariants (missing
root/import owner, wrong key/manifest, and a separately recertified same owner)
to prove preflight does not substitute stable IDs for exact authority. An
independently cached budget test exercises count boundaries, exact 256 MiB,
one-byte overflow and u64 overflow without allocating enormous fixtures.
Full integration and fresh broad bundle review are in progress. Multi-crate
native differential closure remains 04D, not established by these checks.

Full integration `1fb8e797-85dd-479f-a965-67e428b198d1` passed all
340 tests across 399 targets, including C/shared/Java, native proofs, typed
mutations, budgets and lint/policy gates. Fresh broad Sol Extra High review
found no actionable core defects. Its optional Bazel staging observation led
to a smaller action: remove only Bazel's empty precreated tree with rmdir, then
publish directly through the CLI's atomic rename. A nonempty tree still fails
without modification. This avoids an outer temporary directory and per-file
handoff; its final validation is pending below.

Final full gate `5195cc51-42df-4ffc-afc0-4d4dba87cf2d` passed all
340 tests across 399 targets after the action simplification; cache reuse reduced
the gate to 26.4 seconds. Final independent read-only review confirmed no new
core defect. Action inspection `8d24b854-5aa6-48a0-9c95-2180d49cfaac`
confirmed the root bundle action consumes all three source files plus only the
leaf/middle metadata artifacts, not unused root metadata. Its tested ten-file
artifact has an ignored workspace copy at
experiments/rustc-frontend/output/crate-bundle/m35-04c.
Implementation/proof is complete; commit/push remains held for the requested
C/Java integration checkpoint. No multi-crate native equivalence claim is made
until 04D completes.

The probe passed its native/buildifier/docs gate in
`57577c08-0552-4d20-80c9-6a8ca54621a3`. The pinned hermetic C ABI is
glibc 2.17, so the probe uses the existing syscall entry point with the platform's
SYS_renameat2 constant rather than require a newer libc wrapper. Unsupported
kernel/filesystem behavior remains an error; there is no overwriting fallback.

[Rust rename](https://doc.rust-lang.org/std/fs/fn.rename.html) may replace an
existing empty directory on Unix. The Linux
[renameat2 no-replace flag](https://man7.org/linux/man-pages/man2/rename.2.html)
provides the required atomic refusal when supported by the filesystem. The
driver's selected implementation must be measured in the pinned Linux tests,
not assumed from documentation alone.
