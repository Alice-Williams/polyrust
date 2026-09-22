"""Unsupported valid Rust casts fail without creating/replacing output."""
import os
from pathlib import Path
import subprocess
import sys

CAST = "signed widening supports only an unadjusted i32 operand cast to i64"
CASES = {
    "narrow": ("pub fn value(a:i64)->i32 {a as i32}", CAST),
    "same32": ("pub fn value(a:i32)->i32 {a as i32}", CAST),
    "same64": ("pub fn value(a:i64)->i64 {a as i64}", CAST),
    "float": ("pub fn value(a:f64)->i64 {a as i64}", CAST),
    "float_output": ("pub fn value(a:i32)->f64 {a as f64}", CAST),
    "bool": ("pub fn value(a:bool)->i64 {a as i64}", CAST),
    "u32": ("pub fn value()->bool {(4294967295u32 as i64)==0}", CAST),
    "u64": ("pub fn value()->bool {(1u64 as i64)==0}", CAST),
    "i16": ("pub fn value()->bool {(1i16 as i64)==0}", CAST),
    "i128": ("pub fn value()->bool {(1i128 as i64)==0}", CAST),
    "usize": ("pub fn value()->bool {(1usize as i64)==0}", CAST),
    "isize": ("pub fn value()->bool {(1isize as i64)==0}", CAST),
    "char": ("pub fn value()->bool {('a' as i64)==0}", CAST),
    "unsigned_output": ("pub fn value(a:i32)->bool {(a as u64)==0}", CAST),
    "alias_input": ("type Narrow=i32; pub fn value(a:Narrow)->i64 {a as i64}", "type alias uses require"),
    "alias_output": ("type Wide=i64; pub fn value(a:i32)->Wide {a as Wide}", "type alias uses require"),
    "from": ("pub fn value(a:i32)->i64 {i64::from(a)}", "direct calls require resolved ordinary functions"),
    "into": ("pub fn value(a:i32)->i64 {a.into()}", "expression mapping is not implemented"),
}


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "widening-rejections"
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
    print(f"{4 * len(CASES)} atomic unsupported widening rejections")


if __name__ == "__main__":
    main()
