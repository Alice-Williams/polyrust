"""Require early/continuation evidence and reject invalid or changed source."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "early owned return and lexical continuation passed"
MUTATIONS = "19 early MIR and 8 lexical-exit corruptions rejected; residual controls passed"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout, result.stdout
    assert MUTATIONS in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-early-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("move", "let x = Box::new(a); let _moved = x; if flag { return *x; } *x", "E0382"),
            ("borrow", "let mut x = Box::new(a); let r = &x; let s = &mut x; **s = a; if flag { return **r; } **r", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid(flag: bool, a: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        for name, text in {
            "wrong_guard": original.replace("if flag", "if !flag", 1),
            "wrong_result": original.replace("return *x;", "return *y;", 1),
            "wrong_continuation": original.replace("    *y\n", "    *x\n", 1),
            "empty": "// No empty early-return proof.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("early/continuation inventory and source controls passed")


if __name__ == "__main__":
    main()
