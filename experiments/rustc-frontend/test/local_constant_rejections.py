"""Unsupported operations and widths must not publish partial packages."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "local-constant-rejections"
    root.mkdir()
    scalar = "scalar constants support only bool, char, i32, i64 and non-NaN f64"
    item = "only scalar const item statements are implemented"
    cases = {
        "unused_u32": ("pub fn value()->i32 { const VALUE:u32=4; 0 }", scalar, True),
        "unused_array": ("pub fn value()->i32 { const VALUE:[i32;2]=[1,2]; 0 }", scalar, True),
        "unused_ref": ("pub fn value()->i32 { const VALUE:&i32=&4; 0 }", scalar, True),
        "unused_f32": ("pub fn value()->i32 { const VALUE:f32=4.0; 0 }", scalar, True),
        "borrow": ("pub fn value()->i32 { const VALUE:i32=4; let r=&VALUE; *r }", "only resolved local value paths", True),
        "static": ("pub fn value()->i32 { static VALUE:i32=4; 0 }", item, True),
        "type": ("pub fn value()->i32 { type Value=i32; 0 }", item, True),
        "struct": ("pub fn value()->i32 { struct Value; 0 }", item, True),
        "function": ("pub fn value()->i32 { fn other()->i32{4} 0 }", item, True),
        "module": ("pub fn value()->i32 { mod other{pub const V:i32=4;} 0 }", item, True),
        "use": ("const OTHER:i32=3; pub fn value()->i32 { use crate::OTHER; 0 }", item, True),
        "alias": ("type Alias=i32; pub fn value()->i32 { const V:Alias=4; 0 }", "Rust type alias uses require an unimplemented provenance mapping", True),
        "duplicate": ("pub fn value()->i32 { const V:i32=1; const V:i32=2; V }", "error[E0428]", False),
        "capture": ("pub fn value(input:i32)->i32 { const V:i32=input; V }", "error[E0435]", False),
        "overflow": ("pub fn value()->i32 { const V:i32=i32::MAX+1; 0 }", "error[E0080]", False),
        "panic": ('pub fn value()->i32 { const V:i32=panic!("bad"); 0 }', "error[E0080]", False),
        "endless": ("pub fn value()->i32 { const V:i32=loop {}; 0 }", "constant evaluation is taking a long time", False),
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
    print("68 atomic local constant and item boundary rejections")


if __name__ == "__main__":
    main()
