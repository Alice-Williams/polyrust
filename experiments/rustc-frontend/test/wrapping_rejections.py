"""Valid unsupported Rust rejects atomically; names alone never admit a method."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "checked_neg": ("pub fn value(v: i32) -> i32 { v.abs() }", "expression mapping is not implemented"),
    "extra_argument": ("pub fn value(v: i32) -> i32 { v.wrapping_add(1) }", "expression mapping is not implemented"),
    "lookalike_extra_argument": ("trait Fake { fn wrapping_neg(self, extra: i32) -> i32; } impl Fake for bool { fn wrapping_neg(self, extra: i32) -> i32 { extra } } pub fn value(v: bool) -> i32 { v.wrapping_neg(1) }", "wrapping negation takes no extra arguments"),
    "borrow_adjustment": ("pub fn value(v: i32) -> i32 { (&v).wrapping_neg() }", "nongeneric unadjusted by-value"),
    "trait_method": ("trait Fake { fn wrapping_neg(self) -> i32; } impl Fake for bool { fn wrapping_neg(self) -> i32 { 0 } } pub fn value(v: bool) -> i32 { v.wrapping_neg() }", "only the standard primitive wrapping_neg"),
    "trait_qualified": ("trait Fake { fn wrapping_neg(self) -> i32; } impl Fake for bool { fn wrapping_neg(self) -> i32 { 0 } } pub fn value(v: bool) -> i32 { <bool as Fake>::wrapping_neg(v) }", "only the standard primitive wrapping_neg"),
    "inherent_lookalike": ("struct Fake { value: i32 } impl Fake { fn wrapping_neg(self) -> i32 { self.value } } pub fn value(v: i32) -> i32 { Fake { value: v }.wrapping_neg() }", "only the standard primitive wrapping_neg"),
    "indirect": ("pub fn value(v: i32) -> i32 { let f: fn(i32) -> i32 = i32::wrapping_neg; f(v) }", "direct calls require resolved ordinary functions"),
    "generic": ("fn forward<T>(v: T) -> T { v } pub fn value(v: i32) -> i32 { forward(v).wrapping_neg() }", "generic or mismatched direct callee identity"),
    "unsafe": ("unsafe fn forward(v: i32) -> i32 { v } pub fn value(v: i32) -> i32 { unsafe { forward(v) }.wrapping_neg() }", "declaration of an `unsafe` function"),
    "ordinary_minus": ("pub fn value(v: i32) -> i32 { -v }", "only negative scalar literals"),
}
for WIDTH in ["i8", "i16", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize"]:
    CASES[WIDTH] = (f"pub fn value(v: {WIDTH}) -> {WIDTH} {{ v.wrapping_neg() }}",
                    "wrapping negation supports only exact i32 and i64")


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "wrapping-rejections"
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
