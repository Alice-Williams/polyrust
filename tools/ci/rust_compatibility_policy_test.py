"""Enforce compile-only Cargo compatibility and mandatory Bazel execution."""
from pathlib import Path
import re
import sys


CARGO = 'cargo +"${{ matrix.toolchain }}" test --no-run --workspace --all-features --locked'
BAZEL = "bazelisk --batch test --test_output=errors //..."


def job(workflow, name):
    match = re.search(
        rf"^  {re.escape(name)}:\n(.*?)(?=^  [a-z][a-z-]*:|\Z)",
        workflow,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise ValueError(f"missing CI job: {name}")
    return match.group(1)


# This is a deliberately closed CI recipe, not a general YAML/shell validator.
COMPATIBILITY = """
    name: Rust ${{ matrix.toolchain }}
    runs-on: ubuntu-24.04
    strategy:
      fail-fast: false
      matrix:
        toolchain: ["1.98.0", stable]
    steps:
      - uses: actions/checkout@v7
      - name: Install selected Rust
        shell: bash
        run: rustup toolchain install "${{ matrix.toolchain }}" --profile minimal
      - name: Compile workspace tests with selected Rust
        shell: bash
        run: cargo +"${{ matrix.toolchain }}" test --no-run --workspace --all-features --locked
"""
RELEASE = r'''
set -euo pipefail
for tool in cargo rustfmt bazelisk node npm tsc prettier python3 ruff mypy pytest cargo-audit; do
    command -v "${tool}" >/dev/null || {
        echo "required release tool is missing: ${tool}" >&2
        exit 1
    }
done
cargo audit --deny warnings
bazelisk --batch test --test_output=errors //...
bazelisk --batch run //crates/conformance:polyrust-conformance -- --all-targets --determinism
bazelisk --batch test --test_output=errors //:release_gate
'''


def lines(text):
    return [line.rstrip() for line in text.splitlines()
            if line.strip() and not line.lstrip().startswith("#")]


def verify(workflow, release):
    workflow = "\n".join(lines(workflow)) + "\n"
    compatibility = job(workflow, "rust-compatibility")
    if lines(compatibility) != lines(COMPATIBILITY):
        raise ValueError("Rust matrix must follow the complete compile-only recipe")
    gate = job(workflow, "release-gate")
    job_fields = [line for line in gate.splitlines() if re.match(r"^    \S", line)]
    if job_fields != [
        "    name: Cached release gate",
        "    needs: [windows-contract, fast, rust-compatibility, compare-manifests]",
        "    runs-on: ubuntu-24.04",
        "    steps:",
    ]:
        raise ValueError("native release must depend on every prerequisite")
    if "continue-on-error:" in gate:
        raise ValueError("release failures must remain blocking")
    required_steps = re.findall(
        r"^      - name: Cached complete release gate\n(.*?)(?=^      - |\Z)",
        gate, re.MULTILINE | re.DOTALL,
    )
    if len(required_steps) != 1 or lines(required_steps[0]) != [
        "        shell: bash",
        "        run: bash tools/release/release_gate.sh",
    ]:
        raise ValueError("native release step must execute the complete script")
    if lines(release) != lines(RELEASE):
        raise ValueError("release recipe must execute all mandatory checks")


def main():
    workflow, release = (Path(path).read_text(encoding="utf-8") for path in sys.argv[1:])
    verify(workflow, release)
    mutants = [
        (workflow.replace("--no-run ", ""), release),
        (workflow.replace("--workspace ", ""), release),
        (workflow.replace("--all-features ", ""), release),
        (workflow.replace(" --locked", ""), release),
        (workflow.replace('cargo +"${{ matrix.toolchain }}"', "cargo"), release),
        (workflow.replace('toolchain: ["1.98.0", stable]', 'toolchain: ["1.98.0"]'), release),
        (workflow.replace('run: rustup toolchain install', 'run: echo toolchain install'), release),
        (workflow.replace("  rust-compatibility:\n", "  rust-compatibility:\n    continue-on-error: true\n"), release),
        (workflow.replace("  release-gate:\n", "  release-gate:\n    if: false\n"), release),
        (workflow.replace("fast, rust-compatibility, compare-manifests", "fast, compare-manifests"), release),
        (workflow.replace("run: bash tools/release/release_gate.sh", "run: echo skipped"), release),
        (workflow, release.replace(BAZEL, BAZEL.replace(" test ", " build "))),
        (workflow, release.replace(BAZEL, BAZEL.replace("//...", "//:rustfmt_test"))),
        (workflow, release.replace(BAZEL, BAZEL + " --test_tag_filters=-native")),
        (workflow, release.replace(BAZEL, BAZEL + " || true")),
        (workflow.replace("      - name: Compile workspace tests with selected Rust\n",
                          "      - name: Compile workspace tests with selected Rust\n        if: false\n"), release),
        (workflow.replace("      - name: Cached complete release gate\n",
                          "      - name: Cached complete release gate\n        if: false\n"), release),
        (workflow.replace(f"        run: {CARGO}\n",
                          f"        run: {CARGO}\n      - name: Accidental runtime tests\n        run: {CARGO.replace('--no-run ', '')}\n"), release),
        (workflow.replace("run: bash tools/release/release_gate.sh",
                          "run: bash tools/release/release_gate.sh || true"), release),
        (workflow, release.replace("set -euo pipefail", "set -uo pipefail")),
        (workflow, release.replace("set -euo pipefail", "set -euo pipefail\nexit 0")),
        (workflow + "\n    if: false\n", release),
        (workflow.replace("shell: bash", "shell: bash -n {0}"), release),
        (workflow.replace('toolchain: ["1.98.0", stable]',
                          'toolchain: ["1.98.0", stable]\n        exclude: [{toolchain: stable}]'), release),
        (workflow.replace("    needs: [windows-contract, fast, rust-compatibility, compare-manifests]",
                          "    # needs: [windows-contract, fast, rust-compatibility, compare-manifests]\n    needs: [windows-contract, fast, compare-manifests]"), release),
    ]
    for index, (changed_workflow, changed_release) in enumerate(mutants):
        assert (changed_workflow, changed_release) != (workflow, release), index
        try:
            verify(changed_workflow, changed_release)
        except ValueError:
            continue
        raise AssertionError(f"CI boundary mutant {index} unexpectedly passed")
    print(f"Cargo compile/Bazel execution contract passes; {len(mutants)} regressions rejected")


if __name__ == "__main__":
    main()
