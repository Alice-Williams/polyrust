"""Exact fixture inventory, source error isolation, and field-order controls."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "owned record construction proof passed"
MAPPING = "record identities, field ordering and executable bindings passed"


def run(probe, source, dependencies=()):
    args = [str(probe), str(source)]
    for dependency in dependencies:
        args += ["--input", str(dependency)]
    return subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)


def main():
    probe, fixture, *dependencies = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture, dependencies)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and MAPPING in result.stdout, result.stdout
    assert "record allocator identity oracle passed" in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-record-") as directory:
        work = Path(directory)
        copies = []
        for dependency in dependencies:
            copy = work / dependency.name
            copy.write_text(dependency.read_text())
            copies.append(copy)
        for name, expression, error in [
            ("missing", "Record { first: Box::new(a) }", "E0063"),
            ("duplicate", "Record { first: Box::new(a), first: Box::new(a), second: Box::new(a) }", "E0062"),
            ("move", "{ let x = Box::new(a); Record { first: x, second: x } }", "E0382"),
        ]:
            source = work / (name + ".rs")
            source.write_text("struct Record { first: Box<i32>, second: Box<i32> }\nfn invalid(a: i32) -> Record { " + expression + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        first = "first: Box::new(a),\n        second: Box::new(b),"
        for name, text, inputs in [
            ("reordered", original.replace(first, "second: Box::new(b),\n        first: Box::new(a),", 1), copies),
            ("representation", original.replace("pub struct Record", "#[repr(C)]\npub struct Record", 1), copies),
            ("empty", "// No empty record proof.\n", []),
        ]:
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source, inputs)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("record source controls and nonempty inventory passed")


if __name__ == "__main__":
    main()
