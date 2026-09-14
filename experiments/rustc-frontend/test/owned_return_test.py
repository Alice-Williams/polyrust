"""Nonempty explicit-return proof and invalid/unsupported-source controls."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "canonical explicit owned return exits passed"
MUTATIONS = "14 explicit return identity, scope and cleanup corruptions rejected"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MUTATIONS in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-returns-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("move", "let x = Box::new(a); let _moved = x; return *x;", "E0382"),
            ("borrow", "let mut x = Box::new(a); let r = &x; let s = &mut x; **s = a; return **r;", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid(a: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and MUTATIONS not in result.stdout
        for name, text in {
            "wrong_value": original.replace("return *x", "return a", 1),
            "wrong_owner": original.replace("return *y1;", "return *_x2;", 1),
            "empty": "// Empty inventories cannot establish the proof.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("explicit return inventory, canonical identity, cleanup and invalid-source controls passed")


if __name__ == "__main__":
    main()
