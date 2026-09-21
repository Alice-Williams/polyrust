"""Unsupported valid Rust fails before any output or existing file changes."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "f32": ("pub fn value(a:f32,b:f32)->f32 {a%b}", "signatures support only"),
    "i32": ("pub fn value(a:i32,b:i32)->i32 {a%b}", "only comparison binary operators"),
    "i64": ("pub fn value(a:i64,b:i64)->i64 {a%b}", "only comparison binary operators"),
    "borrowed": ("pub fn value(a:f64,b:f64)->f64 {&a%&b}", "unadjusted built-in f64"),
    "overloaded": ("struct Fake {value:f64} impl std::ops::Rem for Fake {type Output=f64; fn rem(self,rhs:Self)->f64 {self.value%rhs.value}} pub fn value(a:f64,b:f64)->f64 {Fake{value:a}%Fake{value:b}}", "unadjusted built-in f64"),
    "cast": ("pub fn value(a:i64,b:f64)->f64 {(a as f64)%b}", "expression mapping is not implemented"),
    "assignment": ("pub fn value(mut a:f64,b:f64)->f64 {a%=b;a}", "only plain immutable parameters"),
    "euclidean": ("pub fn value(a:f64,b:f64)->f64 {a.rem_euclid(b)}", "expression mapping is not implemented"),
    "constant": ("const A:f64=1.0; pub fn value(b:f64)->f64 {A%b}", "constants support only"),
    "generic": ("fn identity<T>(v:T)->T {v} pub fn value(a:f64,b:f64)->f64 {identity(a)%b}", "generic or mismatched direct callee identity"),
}


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "remainder-rejections"
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
    print(f"{4 * len(CASES)} atomic unsupported remainder rejections")


if __name__ == "__main__":
    main()
