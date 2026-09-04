#!/usr/bin/env bash
set -euo pipefail

readonly runfiles="${RUNFILES_DIR:-$0.runfiles}"
readonly root="${runfiles}/${TEST_WORKSPACE}"
readonly workflow="${root}/.github/workflows/ci.yml"

python3 - "${workflow}" <<'PY'
from pathlib import Path
import sys

workflow = Path(sys.argv[1]).read_text(encoding="utf-8")


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
cold = step("Cache-cold complete gate", "Cache-warm complete gate")
warm = step("Cache-warm complete gate", "Save refreshed persistent non-semantic caches")
save = step("Save refreshed persistent non-semantic caches", None)

require("uses: actions/cache/restore@v6" in restore, "cache restore action is not pinned")
require("uses: actions/cache/save@v6" in save, "cache save action is not pinned")
for compatibility_input in (
    "Cargo.lock",
    "MODULE.bazel",
    "MODULE.bazel.lock",
    ".devcontainer/Dockerfile",
    ".bazelversion",
    ".bazelrc",
):
    require(compatibility_input in restore, f"restore key omits {compatibility_input}")
    require(compatibility_input in save, f"save key omits {compatibility_input}")
require("${{ github.run_id }}-${{ github.run_attempt }}" in restore, "restore key is not per attempt")
require("${{ github.run_id }}-${{ github.run_attempt }}" in save, "save key is not per attempt")
require("restore-keys:" in restore, "compatible prefix restore is missing")
require("if: ${{ success() }}" in save, "cache save is not success-gated")
require(workflow.index("Cache-warm complete gate") < workflow.index("Save refreshed persistent non-semantic caches"), "cache saves before warm proof")

for cache_path in ("bazelisk", "bazel-repository", "bazel-disk"):
    persistent = f'$RUNNER_TEMP/polyrust-cache/{cache_path}:/root/.cache/{cache_path}'
    isolated = f'$RUNNER_TEMP/polyrust-cold/{cache_path}:/root/.cache/{cache_path}'
    require(persistent in warm, f"warm gate does not mount persistent {cache_path}")
    require(isolated in cold, f"cold gate does not mount isolated {cache_path}")
    require(persistent not in cold, f"cold gate mounts persistent {cache_path}")
    require(isolated not in restore and isolated not in save, f"cold {cache_path} is persisted")

require('rm -rf "$RUNNER_TEMP/polyrust-cold"' in cold, "cold tree is not freshly removed")
require('find "$RUNNER_TEMP/polyrust-cold/$path" -mindepth 1 -print -quit' in cold, "cold emptiness is not asserted")
release_command = 'bash tools/release/release_gate.sh'
require(cold.count(release_command) == 1, "cold gate does not run the complete release script exactly once")
require(warm.count(release_command) == 1, "warm gate does not run the complete release script exactly once")
require("$PWD" not in restore and "$PWD" not in save, "checkout path is cached")
require("password" not in restore.lower() and "token" not in restore.lower(), "credential-like path is cached")
PY
