#!/usr/bin/env python3
"""Release dependency, toolchain, evidence, and generated-source policies."""

from __future__ import annotations

import re
import sys
import tempfile
import tomllib
from pathlib import Path


ALLOWED_REGISTRY = {
    "itoa": "1.0.18",
    "memchr": "2.8.3",
    "proc-macro2": "1.0.107",
    "quote": "1.0.47",
    "serde": "1.0.229",
    "serde_core": "1.0.229",
    "serde_derive": "1.0.229",
    "serde_json": "1.0.151",
    "syn": "3.0.4",
    "unicode-ident": "1.0.24",
    "zmij": "1.0.23",
}

PINNED_TEXT = {
    ".devcontainer/Dockerfile": [
        "FROM rust:1.98.0-trixie",
        "ARG BAZELISK_VERSION=1.29.0",
        "ARG NODE_VERSION=24.20.0",
        "ARG TYPESCRIPT_VERSION=7.0.2",
        "ARG PRETTIER_VERSION=3.9.6",
        "ARG RUFF_VERSION=0.16.5",
        "ARG MYPY_VERSION=2.3.1",
        "ARG PYTEST_VERSION=9.1.1",
    ],
    "MODULE.bazel": [
        'version = "0.74.0"',
        'version = "0.63.0"',
        'version = "1.25.14"',
        'versions = ["1.98.0"]',
    ],
    "docs/dependencies.md": ["Apache-2.0", "MIT", "BSD-3-Clause", "PSF-2.0"],
}


class PolicyError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise PolicyError(message)


def verify_lock(lock: Path) -> None:
    packages = tomllib.loads(lock.read_text(encoding="utf-8"))["package"]
    registry = {
        package["name"]: package["version"]
        for package in packages
        if str(package.get("source", "")).startswith("registry+")
    }
    require(registry == ALLOWED_REGISTRY, f"dependency policy violation: {registry}")


def verify_safety(rust_sources: list[Path], go_sources: list[Path]) -> None:
    unsafe_lines = []
    for source in rust_sources:
        unsafe_lines.extend(
            line.strip()
            for line in source.read_text(encoding="utf-8").splitlines()
            if "unsafe" in line
        )
    require(
        unsafe_lines and set(unsafe_lines) == {"#![forbid(unsafe_code)]"},
        f"unsafe generated Rust surface: {unsafe_lines}",
    )
    forbidden = re.compile(r'"(?:unsafe|reflect)"')
    offenders = [str(path) for path in go_sources if forbidden.search(path.read_text(encoding="utf-8"))]
    require(not offenders, f"unsafe generated Go imports: {offenders}")


def verify_evidence(evidence: dict[str, object]) -> None:
    require(evidence.get("snapshot_equal") is True, "snapshot drift")
    require(evidence.get("conformance") is True, "conformance failure")
    tools = set(evidence.get("formatters", []))
    require({"rustfmt", "prettier", "ruff", "gofmt"} <= tools, "missing formatter")
    backends = set(evidence.get("backends", []))
    require({"rust", "typescript", "python", "go"} <= backends, "required backend skipped")


def generated_sources(root: Path, target: str, suffix: str) -> list[Path]:
    return [
        path
        for path in root.rglob(f"*{suffix}")
        if "models-and-validation/generated" in path.as_posix() and f"/generated/{target}/" in path.as_posix()
    ]


def verify(root: Path) -> None:
    verify_lock(root / "Cargo.lock")
    for relative, required in PINNED_TEXT.items():
        contents = (root / relative).read_text(encoding="utf-8")
        for text in required:
            require(text in contents, f"missing pinned policy text in {relative}: {text}")
    rust_sources = generated_sources(root, "rust", ".rs")
    go_sources = generated_sources(root, "go", ".go")
    require(rust_sources and go_sources, "required generated safety inputs missing")
    verify_safety(rust_sources, go_sources)
    verify_evidence(
        {
            "snapshot_equal": True,
            "conformance": True,
            "formatters": ["rustfmt", "prettier", "ruff", "gofmt"],
            "backends": ["rust", "typescript", "python", "go"],
        }
    )


def expect_failure(name: str, action) -> None:
    try:
        action()
    except PolicyError:
        print(f"detected deliberate {name}")
        return
    raise AssertionError(f"deliberate {name} was not detected")


def self_test() -> None:
    good = {
        "snapshot_equal": True,
        "conformance": True,
        "formatters": ["rustfmt", "prettier", "ruff", "gofmt"],
        "backends": ["rust", "typescript", "python", "go"],
    }
    for name, change in [
        ("snapshot drift", {"snapshot_equal": False}),
        ("conformance failure", {"conformance": False}),
        ("missing formatter", {"formatters": ["rustfmt", "ruff", "gofmt"]}),
        ("skipped Go", {"backends": ["rust", "typescript", "python"]}),
    ]:
        evidence = {**good, **change}
        expect_failure(name, lambda evidence=evidence: verify_evidence(evidence))
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        rust = root / "generated.rs"
        go = root / "generated.go"
        rust.write_text("#![forbid(unsafe_code)]\npub unsafe fn injected() {}\n", encoding="utf-8")
        go.write_text('package injected\nimport "unsafe"\n', encoding="utf-8")
        expect_failure("unsafe generated source", lambda: verify_safety([rust], [go]))
        lock = root / "Cargo.lock"
        lock.write_text(
            'version = 4\n\n[[package]]\nname = "forbidden"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n',
            encoding="utf-8",
        )
        expect_failure("dependency policy violation", lambda: verify_lock(lock))


def main() -> int:
    try:
        if sys.argv[1:] == ["self-test"]:
            self_test()
        elif len(sys.argv) == 3 and sys.argv[1] == "verify":
            verify(Path(sys.argv[2]).resolve())
            print("dependency, toolchain, release evidence, and safety policies passed")
        else:
            raise SystemExit("usage: policy.py verify ROOT | self-test")
    except PolicyError as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

