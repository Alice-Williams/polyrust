"""Require executable role mappings and a nonempty direct-call inventory."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "local-call identity proof passed"
REQUIRED = "three local roles and both registration orders passed"


def run(probe, fixture):
    return subprocess.run([str(probe), str(fixture)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout
    assert REQUIRED in result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="local-call-proof-") as directory:
        work = Path(directory)
        for name, text, error in [
            ("type", "fn consume(v: Box<i32>) -> i32 { *v } pub fn invalid() -> i32 { consume(1) }", "E0308"),
            ("move", "fn consume(v: Box<i32>) -> i32 { *v } pub fn invalid(v: Box<i32>) -> i32 { consume(v); consume(v) }", "E0382"),
            ("borrow", "pub fn invalid(mut v: Box<i32>) -> i32 { let a = &v; let b = &mut v; **b = 1; **a }", "E0502"),
            ("unsafe", "unsafe fn produce(v: i32) -> Box<i32> { Box::new(v) } pub fn invalid(v: i32) -> Box<i32> { produce(v) }", "E0133"),
        ]:
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and REQUIRED not in result.stdout
        for name, text in {
            "callee": original.replace("let owner = produce(value);", "let owner = produce_return(value);", 1),
            "role_identity": original.replace("    replacement(owner)\n", "    relay(owner)\n", 1),
            "adjustment": original.replace("pub fn result_adjusted(value: i32) -> Box<dyn std::fmt::Debug>", "pub fn result_adjusted(value: i32) -> Box<i32>", 1),
            "inventory": original.replace("pub fn via_producer(", "pub fn renamed_producer(", 1),
            "nominal_function": original.replace("pub mod right {\n    pub fn produce(value: i32) -> Box<i32> {\n        Box::new(value)\n    }\n}", "pub mod right { pub use super::left::produce; }", 1),
            "empty": "// Empty proof must not pass.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("local-call executable identity proof passed")


if __name__ == "__main__":
    main()
