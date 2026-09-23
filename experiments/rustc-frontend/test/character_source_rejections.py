"""Character admission has explicit boundaries and atomic failure behavior."""
import os
from pathlib import Path
import subprocess
import sys


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "character-rejections"
    root.mkdir()
    cases = {
        "trait_constant": ("struct S; trait T{const V:char;} impl T for S{const V:char='x';} pub fn value()->char{<S as T>::V}", "scalar constants require nongeneric", True),
        "concrete_generic_constant": ("struct S<const N:usize>; impl S<1>{const V:char='x';} pub fn value()->char{S::<1>::V}", "scalar constants require nongeneric nominal", True),
        "constant_type_alias": ("type C=char; pub const V:C='x';", "Rust type alias uses require an unimplemented provenance mapping", True),
        "local_constant_type_alias": ("type C=char; pub fn value()->char{const V:C='x'; V}", "Rust type alias uses require an unimplemented provenance mapping", True),
        "invalid_constant_surrogate": ("pub const V:char=char::from_u32(0xd800).unwrap();", "error[E0080]", True),
        "invalid_constant_above_maximum": ("pub const V:char=char::from_u32(0x110000).unwrap();", "error[E0080]", True),
        "borrowed_constant": ("pub const V:&char=&'x';", "scalar constants support", True),
        "generic_constant": ("struct S<const N:usize>; impl<const N:usize>S<N>{const V:char='x';} pub fn value()->char{S::<1>::V}", "scalar constants require nongeneric", True),
        "cast_from_char": ("pub fn value(v:char)->i64{v as i64}", "signed widening supports only", True),
        "cast_to_char": ("pub fn value()->char{65u8 as char}", "signed widening supports only", True),
        "method": ("pub fn value(v:char)->bool{v.is_ascii()}", "not implemented", True),
        "reference": ("pub fn value(v:char)->char{let r=&v; *r}", "character references", True),
        "char_entry": ("pub fn score(v:char)->char{v}", "entry signature must be fn(i32) -> i32", False),
        "mixed_entry": ("pub fn score(_:char)->i32{0}", "entry signature must be fn(i32) -> i32", False),
        "mixed_result": ("pub fn score(_:i32)->char{'x'}", "entry signature must be fn(i32) -> i32", False),
        "surrogate": (r"pub fn value()->char{'\u{d800}'}", "invalid unicode character escape", True),
        "above_maximum": (r"pub fn value()->char{'\u{110000}'}", "invalid unicode character escape", True),
        "multiple": ("pub fn value()->char{'ab'}", "character literal may only contain one codepoint", True),
    }
    for language, adapter in zip(["c", "java"], sys.argv[1:], strict=True):
        positive = root / (language + "-positive.rs")
        positive.write_text("pub fn value(v:char)->char{v}\n")
        positive_directory = root / (language + "-positive")
        positive_directory.mkdir()
        positive_out = positive_directory / ("package" if language == "c" else "Generated.java")
        result = subprocess.run([adapter, positive, positive_out, "--package"],
                                capture_output=True, text=True, timeout=90)
        assert result.returncode == 0 and positive_out.exists(), result.stderr
        for label, (code, diagnostic, package) in cases.items():
            source = root / f"{language}-{label}.rs"
            source.write_text(code + "\n")
            for existing in [False, True]:
                work = root / f"{language}-{label}-{existing}"
                work.mkdir()
                output = work / ("Generated.java" if language == "java" else "package" if package else "output")
                if existing:
                    if language == "c" and package:
                        output.mkdir()
                        (output / "sentinel").write_bytes(b"preserved\x00\xff")
                    else:
                        output.write_bytes(b"preserved\x00\xff")
                before = files(work)
                result = subprocess.run([adapter, source, output, *(["--package"] if package else [])],
                                        capture_output=True, text=True, timeout=90)
                assert result.returncode != 0 and diagnostic in result.stderr, (language, label, result.stderr)
                if label not in ["surrogate", "above_maximum", "multiple", "invalid_constant_surrogate", "invalid_constant_above_maximum"]:
                    assert "error[E" not in result.stderr, (label, result.stderr)
                assert files(work) == before and output.exists() == existing
    print(f"{len(cases) * 4} atomic character boundaries; positive source controls; "
          "unsupported constant forms/casts/methods/refs/entry shapes rejected; invalid scalars rejected by rustc")


if __name__ == "__main__":
    main()
