"""Observe real checked nominal identities, including same-spelled impostors."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    probe = str(Path(sys.argv[1]).resolve())
    work = Path(os.environ["TEST_TMPDIR"]) / "scalar-result-identity"
    work.mkdir()

    def run(name, source, error=None):
        path = work / f"{name}.rs"
        path.write_text(source)
        output = subprocess.run([probe, str(path)], capture_output=True, text=True, timeout=90)
        if error is not None:
            assert output.returncode != 0 and error in output.stderr, (name, output)
            assert not output.stdout, (name, output.stdout)
            return None
        assert output.returncode == 0, (name, output.stderr)
        lines = output.stdout.splitlines()
        assert len(lines) == 1 and lines[0].startswith("RESULT ") and lines[0].endswith(" NO_DROP")
        return lines[0]

    standard = "core::result::Result<i32, core::num::TryFromIntError>"
    direct = f"pub fn identity(value: {standard}) -> {standard} {{ value }}"
    truth = run("direct", direct)
    assert run("no_std", "#![no_std]\n" + direct) == truth
    assert run("renamed", """
        use core::result::Result as Outcome;
        use core::num::TryFromIntError as Failure;
        pub fn identity(value: Outcome<i32, Failure>) -> Outcome<i32, Failure> { value }
    """) == truth
    assert run("alias", f"type Outcome = {standard}; pub fn identity(v: Outcome) -> Outcome {{ v }}") == truth
    projection = "core::result::Result<i32, <i32 as core::convert::TryFrom<i64>>::Error>"
    assert run("projection", f"pub fn identity(v: {projection}) -> {projection} {{ v }}") == truth
    assert run("reconstruct", f"""
        pub fn identity(value: {standard}) -> {standard} {{
            match value {{ Ok(v) => Ok(v), Err(e) => Err(e) }}
        }}
    """) == truth
    cases = [
        ("wrong_width", direct.replace("i32", "i64"), "result success payload"),
        ("unit_error", direct.replace("core::num::TryFromIntError", "()"), "result error payload"),
        ("local_error", "pub struct TryFromIntError; " + direct.replace("core::num::TryFromIntError", "TryFromIntError"), "result error payload"),
        ("local_result", "pub enum Result<T, E> { Ok(T), Err(E) } " + direct.replace("core::result::Result", "Result"), "expected standard scalar Result"),
        ("borrowed", direct.replace(f"value: {standard}", f"value: &'static {standard}").replace(f"-> {standard}", f"-> &'static {standard}"), "expected standard scalar Result"),
        ("generic", "pub fn identity<T>(v: Result<T, ()>) -> Result<T, ()> { v }", "generic result signatures"),
        ("empty", "pub const VALUE: i32 = 1;", "no result signature"),
        ("invalid", direct.replace("{ value }", "{ absent }"), "cannot find value"),
        ("output_width", f"""
            pub fn convert(v: {standard}) -> Result<i64, core::num::TryFromIntError> {{
                match v {{ Ok(x) => Ok(i64::from(x)), Err(e) => Err(e) }}
            }}
        """, "result success payload"),
        ("output_error", f"""
            pub fn convert(v: {standard}) -> Result<i32, ()> {{
                match v {{ Ok(x) => Ok(x), Err(_) => Err(()) }}
            }}
        """, "result error payload"),
    ]
    for name, source, error in cases:
        run(name, source, error)
    print("Six standard/renamed/alias/no_std/projection observations and ten negative controls")


if __name__ == "__main__":
    main()
