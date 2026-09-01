# v0.1 generation benchmark

The release benchmark builds and checks 1,000 public alias declarations, then
generates Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C manifests in
one process. The enforced
PRD limits are generation under 2,000 ms and peak process memory under 512 MiB.

Run it in the pinned Linux development container:

```sh
bazelisk test //crates/benchmark:generation_benchmark_test --test_output=all
```

The Bazel test launches `generation_benchmark`, measures child peak RSS with
Linux `getrusage`, records wall time and platform information, verifies exactly
1,000 declarations and eight targets, and fails either limit. It prints one
canonical JSON measurement suitable for attaching to CI evidence.

## Baseline measurement

- Tool: `polyrust-v0.1`, Rust/Bazel toolchain 1.98.0/9.2.0
- Host: Linux 6.18.33.2 Microsoft WSL2 x86_64, glibc 2.41
- Fixture: 1,000 declarations, eight targets, 47 files, 2,143,600 output bytes
- Generation time: 423 ms (427 ms measured process wall time)
- Peak RSS: 11.80 MiB
- Result: pass; 1,577 ms and 500.20 MiB below the PRD limits

This is a local feasibility baseline, not a promise of identical timing on other
machines. CI enforces the same limits and records its own host-specific line.
