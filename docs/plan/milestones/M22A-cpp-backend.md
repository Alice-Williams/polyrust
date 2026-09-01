# M22A — C++20 backend checkpoint

- Status: complete
- Phase: 7
- Depends on: M21

## Outcome

Add C++20 as the seventh independent output. The backend consumes checked IR,
emits a typed public API plus a private portable runtime, and is compiled by the
pinned hermetic Zig toolchain. A separately pinned GCC 14.2 container compiler
provides ASan and UBSan because the Zig Bazel wrapper does not link sanitizer
runtimes.

## Completion evidence

- Registration, public consumer, comprehensive models-and-validation, and all
  four completed MIT real-world ports compile and execute under C++20.
- Every generated C++ compile uses `-Wall -Wextra -Wpedantic -Werror`.
- The shared C++ style gate passes for generated headers, runtime, sources, and
  tests.
- ASan with leak detection and UBSan with recovery disabled pass for the
  registration fixture, comprehensive model, and all completed ports.
- Seven manifests regenerate byte-identically; CLI, conformance, benchmark,
  release policy, Rustfmt, Clippy, and Buildifier include C++.
- `bazelisk test //...` and `bazelisk test //:release_gate` pass in the Linux
  development container.

## Scope boundary

Zig is the C/C++ compiler driver, not a source conversion hub. Checked IR
remains the semantic hub. C17 remains M22B and cannot reuse C++ source, STL
containers, exceptions, or destructors.
