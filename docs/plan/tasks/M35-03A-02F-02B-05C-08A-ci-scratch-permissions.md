# M35-03A-02F-02B-05C-08A — Read-only Bazel scratch copies

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-08-end-to-end-closure.md)

## Failure and contract

GitHub run 35169696731 at commit 35a72e6 passed 740/741 tests, but
constant_export_native_test failed with PermissionError when mutating its
copied bundle.json. shutil.copytree preserved read-only Bazel artifact modes.
The root dev-container user masked the failure.

Make only newly created, private test copies owner-writable, including nested
directories and files. Do not chmod or modify original Bazel outputs, relax
sandboxing, rerun tests as root in CI, or disable the mutation controls.

## Implementation and definition of done

- One small shared test helper copies to a new destination and enables owner
  write permission there while preserving executable bits.
- Schema, generated-producer and exported-example copies use that helper, declared as
  an explicit Bazel test input.
- A regression test runs without root privileges even in the dev container.
  It first reproduces PermissionError with an unmodified readonly copy, then
  proves schema/source edits and nested output creation work with the helper.
  Original bytes/modes remain unchanged; an existing destination rejects.
- The native constant export proof and complete Linux Bazel release/lint gate
  pass on an exact tree containing only this follow-up, before its own commit
  and push. Preserve the independent in-progress unit-result checkpoint.
- Monitor the pushed revision's actual GitHub result; local success is not a
  substitute for remote completion.

## Completion evidence

- Exact isolated tree 5f02baa798ec690a7fc7ee5b2fba4a748e6e3300 passed Linux
  dev-container Bazel //... //:release_gate: 1,111 targets, **742/742 tests**,
  5 executed / 737 cached, 65.085 seconds. Invocation:
  67c89536-9963-496a-b06d-13b0eb49a101.
- The new Bazel permission test actually runs the child unprivileged. The old
  readonly copy fails its deliberate write; all three production test-copy sites
  use the corrected helper. Schema/producer mutation controls and native
  Rust/GCC/Zig/Java21 equivalence pass unchanged.
- Exact original file/directory modes and bytes stay unchanged; destination
  modes add only owner write/required directory execute. Executable bits remain.
  Existing-destination rejection preserves already-mutated contents.
- Review findings about the example-output copy site and root-only runfile
  ancestry were accepted: all copies use the helper, and the permission probe
  stages the helper into its accessible temporary directory before demotion.
- The unrelated unit-result and ownership work is excluded. An earlier
  shared-index archive had out-of-scope omissions and its 327-test receipt is
  invalid; isolated-index reconstruction and an explicit changed-path allowlist
  prevent using that incomplete tree as release evidence.
- The documentation-only completion tree is gated again before its scoped
  commit/push. GitHub completion is tracked separately for the exact pushed SHA.
