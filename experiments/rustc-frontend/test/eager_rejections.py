"""Unsupported operations and widths must not publish partial packages."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "eager-rejections"
    root.mkdir()
    cases = {
        "ref_and": ("pub fn value(a: bool,b: bool) -> bool { (&a) & b }", "eager Boolean input requires", True),
        "ref_or": ("pub fn value(a: bool,b: bool) -> bool { a | (&b) }", "eager Boolean input requires", True),
        "ref_xor": ("pub fn value(a: bool,b: bool) -> bool { (&a) ^ (&b) }", "eager Boolean input requires", True),
        "mixed": ("pub fn value(a: bool,b: i32) -> bool { a & b }", "error[E", False),
        "cast": ("pub fn value(a: bool,b: bool) -> bool { (a as i32 & b as i32) != 0 }", "expression mapping is not implemented", True),
        "block_left": ("pub fn value(a: bool,b: bool) -> bool { ({ a }) & b }", "expression mapping is not implemented", True),
        "block_right": ("pub fn value(a: bool,b: bool) -> bool { a | { b } }", "expression mapping is not implemented", True),
        "write": ("pub fn value(a: bool,b: bool) -> bool { let mut c=a; c &= b; c }", "only plain immutable bindings are implemented", True),
        "u32": ("pub fn value(a: u32,b: u32) -> u32 { a & b }", "only i32, i64 and bool", True),
        "arithmetic": ("pub fn value(a: i32,b: i32) -> bool { a + b > 0 }", "only comparison binary operators", True),
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
    print("40 atomic eager boundary rejections")


if __name__ == "__main__":
    main()
