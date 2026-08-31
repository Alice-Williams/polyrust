# Development environment

The authoritative development environment is a disposable Linux container with
the repository bind-mounted at `/workspace`. This prevents Windows host details
from influencing generated-code tests.

## Pinned toolchain

| Component | Version | Source of truth |
| --- | --- | --- |
| Container Rust | 1.98.0 | `.devcontainer/Dockerfile` |
| Bazelisk | 1.29.0 | `.devcontainer/Dockerfile` |
| Bazel | 9.2.0 | `.bazelversion` |
| Bazel Rust SDK | 1.98.0 | `MODULE.bazel` |
| `rules_rust` | 0.74.0 | `MODULE.bazel` |
| Bazel Go SDK | 1.25.14 | `MODULE.bazel` |
| `rules_go` | 0.63.0 | `MODULE.bazel` |
| Buildifier | 8.5.1.4 | `MODULE.bazel` |

Bazel downloads the Rust and Go SDKs hermetically. The container also includes
Cargo, rustfmt, and Clippy for Rust ecosystem tooling. It intentionally does not
install a second system Go; use the Bazel-managed SDK.

## Start the environment

The normal route is **Dev Containers: Reopen in Container** in an editor that
supports the checked-in configuration.

The equivalent Docker commands from PowerShell are:

```powershell
docker build --tag polyrust-dev --file .devcontainer/Dockerfile .
docker run --rm --interactive --tty `
  --mount "type=bind,source=$($PWD.Path),target=/workspace" `
  polyrust-dev bash
```

Inside the container, the repository is `/workspace`.

## Verify Step 0

```bash
bazelisk version
bazel test //...
bazel run @io_bazel_rules_go//go -- version
rustc --version
cargo --version
```

`bazel test //...` builds and tests Rust and Go through their pinned Bazel
toolchains. It also runs `rustfmt`, Clippy with warnings denied, and Buildifier's
format and Starlark lint checks. It is the Step 0 release gate and remains the
top-level verification command as implementation targets are added.

## Dependency policy

- Add build rules through Bzlmod in `MODULE.bazel` and commit the resulting
  `MODULE.bazel.lock`.
- Pin compiler and build-rule versions. Do not select toolchains from `PATH`.
- Record the purpose and license of runtime/library dependencies when the first
  real crate is added in M01.
- Persistent Docker volumes cache downloads but are disposable and never part of
  correctness.
