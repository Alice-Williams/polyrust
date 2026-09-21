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

## Local storage budgets

Run builds through `bazel` (the container's Bazelisk symlink) or `bazelisk`.
The checked-in `tools/bazel` hook automatically applies local storage policy
when `/workspace` is a mounted directory in a Docker container. Do not bypass
this hook with `BAZELISK_SKIP_WRAPPER`, a directly downloaded Bazel binary, or an
older candidate snapshot without the hook. GitHub Actions is passed through
unchanged and retains its existing persistent caches and cached test results.

- Artifact cache: **50 GB** (decimal), oldest entries evicted before and after
  builds; entries older than 14 days are also evicted. Both AC and CAS entries
  remain cacheable, including test results. A running build can temporarily
  exceed this cache target; this is an eviction budget, not a filesystem quota.
- Docker filesystem: local builds are refused/stopped at **200 GB used**.
- Host drive: local builds require at least **150 GB free** on the filesystem
  containing `/workspace`. Checks run before, during (every two seconds), and
  at the end of a build. Already-running writes can overshoot these thresholds.
- Local Bazel invocations use batch mode and a shared lock to prevent cache
  maintenance from overlapping other guarded builds. Termination stops the
  batch process group; source files and output bases are never pruned.

These are **build guards, not a hard limit on Docker's virtual-disk capacity**.
Other containers and direct/unwrapped commands can still consume disk space.
Deleting cache files also does not necessarily shrink the host VHDX file.
Keep only active build output bases: after a candidate is completed, inspect
and explicitly remove its disposable outputs, never its source snapshot.

New Dev Containers mount the named Bazel volumes at `/var/tmp/polyrust-cache`,
matching `.bazelrc`. Existing containers retain their original mounts until
recreated; do not recreate an existing container without preserving its work.
Focused verification: `bazelisk test //tools/dev:storage_budget_test`.

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
