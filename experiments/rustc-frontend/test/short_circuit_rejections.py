"""Lazy source boundary: invalid/unsupported input never publishes partial code."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "lazy-rejections"
    root.mkdir()
    cases = {
        "integer": ("if value && true { 1 } else { 0 }", "error[E0308]"),
        "reference": ("let flag = value > 0; let reference = &flag; if reference && true { 1 } else { 0 }", "error[E0308]"),
        "right_block": ("if (value > 0) && { value < 10 } { 1 } else { 0 }", "expression mapping is not implemented"),
        "left_block": ("if { value > 0 } || (value < 10) { 1 } else { 0 }", "expression mapping is not implemented"),
        "mutable": ("let mut flag = value > 0; flag = !flag; if flag && true { 1 } else { 0 }", "only plain immutable bindings are implemented"),
        "bitwise": ("if (value > 0) && ((&(value > 1)) ^ (value < 10)) { 1 } else { 0 }", "eager Boolean input requires"),
    }
    for label, (body, diagnostic) in cases.items():
        source = root / (label + ".rs")
        source.write_text(f"pub fn score(value: i32) -> i32 {{ {body} }}\n")
        for language, adapter, filename in [("c", sys.argv[1], "out.c"), ("java", sys.argv[2], "Generated.java")]:
            directory = root / (label + "-" + language)
            directory.mkdir()
            output = directory / filename
            for existing in [False, True]:
                if existing:
                    output.write_bytes(b"unchanged\x00\xff")
                result = subprocess.run([adapter, str(source), str(output)], capture_output=True, text=True, timeout=60)
                assert result.returncode != 0 and diagnostic in result.stderr, (label, language, result.stderr)
                if not diagnostic.startswith("error["):
                    assert "error[E" not in result.stderr, (label, "must be valid unsupported Rust", result.stderr)
                assert set(directory.iterdir()) == ({output} if existing else set())
                if existing:
                    assert output.read_bytes() == b"unchanged\x00\xff"
    for label, condition in [("and", "(value > 0) && (value < 10)"),
                             ("or", "(value < 0) || (value > 10)"),
                             ("nested", "!((value > 0) && ((value < 10) || (value == 42)))")]:
        source = root / (label + ".rs")
        source.write_text(f"pub fn score(value: i32) -> i32 {{ if {condition} {{ 1 }} else {{ 0 }} }}\n")
        for language, adapter, filename in [("c", sys.argv[1], "out.c"), ("java", sys.argv[2], "Generated.java")]:
            directory = root / (label + "-" + language)
            directory.mkdir()
            output = directory / filename
            result = subprocess.run([adapter, str(source), str(output)], capture_output=True, text=True, timeout=60)
            assert result.returncode == 0 and output.is_file(), (label, language, result.stderr)
    print("Six source boundary cases x two targets x absent/existing outputs reject; six positive controls publish")


if __name__ == "__main__":
    main()
