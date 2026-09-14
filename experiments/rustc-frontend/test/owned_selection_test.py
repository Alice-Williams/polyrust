"""Require selected-source evidence, mutation controls and failed-source isolation."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "conditional owned selection correspondence passed"
MUTATIONS = "33 selection MIR, 2 isolated flag and 4 source corruptions rejected"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MUTATIONS in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-selection-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("move", "let x = Box::new(a); let y = Box::new(b); let s = if flag { x } else { y }; *x + *s", "E0382"),
            ("borrow", "let mut x = Box::new(a); let r = &x; let y = &mut x; **y = b; if flag { **r } else { **r }", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid(flag: bool, a: i32, b: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        for name, text in {
            "wrong_guard": original.replace("if flag", "if !flag", 1),
            "swapped": original.replace("if flag { first } else { second }", "if flag { second } else { first }", 1),
            "empty": "// No empty selection proof.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("selected ownership inventory and source controls passed")


if __name__ == "__main__":
    main()
