#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"
readonly workflow="${root}/.github/workflows/ci.yml"
readonly bazelrc="${root}/.bazelrc"
readonly install_bazelisk="${root}/tools/ci/install_bazelisk.sh"
readonly install_release_tools="${root}/tools/ci/install_release_tools.sh"

python3 - "${workflow}" "${bazelrc}" "${install_bazelisk}" "${install_release_tools}" <<'PY'
from pathlib import Path
import sys

workflow = Path(sys.argv[1]).read_text(encoding="utf-8")
bazelrc = Path(sys.argv[2]).read_text(encoding="utf-8")
install_bazelisk = Path(sys.argv[3]).read_text(encoding="utf-8")
install_release_tools = Path(sys.argv[4]).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"CI cache policy violation: {message}")


def step(name: str, next_name: str | None) -> str:
    start_marker = f"      - name: {name}\n"
    require(start_marker in workflow, f"missing step {name!r}")
    start = workflow.index(start_marker)
    if next_name is None:
        return workflow[start:]
    end_marker = f"      - name: {next_name}\n"
    require(end_marker in workflow[start + len(start_marker):], f"missing step {next_name!r}")
    end = workflow.index(end_marker, start + len(start_marker))
    return workflow[start:end]


restore = step("Restore persistent non-semantic caches", "Report persistent cache restoration")
gate = step("Cached complete release gate", "Prepare persistent cache archive")
prepare = step("Prepare persistent cache archive", "Save refreshed persistent non-semantic caches")
save = step("Save refreshed persistent non-semantic caches", None)

require("uses: actions/cache/restore@v6" in restore, "cache restore action is not pinned")
require("uses: actions/cache/save@v6" in save, "cache save action is not pinned")
for compatibility_input in (
    "Cargo.lock",
    "MODULE.bazel",
    "MODULE.bazel.lock",
    ".bazelversion",
    ".bazelrc",
    "tools/ci/install_bazelisk.sh",
    "tools/ci/install_release_tools.sh",
):
    require(compatibility_input in restore, f"restore key omits {compatibility_input}")
    require(compatibility_input in save, f"save key omits {compatibility_input}")
require("${{ github.run_id }}-${{ github.run_attempt }}" in restore, "restore key is not per attempt")
require("${{ github.run_id }}-${{ github.run_attempt }}" in save, "save key is not per attempt")
require("restore-keys:" in restore, "compatible prefix restore is missing")
require("polyrust-v4-" in restore and "polyrust-v4-" in save, "native cache schema is not selected")
require("if: ${{ success() }}" in save, "cache save is not success-gated")
require(workflow.index("Cached complete release gate") < workflow.index("Prepare persistent cache archive") < workflow.index("Save refreshed persistent non-semantic caches"), "cache archive preparation/save ordering is invalid")
require('find /var/tmp/polyrust-cache ! -readable -print -quit' in prepare, "Bazel cache readability is not asserted before save")
require('find "$RUNNER_TEMP/polyrust-cache" ! -readable -print -quit' in prepare, "cache readability is not asserted before save")

for cache_path in ("bin", "bazelisk", "bazel-repository", "bazel-disk"):
    persistent = f"/var/tmp/polyrust-cache/{cache_path}"
    require(persistent in restore and persistent in save, f"persistent {cache_path} is not cached")

for cache_path in ("cargo-home", "cargo-tools", "node-tools", "python-tools"):
    persistent = f"${{{{ runner.temp }}}}/polyrust-cache/{cache_path}"
    require(persistent in restore and persistent in save, f"persistent {cache_path} is not cached")

require("polyrust-cold" not in workflow, "obsolete cold cache tree is present")
require("docker build" not in workflow and "docker run" not in workflow, "Linux CI invokes Docker")
require(".devcontainer/Dockerfile" not in workflow, "Linux CI depends on the development image")
require("uses: actions/setup-node@v6" in workflow, "native Node setup is missing")
require("uses: actions/setup-python@v6" in workflow, "native Python setup is missing")
require("bash tools/ci/install_bazelisk.sh" in workflow, "pinned Bazelisk bootstrap is missing")
require("bash tools/ci/install_release_tools.sh" in workflow, "native release-tool bootstrap is missing")
release_command = 'bash tools/release/release_gate.sh'
require(gate.count(release_command) == 1, "cached gate does not run the complete release script exactly once")
require(workflow.count(release_command) == 1, "workflow runs the complete release script more than once")
require("$PWD" not in restore and "$PWD" not in save, "checkout path is cached")
require("password" not in restore.lower() and "token" not in restore.lower(), "credential-like path is cached")
require("BAZELISK_HOME: /var/tmp/polyrust-cache/bazelisk" in workflow, "Bazelisk home is not persistent")
require("--repository_cache=/var/tmp/polyrust-cache/bazel-repository" in bazelrc, "repository cache is not host-neutral")
require("--disk_cache=/var/tmp/polyrust-cache/bazel-disk" in bazelrc, "disk cache is not host-neutral")
require("test --test_env=RUSTUP_HOME" in bazelrc, "Rust native tests do not receive the selected rustup home")
require('readonly version="1.29.0"' in install_bazelisk, "Bazelisk version is not pinned")
require('readonly expected_sha256="' in install_bazelisk, "Bazelisk checksum is not pinned")
require('printf \'RUSTUP_HOME=%s\\n\'' in install_release_tools, "native Rust toolchain home is not exported")
for version in ("1.98.0", "7.0.2", "3.9.6", "0.16.5", "2.3.1", "9.1.1", "0.22.2"):
    require(f'"{version}"' in install_release_tools, f"native release tool {version} is not pinned")
PY
