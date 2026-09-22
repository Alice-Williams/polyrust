"""Valid unsupported Rust rejects atomically; names alone never admit a method."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "f32": ("pub fn value(v:f32)->f32 {v.trunc()}", "Floating truncation supports only exact f64"),
    "borrow_adjustment": ("pub fn value(v:f64)->f64 {(&v).trunc()}", "nongeneric unadjusted by-value"),
    "trait_method": ("trait Fake { fn trunc(self)->bool; } impl Fake for bool { fn trunc(self)->bool {self} } pub fn value(v:bool)->bool {v.trunc()}", "only the standard primitive trunc"),
    "trait_qualified": ("trait Fake { fn trunc(self)->bool; } impl Fake for bool { fn trunc(self)->bool {self} } pub fn value(v:bool)->bool {<bool as Fake>::trunc(v)}", "only the standard primitive trunc"),
    "inherent_lookalike": ("struct Fake {value:bool} impl Fake {fn trunc(self)->bool {self.value}} pub fn value(v:bool)->bool {Fake {value:v}.trunc()}", "only the standard primitive trunc"),
    "extra_argument": ("trait Fake {fn trunc(self, extra:bool)->bool;} impl Fake for bool {fn trunc(self, extra:bool)->bool {extra}} pub fn value(v:bool)->bool {v.trunc(true)}", "Floating truncation takes no extra arguments"),
    "other_method": ("pub fn value(v:f64)->f64 {v.floor()}", "expression mapping is not implemented"),
    "cast": ("pub fn value(v:i64)->f64 {(v as f64).trunc()}", "signed widening supports only an unadjusted i32 operand cast to i64"),
    "constant": ("const V:f64=f64::INFINITY; pub fn value()->f64 {V.trunc()}", "nonfinite f64 constants"),
    "euclidean_remainder": ("pub fn value(v:f64)->f64 {(v.rem_euclid(1.0)).trunc()}", "expression mapping is not implemented"),
    "indirect": ("pub fn value(v:f64)->f64 {let f:fn(f64)->f64=f64::trunc; f(v)}", "direct calls require resolved ordinary functions"),
    "generic": ("fn forward<T>(v:T)->T {v} pub fn value(v:f64)->f64 {forward(v).trunc()}", "generic or mismatched direct callee identity"),
}

def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "truncation-rejections"
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
    print(f"{4 * len(CASES)} atomic unsupported-method/width/signature/adjustment rejections")


if __name__ == "__main__":
    main()
