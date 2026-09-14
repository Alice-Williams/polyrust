"""Run canonical compiler inputs, invalid-source controls and oracle mutations."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "typed constructor identities and executable binding passed"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0 and SUCCESS in result.stdout, result.stderr
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-construction-") as directory:
        work = Path(directory)
        for name, body, diagnostic in [
            ("move", "let x = Box::new(1); let _moved = x; *x", "E0382"),
            ("borrow", "let mut x = Box::new(1); let r = &x; let s = &mut x; **s = 2; **r", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid() -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        mutations = {
            "wrapper": original.replace("*Box::new(value)", "*imitation::new(value)", 1),
            "payload": original.replace("u32", "i32"),
            "counterfeit_name": original.replace("pub fn new(value: i32) -> Self", "pub fn different(value: i32) -> Self")
                                        .replace("imitation::Box::new", "imitation::Box::different"),
        }
        for name, text in mutations.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "assertion" in result.stderr, result.stderr
            assert "error[E" not in result.stderr, result.stderr
        empty = work / "empty.rs"
        empty.write_text("// closed fixture inventory must not pass vacuously\n")
        result = run(probe, empty)
        assert result.returncode != 0 and SUCCESS not in result.stdout
    print("constructor identities, executable bindings, invalid source and identity/payload mutations passed")


if __name__ == "__main__":
    main()
