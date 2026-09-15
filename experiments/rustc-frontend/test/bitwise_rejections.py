"""Unsupported operations and widths must not publish partial packages."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "bitwise-rejections"
    root.mkdir()
    cases = {
        "u32": ("pub fn value(v: u32) -> u32 { !v }", "only i32, i64 and bool", True),
        "i16": ("pub fn value(v: i16) -> i16 { !v }", "only i32, i64 and bool", True),
        "shift": ("pub fn value(v: i64) -> i64 { v << 1 }", "only comparison binary operators", True),
        "cast": ("pub fn value(v: i64) -> i64 { (v as i32) as i64 }", "expression mapping is not implemented", True),
        "bool_and": ("pub fn value(a: bool,b: bool) -> bool { a & b }", "integer bitwise requires", True),
        "bool_or": ("pub fn value(a: bool,b: bool) -> bool { a | b }", "integer bitwise requires", True),
        "bool_xor": ("pub fn value(a: bool,b: bool) -> bool { a ^ b }", "integer bitwise requires", True),
        "mixed": ("pub fn value(a: i32,b: i64) -> i64 { a & b }", "error[E", False),
        "ref": ("pub fn value(a: i64) -> i64 { let r = &a; !r }", "integer bitwise requires", True),
        "arithmetic": ("pub fn value(a: i64) -> i64 { a + 1 }", "only comparison binary operators", True),
    }
    for label, (code, diagnostic, valid) in cases.items():
        source = root / (label + ".rs")
        source.write_text(code + "\n")
        for lang, adapter in zip(["c", "java"], sys.argv[1:], strict=True):
            for existing in [False, True]:
                work = root / (label + lang + str(existing))
                work.mkdir()
                output = work / ("package" if lang == "c" else "Generated.java")
                if existing:
                    if lang == "c":
                        output.mkdir()
                    sentinel = output / "sentinel" if lang == "c" else output
                    sentinel.write_bytes(b"preserved\x00\xff")
                before = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                result = subprocess.run([adapter, source, output, "--package"], capture_output=True, text=True, timeout=90)
                assert result.returncode != 0 and diagnostic in result.stderr, (label, lang, result.stderr)
                if valid:
                    assert "error[E" not in result.stderr, (label, result.stderr)
                after = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    print("40 atomic bitwise boundary rejections")


if __name__ == "__main__":
    main()
