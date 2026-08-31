# v0.1 generation benchmark

The release benchmark builds and checks 1,000 public alias declarations, then
generates Rust, TypeScript, Python, and Go manifests in one process. The enforced
PRD limits are generation under 2,000 ms and peak process memory under 512 MiB.

Run it in the pinned Linux development container:

```sh
bazelisk test //crates/benchmark:generation_benchmark_test --test_output=all
```

The Bazel test launches `generation_benchmark`, measures child peak RSS with
Linux `getrusage`, records wall time and platform information, verifies exactly
1,000 declarations and four targets, and fails either limit. It prints one
canonical JSON measurement suitable for attaching to CI evidence.

## Baseline measurement

- Tool: `polyrust-v0.1`, Rust/Bazel toolchain 1.98.0/9.2.0
- Host: Linux 6.18.33.2 Microsoft WSL2 x86_64, glibc 2.41
- Fixture: 1,000 declarations, four targets, 23 files, 1,073,079 output bytes
- Generation time: 265 ms (269 ms measured process wall time)
- Peak RSS: 11.09 MiB
- Result: pass; 1,735 ms and 500.91 MiB below the PRD limits

This is a local feasibility baseline, not a promise of identical timing on other
machines. CI enforces the same limits and records its own host-specific line.

