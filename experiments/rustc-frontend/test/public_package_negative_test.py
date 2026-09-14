"""Unsupported/invalid Rust never creates or overwrites a package."""
import os
from pathlib import Path
import subprocess
import sys

adapter = str(Path(sys.argv[1]).resolve())
work = Path(os.environ["TEST_TMPDIR"]) / "package-negative"
work.mkdir()
entry = "pub fn entry() -> i32 { 1 }\n"
cases = {
    "empty": ("fn private() -> i32 { 1 }", "requires an exported"),
    "struct": (entry + "pub struct Public { pub value: i32 }", "API mapping"),
    "constant": (entry + "pub const VALUE: i32 = 1;", "API mapping"),
    "static": (entry + "pub static VALUE: i32 = 1;", "API mapping"),
    "enum": (entry + "pub enum Public { One, Two }", "API mapping"),
    "type_alias": (entry + "pub type Number = i32;", "API mapping"),
    "trait": (entry + "pub trait Public { fn value(&self) -> i32; }", "API mapping"),
    "macro": (entry + "#[macro_export] macro_rules! marker { () => { 1 }; }", "API mapping"),
    "foreign_function": (entry + "pub use core::cmp::max as foreign;", "API mapping"),
    "foreign_module": (entry + "pub use core::cmp as dependency;", "foreign or unsupported module"),
    "generic": ("pub fn generic<T>(value: T) -> i32 { 1 }", "nongeneric ordinary"),
    "i64": ("pub fn wide(value: i64) -> i64 { value }", "only i32 and bool"),
    "unit": ("pub fn empty() {}", "only i32 and bool"),
    "async": ("pub async fn later() -> i32 { 1 }", "only i32 and bool"),
    "extern": ('pub extern "C" fn foreign() -> i32 { 1 }', "ordinary Rust signatures"),
    "unsafe": ("pub unsafe fn forbidden() -> i32 { 1 }", "unsafe"),
    "unstable": ("#![feature(rustc_private)]\n" + entry, "E0554"),
    "private_type_error": (entry + 'fn broken() -> i32 { "wrong" }', "mismatched types"),
    "moved": (entry + "struct Record { value: i32 } fn broken() -> i32 { let x = Record { value: 1 }; let y = x; x.value }", "moved value"),
    "borrow": (entry + "fn broken() -> i32 { let mut x = 1; let shared = &x; let unique = &mut x; *unique = 2; *shared }", "cannot borrow"),
    "undeclared_module": ("mod extra; pub fn entry() -> i32 { extra::helper() }", "undeclared compiler file input"),
}
(work / "extra.rs").write_text("pub fn helper() -> i32 { 1 }")
for name, (source, diagnostic) in cases.items():
    input_path = work / f"{name}.rs"
    input_path.write_text(source)
    for existing in (False, True):
        output = work / f"{name}-{existing}"
        if existing:
            output.mkdir()
            (output / "sentinel").write_bytes(b"unchanged\x00package")
        environment = dict(os.environ, RUSTC_BOOTSTRAP="1")
        result = subprocess.run([adapter, str(input_path), str(output), "--package"], env=environment, capture_output=True, text=True)
        assert result.returncode != 0, name
        assert diagnostic in result.stderr, (name, result.stderr)
        if existing:
            assert list(output.iterdir()) == [output / "sentinel"], name
            assert (output / "sentinel").read_bytes() == b"unchanged\x00package", name
        else:
            assert not output.exists(), name
        assert not list(work.glob(".polyrust-stage-*")), name
# The same out-of-line source becomes admitted with its explicit input edge.
subprocess.run([adapter, str(work / "undeclared_module.rs"), str(work / "declared"),
                "--package", "--input", str(work / "extra.rs")], check=True)
assert len(list((work / "declared").iterdir())) == 3
print(f"{len(cases)} compiler/package rejection cases preserve missing and existing destinations")
