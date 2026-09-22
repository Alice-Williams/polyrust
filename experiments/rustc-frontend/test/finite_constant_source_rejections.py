"""Finite source admission does not expand unsupported storage or evaluation."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    scalar = "scalar constants support only bool, i32, i64 and non-NaN f64"
    nonfinite = "NaN f64 constants are not implemented"
    cases = {
        "positive_payload_nan": ("pub const V:f64=f64::from_bits(0x7ff8000000000001);", nonfinite),
        "negative_payload_nan": ("pub const V:f64=f64::from_bits(0xfff8000000000001);", nonfinite),
        "nan": ("pub const V:f64=f64::NAN;", nonfinite),
        "signalling_nan": ("pub const V:f64=f64::from_bits(0x7ff0000000000001);", nonfinite),
        "negative_nan": ("pub const V:f64=f64::from_bits(0xfff8000000000001);", nonfinite),
        "infinity_difference_nan": ("pub const V:f64=f64::INFINITY-f64::INFINITY;", nonfinite),
        "nonfinite_private": ("const V:f64=f64::NAN; pub fn value()->f64{V}", nonfinite),
        "nan_unused_local": ("pub fn value()->f64{const V:f64=f64::NAN; 0.0}", nonfinite),
        "nonfinite_alias": ("mod m{pub const V:f64=f64::NAN;} pub use m::V;", nonfinite),
        "public_f32": ("pub const V:f32=1.0;", scalar),
        "unused_local_f32": ("pub fn value()->f64{const V:f32=1.0; 0.0}", scalar),
        "borrowed_storage": ("pub const V:&f64=&1.0;", scalar),
        "borrowed_read": ("const V:f64=1.0; pub fn value()->f64{let r=&V; *r}", "only resolved local value paths"),
        "type_alias": ("type F=f64; pub const V:F=1.0;", "Rust type alias uses require an unimplemented provenance mapping"),
        "local_type_alias": ("type F=f64; pub fn value()->f64{const V:F=1.0; V}", "Rust type alias uses require an unimplemented provenance mapping"),
        "generic_owner": ("struct S<const N:usize>; impl<const N:usize>S<N>{const V:f64=1.0;} pub fn value()->f64{S::<1>::V}", "scalar constants require nongeneric"),
        "concrete_generic_owner": ("struct S<const N:usize>; impl S<1>{const V:f64=1.0;} pub fn value()->f64{S::<1>::V}", "scalar constants require nongeneric nominal"),
        "trait_owner": ("struct S; trait T{const V:f64;} impl T for S{const V:f64=1.0;} pub fn value()->f64{<S as T>::V}", "scalar constants require nongeneric"),
        "static": ("static V:f64=1.0; pub fn value()->f64{V}", "only resolved local value paths"),
        "runtime_index": ("pub fn value()->f64{[0.25, 0.5][1]}", "expression mapping is not implemented"),
    }
    root = Path(os.environ["TEST_TMPDIR"]) / "finite-constant-rejections"
    root.mkdir()
    for label, (code, diagnostic) in cases.items():
        source = root / (label + ".rs")
        source.write_text(code + "\n")
        for language, adapter in zip(["c", "java"], sys.argv[1:], strict=True):
            for existing in [False, True]:
                work = root / (label + language + str(existing))
                work.mkdir()
                output = work / ("package" if language == "c" else "Generated.java")
                if existing:
                    if language == "c":
                        output.mkdir()
                    sentinel = output / "sentinel" if language == "c" else output
                    sentinel.write_bytes(b"preserved\x00\xff")
                before = {p.relative_to(work).as_posix(): p.read_bytes()
                          for p in work.rglob("*") if p.is_file()}
                result = subprocess.run([str(adapter), source, output, "--package"],
                                        capture_output=True, text=True, timeout=90)
                assert result.returncode != 0 and diagnostic in result.stderr, (label, language, result.stderr)
                assert "error[E" not in result.stderr, (label, result.stderr)
                after = {p.relative_to(work).as_posix(): p.read_bytes()
                         for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    print(f"{len(cases) * 4} atomic finite-constant source boundary controls")


if __name__ == "__main__":
    main()
