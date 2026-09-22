"""Valid-but-unsupported unary neighbors fail before publishing any output."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    c, java = sys.argv[1:]
    root = Path(os.environ["TEST_TMPDIR"]) / "floating-rejections"
    root.mkdir()
    cases = {
        "f32": ("pub fn value(v:f32)->f32 {-v}", "signatures support only"),
        "reference": ("pub fn value(v:f64)->f64 {-&v}", "floating negation requires an unadjusted built-in f64"),
        "overloaded": ("struct V; impl std::ops::Neg for V {type Output=f64; fn neg(self)->f64 {0.0}} pub fn value()->f64 {-V}", "floating negation requires an unadjusted built-in f64"),
        "method": ("use std::ops::Neg; pub fn value(v:f64)->f64 {v.neg()}", "expression mapping is not implemented"),
        "integer": ("pub fn value(v:i64)->i64 {-v}", "only negative scalar literals"),
        "cast": ("pub fn value(v:i64)->f64 {-(v as f64)}", "signed widening supports only an unadjusted i32 operand cast to i64"),
        "euclidean_remainder": ("pub fn value(v:f64)->f64 {-(v.rem_euclid(1.0))}", "expression mapping is not implemented"),
        "nonfinite": ("#![allow(overflowing_literals)]\npub fn value()->f64 {-1e400}", "nonfinite f64 literals"),
        "constant": ("const V:f64=f64::NAN; pub fn value()->f64 {-V}", "NaN f64 constants"),
    }
    for label, (text, diagnostic) in cases.items():
        source = root / (label + ".rs")
        source.write_text(text + "\n")
        for language, adapter in [("c", c), ("java", java)]:
            for existing in [False, True]:
                work = root / (label + language + str(existing))
                work.mkdir()
                output = work / ("package" if language == "c" else "Generated.java")
                if existing:
                    if language == "c":
                        output.mkdir()
                    sentinel = output / "sentinel" if language == "c" else output
                    sentinel.write_bytes(b"preserved\x00\xff")
                before = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                result = subprocess.run([adapter, source, output, "--package"], capture_output=True, text=True, timeout=90)
                assert result.returncode != 0 and diagnostic in result.stderr, (label, language, result.stderr)
                assert "error[E" not in result.stderr, (label, "must be valid Rust", result.stderr)
                after = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    print("36 atomic floating-negation boundary rejections across C and Java")


if __name__ == "__main__":
    main()
