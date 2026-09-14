"""Require complete owner/mutation inventories and reject invalid source first."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "disjoint parameter-anchored ownership chains passed"
MUTATIONS = "22 multi-owner corruptions rejected; unread constant substitution remains harmless"
RENAMING = "bijective owner-local renaming preserves the authenticated relation"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MUTATIONS in result.stdout, result.stdout
    assert RENAMING in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-multiple-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("move", "let x = Box::new(a); let y = Box::new(b); let _moved = x; *x + *y", "E0382"),
            ("borrow", "let mut x = Box::new(a); let r = &x; let s = &mut x; **s = b; **r", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid(a: i32, b: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and MUTATIONS not in result.stdout
        for name, text in {
            "wrong_read": original.replace("let _y = Box::new(b);\n    *x", "let _y = Box::new(b);\n    *_y", 1),
            "repeated_anchor": original.replace("let _y = Box::new(b);", "let _y = Box::new(a);", 1),
            "empty": "// Do not claim equivalence for an empty inventory.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("multiple owner anchors, corruption oracles, residual-use checks and invalid-source controls passed")


if __name__ == "__main__":
    main()
