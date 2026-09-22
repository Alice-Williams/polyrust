"""Unsupported operations and widths must not publish partial packages."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "constant-rejections"
    root.mkdir()
    cases = {
        "foreign_public_read": ("#[allow(deprecated)] pub fn value()->i32 { std::i32::MAX }",
                                "foreign public constant reads require a certified producer mapping", True),
        "public_u64": ("pub const VALUE:u64=9007199254740993;",
                       "scalar constants support only bool, i32, i64 and finite f64", True),
        "public_alias_borrowed": ("mod inner { pub const VALUE:&str=\"text\"; } pub use inner::VALUE;",
                                 "scalar constants support only bool, i32, i64 and finite f64", True),
        "public_char": ("pub const VALUE:char='x'; pub fn value()->i32 { 4 }",
                        "scalar constants support only bool, i32, i64 and finite f64", True),
        "public_f32": ("pub const VALUE:f32=4.0; pub fn value()->i32 { 4 }",
                         "scalar constants support only bool, i32, i64 and finite f64", True),
        "local_unsupported": ("pub fn value()->i32 { const VALUE:u32=4; 0 }", "scalar constants support only bool, i32, i64 and finite f64", True),
        "generic": ("struct S<const N:i32>; impl<const N:i32> S<N> { const VALUE:i32=N; } pub fn value()->i32 { S::<4>::VALUE }",
                    "scalar constants require nongeneric", True),
        "trait": ("struct S; trait T {const VALUE:i32;} impl T for S {const VALUE:i32=4;} pub fn value()->i32 { <S as T>::VALUE }",
                  "scalar constants require nongeneric", True),
        "concrete_const": ("struct S<const N:i32>; impl S<4>{const VALUE:i32=5;} pub fn value()->i32{S::<4>::VALUE}",
                           "scalar constants require nongeneric nominal", True),
        "default_const": ("struct S<const N:i32=4>; impl S<4>{const VALUE:i32=5;} pub fn value()->i32{S::VALUE}",
                          "scalar constants require nongeneric nominal", True),
        "concrete_type": ("struct S<T>(std::marker::PhantomData<T>); impl S<i64>{const VALUE:i32=6;} pub fn value()->i32{S::<i64>::VALUE}",
                          "scalar constants require nongeneric nominal", True),
        "default_type": ("struct S<T=i64>(std::marker::PhantomData<T>); impl S<i64>{const VALUE:i32=6;} pub fn value()->i32{S::VALUE}",
                         "scalar constants require nongeneric nominal", True),
        "borrow": ("const VALUE:i32=4; pub fn value()->i32 {let r=&VALUE; *r}",
                   "only resolved local value paths", True),
        "static": ("static VALUE:i32=4; pub fn value()->i32 { VALUE }",
                   "only resolved local value paths", True),
        "u32": ("const VALUE:u32=4; pub fn value()->u32 { VALUE }", "only i32, i64, bool and f64", True),
        "arithmetic": ("const VALUE:i32=7*9; pub fn value()->i32 { VALUE+1 }",
                       "only comparison binary operators", True),
        "overflow": ("const VALUE:i32=i32::MAX+1; pub fn value()->i32 { VALUE }", "error[E0080]", False),
        "panic": ('const VALUE:i32=panic!("bad"); pub fn value()->i32 { VALUE }', "error[E0080]", False),
        "allow_limit": ("#![allow(long_running_const_eval)]\nconst VALUE:i32=4; pub fn value()->i32 { VALUE }",
                        "error[E0453]", False),
        "endless": ("const VALUE:i32=loop {}; pub fn value()->i32 { VALUE }",
                    "constant evaluation is taking a long time", False),
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
                expected = diagnostic[lang == "java"] if isinstance(diagnostic, tuple) else diagnostic
                assert result.returncode != 0 and expected in result.stderr, (label, lang, result.stderr)
                if valid:
                    assert "error[E" not in result.stderr, (label, result.stderr)
                after = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    print(f"{len(cases) * 4} atomic constant boundary rejections")


if __name__ == "__main__":
    main()
