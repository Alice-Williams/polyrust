"""The operation reader must distinguish concrete ownership from references."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "box-clone identity proof passed"
REQUIRED = "four scalar Box clones and both registration orders passed"


def run(probe, fixture):
    return subprocess.run([str(probe), str(fixture)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and REQUIRED in result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="box-clone-proof-") as directory:
        work = Path(directory)
        for name, text, error in [
            ("type", "pub fn invalid(v: Box<bool>) -> Box<i32> { v.clone() }", "E0308"),
            ("move", "pub fn invalid(v: Box<i32>) -> Box<i32> { let moved = v; v.clone() }", "E0382"),
            ("borrow", "pub fn invalid(mut v: Box<i32>) -> Box<i32> { let a = &mut v; let b = v.clone(); **a = 1; b }", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        for name, text in {
            "method_form": original.replace("let cloned = original.clone();", "let cloned = Clone::clone(&original);", 1),
            "identity": original.replace("let cloned = original.clone();", "let cloned = Box::new(*original);", 1),
            "reference": original.replace("let cloned = original.clone();", "let cloned = <&Box<i32> as Clone>::clone(&&original);", 1).replace("    *cloned\n", "    **cloned\n", 1),
            "inventory": original.replace("pub fn qualified(", "pub fn renamed("),
            "empty": "// An empty crate must not pass.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("box-clone executable identity proof passed")


if __name__ == "__main__":
    main()
