"""Valid-source exclusions, compiler errors, and non-vacuous mutation oracles."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "closed linear ownership correspondence passed"
MUTATIONS = "24 private ownership correspondence mutations rejected"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MUTATIONS in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-linear-") as directory:
        work = Path(directory)
        for name, body, diagnostic in [
            ("move", "let x = Box::new(1); let _moved = x; *x", "E0382"),
            ("borrow", "let mut x = Box::new(1); let r = &x; let s = &mut x; **s = 2; **r", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("pub fn invalid() -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and diagnostic in result.stderr, result.stderr
            assert SUCCESS not in result.stdout and MUTATIONS not in result.stdout
        mutations = {
            "constant_argument": original.replace("let owner = Box::new(value);", "let owner = Box::new(7);", 1),
            "mutable_owner": original.replace("let owner = Box::new(value);", "let mut owner = Box::new(value);", 1),
            "nested_block": original.replace("let owner = Box::new(value);", "let owner = { Box::new(value) };", 1),
            "explicit_return": original.replace("    *owner\n", "    return *owner;\n", 1),
            "empty_inventory": "// Must not pass an empty fixture.\n",
        }
        for name, text in mutations.items():
            assert text != original
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("linear correspondence, private MIR mutations, valid-source exclusions and invalid-source controls passed")


if __name__ == "__main__":
    main()
