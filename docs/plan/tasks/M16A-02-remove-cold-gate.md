# M16A-02 — Remove the redundant cold release gate

- Status: in progress
- Depends on: M16A-01
- Blocks: none

## Goal

Run the complete release gate once against the restored Bazel cache and rely
on Bazel action keys to invalidate work whose declared inputs change.

## Definition of done

- GitHub Actions contains no isolated `polyrust-cold` cache tree.
- The release job invokes `tools/release/release_gate.sh` exactly once.
- The single gate mounts all persistent Bazel and Cargo cache paths.
- Cache absence still falls back to a complete execution that populates the
  persistent tree.
- Cache save remains success-gated and happens after the release gate.

## Tests

- The Bazel cache-policy test rejects a cold tree or duplicate release-script
  invocation and checks the persistent mounts and ordering.
- Buildifier, documentation tests, and actionlint pass.
- The complete release gate passes in the Linux development container.
- A hosted run passes after the change is pushed.

## Commit gate

Commit and push after local verification. Mark complete only after the hosted
run supplies the final evidence.

## Local evidence

- Pinned actionlint 1.7.12 accepts the updated workflow.
- The focused cache-policy, Buildifier, and documentation tests pass.
- The uncached tracked release suite passes 237 of 237 tests, including the
  Rust and Bazel linters.
- An immediate normal cached replay also passes 237 of 237 while executing
  only one test; Bazel reports the other 236 results as cached.
