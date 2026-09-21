"""Valid unsupported Rust rejects atomically; names alone never admit a method."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "f32": ("pub fn value(v:f32)->bool {v.is_nan()}", "NaN classification supports only exact f64"),
    "borrow_adjustment": ("pub fn value(v:f64)->bool {(&v).is_nan()}", "nongeneric unadjusted by-value"),
    "trait_method": ("trait Fake { fn is_nan(self)->bool; } impl Fake for bool { fn is_nan(self)->bool {self} } pub fn value(v:bool)->bool {v.is_nan()}", "only the standard primitive is_nan"),
    "trait_qualified": ("trait Fake { fn is_nan(self)->bool; } impl Fake for bool { fn is_nan(self)->bool {self} } pub fn value(v:bool)->bool {<bool as Fake>::is_nan(v)}", "only the standard primitive is_nan"),
    "inherent_lookalike": ("struct Fake {value:bool} impl Fake {fn is_nan(self)->bool {self.value}} pub fn value(v:bool)->bool {Fake {value:v}.is_nan()}", "only the standard primitive is_nan"),
    "extra_argument": ("trait Fake {fn is_nan(self, extra:bool)->bool;} impl Fake for bool {fn is_nan(self, extra:bool)->bool {extra}} pub fn value(v:bool)->bool {v.is_nan(true)}", "NaN classification takes no extra arguments"),
    "other_method": ("pub fn value(v:f64)->bool {v.is_infinite()}", "expression mapping is not implemented"),
    "cast": ("pub fn value(v:i64)->bool {(v as f64).is_nan()}", "expression mapping is not implemented"),
    "constant": ("const V:f64=1.0; pub fn value()->bool {V.is_nan()}", "constants support only"),
    "euclidean_remainder": ("pub fn value(v:f64)->bool {(v.rem_euclid(1.0)).is_nan()}", "expression mapping is not implemented"),
    "indirect": ("pub fn value(v:f64)->bool {let f:fn(f64)->bool=f64::is_nan; f(v)}", "direct calls require resolved ordinary functions"),
    "generic": ("fn forward<T>(v:T)->T {v} pub fn value(v:f64)->bool {forward(v).is_nan()}", "generic or mismatched direct callee identity"),
}

def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "nan-rejections"
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
