"""Unsupported valid Rust rejects atomically; ordinary names confer no authority."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "i8": ("pub fn value()->bool { 1i8.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "i16": ("pub fn value()->bool { 1i16.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "i128": ("pub fn value()->bool { 1i128.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "isize": ("pub fn value()->bool { 1isize.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "u32": ("pub fn value()->bool { 1u32.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "u64": ("pub fn value()->bool { 1u64.wrapping_sub(2)==3 }", "supports only exact i32 and i64"),
    "borrowed32": ("pub fn value(a:i32,b:i32)->i32 { (&a).wrapping_sub(b) }", "unadjusted by-value operands"),
    "borrowed64": ("pub fn value(a:i64,b:i64)->i64 { (&a).wrapping_sub(b) }", "unadjusted by-value operands"),
    "cast": ("pub fn value(a:i64,b:i32)->i32 { (a as i32).wrapping_sub(b) }", "expression mapping is not implemented"),
    "ordinary32": ("pub fn value(a:i32,b:i32)->i32 { a-b }", "only comparison binary operators"),
    "ordinary64": ("pub fn value(a:i64,b:i64)->i64 { a-b }", "only comparison binary operators"),
    "div": ("pub fn value(a:i32,b:i32)->i32 { a.wrapping_div(b) }", "expression mapping is not implemented"),
    "mul": ("pub fn value(a:i64,b:i64)->i64 { a.wrapping_mul(b) }", "expression mapping is not implemented"),
    "saturating": ("pub fn value(a:i64,b:i64)->i64 { a.saturating_sub(b) }", "expression mapping is not implemented"),
    "checked": ("pub fn value(a:i32,b:i32)->i32 { a.checked_sub(b).unwrap_or(0) }", "expression mapping is not implemented"),
    "generic": ("fn identity<T>(v:T)->T {v} pub fn value(a:i64,b:i64)->i64 { identity(a).wrapping_sub(b) }", "generic or mismatched direct callee identity"),
    "trait": ("struct Fake {v:i32} trait Subtract {fn wrapping_sub(self,b:i32)->i32;} impl Subtract for Fake {fn wrapping_sub(self,_b:i32)->i32 {self.v}} pub fn value(a:i32,b:i32)->i32 {Fake{v:a}.wrapping_sub(b)}", "only the standard primitive wrapping_sub"),
    "inherent": ("struct Fake {v:i32} impl Fake {fn wrapping_sub(self,_b:i32)->i32 {self.v}} pub fn value(a:i32,b:i32)->i32 {Fake{v:a}.wrapping_sub(b)}", "only the standard primitive wrapping_sub"),
}


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "subtraction-rejections"
    root.mkdir()
    for label, (code, diagnostic) in CASES.items():
        source = root / (label + ".rs")
        source.write_text(code + "\n")
        for language, adapter in zip(["c", "java"], sys.argv[1:], strict=True):
            for existing in [False, True]:
                directory = root / (label + language + str(existing))
                directory.mkdir()
                output = directory / ("package" if language == "c" else "Generated.java")
                if existing:
                    if language == "c":
                        output.mkdir()
                    sentinel = output / "sentinel" if language == "c" else output
                    sentinel.write_bytes(b"preserved\x00\xff")
                before = {str(p.relative_to(directory)): p.read_bytes() for p in directory.rglob("*") if p.is_file()}
                result = subprocess.run([adapter, source, output, "--package"], capture_output=True, text=True, timeout=90)
                assert result.returncode != 0 and diagnostic in result.stderr, (label, language, result.stderr)
                assert "error[E" not in result.stderr and "panicked at" not in result.stderr, (label, result.stderr)
                after = {str(p.relative_to(directory)): p.read_bytes() for p in directory.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    print(f"{4 * len(CASES)} atomic unsupported wrapping-subtraction rejections")


if __name__ == "__main__":
    main()
