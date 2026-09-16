"""Real compiler exports, not a fabricated source graph or parsed target output."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    probe = str(Path(sys.argv[1]).resolve())
    work = Path(os.environ["TEST_TMPDIR"]) / "public-inventory"
    work.mkdir()

    def run(name, source, error=None):
        path = work / f"{name}.rs"
        path.write_text(source)
        result = subprocess.run([probe, str(path)], capture_output=True, text=True)
        if error is not None:
            assert result.returncode != 0, (name, result.stdout)
            assert error in result.stderr, (name, result.stderr)
            return ""
        assert result.returncode == 0, (name, result.stderr)
        return result.stdout

    def declarations(text, kind):
        return [line for line in text.splitlines() if line.startswith(f"DECL {kind} ")]

    def binding(text, name):
        ids = [line.split()[2] for line in text.splitlines()
               if line.startswith(f"BIND {name} ")]
        assert len(ids) == 1, (name, text)
        return ids[0]

    functions = run("functions", """
        mod hidden { /// Function documentation.
            pub fn original() -> i32 { 1 }
        }
        pub use hidden::original as first;
        pub use hidden::original as second;
        fn original() -> i32 { 2 }
    """)
    assert len(declarations(functions, "Function")) == 1
    assert not declarations(functions, "Constant")
    assert binding(functions, "first") == binding(functions, "second")
    assert "DOC Function documentation." in functions
    assert "FUNCTIONS_ONLY 1" in functions

    constants = run("constants", """
        mod hidden { /// Constant documentation.
            pub const VALUE: i64 = 9007199254740993;
        }
        pub use hidden::VALUE as first;
        pub use hidden::VALUE as second;
        const VALUE: i64 = 4;
    """)
    assert len(declarations(constants, "Constant")) == 1
    assert not declarations(constants, "Function")
    assert binding(constants, "first") == binding(constants, "second")
    assert "DOC Constant documentation." in constants
    assert "FUNCTIONS_ONLY_REJECTED" in constants

    mixed = run("mixed", """
        pub const VALUE: i32 = 3;
        pub fn value() -> i32 { VALUE }
        pub mod inner {
            pub const VALUE: bool = true;
            pub fn value() -> bool { VALUE }
            pub use crate::inner as cycle;
        }
        pub use inner as alternate;
        mod private { pub const VALUE: i32 = 9; }
    """)
    assert len(declarations(mixed, "Constant")) == 2
    assert len(declarations(mixed, "Function")) == 2
    assert "MODULES 2" in mixed
    assert "FUNCTIONS_ONLY_REJECTED" in mixed
    assert len({line.split()[-1] for line in declarations(mixed, "Constant")}) == 2

    # Classification is not scalar-type admission: no target is generated here.
    nonscalar = run("classification", "pub const VALUE: &str = \"text\";")
    assert len(declarations(nonscalar, "Constant")) == 1
    assert "FUNCTIONS_ONLY_REJECTED" in nonscalar

    boundary = run("at_limit", "\n".join(
        f"pub const C{i}:i32={i};" for i in range(4096)
    ))
    assert len(declarations(boundary, "Constant")) == 4096
    assert "FUNCTIONS_ONLY_REJECTED" in boundary

    cases = {
        "empty": ("fn private() {}", "requires an exported"),
        "empty_module": ("pub mod empty {}", "requires an exported"),
        "struct": ("pub struct Shape { pub x:i32 }", "API mapping"),
        "static": ("pub static VALUE:i32=1;", "API mapping"),
        "enum": ("pub enum Shape { A, B }", "API mapping"),
        "trait": ("pub trait Shape { const VALUE:i32; }", "API mapping"),
        "alias": ("pub type Shape=i32;", "API mapping"),
        "macro": ("#[macro_export] macro_rules! shape {()=>{1}}", "API mapping"),
        "foreign_function": ("pub use core::cmp::max;", "API mapping"),
        "foreign_module": ("pub use core::cmp;", "foreign or unsupported module"),
        "foreign_constant": ("pub use core::f64::consts::PI;", "API mapping"),
        "too_many": ("\n".join(f"pub const C{i}:i32=1;" for i in range(4097)),
                     "inventory size budget"),
    }
    for name, (source, diagnostic) in cases.items():
        run(name, source, diagnostic)
    print("5 compiler inventory graphs and 12 rejection controls passed")


if __name__ == "__main__":
    main()
