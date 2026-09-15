"""Pinned observations are preparation, not target generation or admission."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "owned-clone representation observation passed"


def run(probe, fixture):
    return subprocess.run([str(probe), str(fixture)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout
    print(result.stdout)
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="clone-observation-") as directory:
        for name, text in {
            "borrow_form": original.replace("let cloned = original.clone();", "let cloned = Clone::clone(&original);", 1),
            "replacement": original.replace("let cloned = original.clone();", "let cloned = Box::new(*original);", 1),
            "drop_place": original.replace("let cloned = original.clone();", "let cloned = original.clone(); let original = original;", 1),
            "reference_impl": original.replace("<&Box<i32> as Clone>::clone(&borrowed)", "<Box<i32> as Clone>::clone(borrowed)").replace("    **cloned", "    *cloned"),
        }.items():
            assert text != original, name
            source = Path(directory) / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("clone representation mutation controls passed")


if __name__ == "__main__":
    main()
