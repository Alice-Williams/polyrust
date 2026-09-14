"""Require canonical scopes, non-vacuous metadata mutations and compiler errors."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "canonical tail ownership scopes passed"
MUTATIONS = "12 scope and outer-cleanup mutations rejected"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MUTATIONS in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-scopes-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("move", "let x = Box::new(1); { let _moved = x; } *x", "E0382"),
            ("borrow", "let mut x = Box::new(1); let r = &x; let s = &mut x; **s = 2; **r", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid() -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and MUTATIONS not in result.stdout
        for name, text in {
            "extra_wrapper": original.replace("    *owner\n", "    { *owner }\n", 1),
            "early_return": original.replace("    *owner\n", "    return *owner;\n", 1),
            "empty": "// Empty inventory is not proof.\n",
        }.items():
            assert text != original
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("canonical scopes, metadata mutations, outer cleanup and invalid source controls passed")


if __name__ == "__main__":
    main()
