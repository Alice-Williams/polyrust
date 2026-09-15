"""Source controls fail before success, independently of compile-negative APIs."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "nested record construction proof passed"


def run(probe, source, dependencies=()):
    args = [str(probe), str(source)]
    for dependency in dependencies:
        args += ["--input", str(dependency)]
    return subprocess.run(args, capture_output=True, text=True, check=False, timeout=60)


def main():
    probe, fixture, budget = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture, [budget])
    assert result.returncode == 0, result.stderr
    for marker in [SUCCESS, "nested identities, budgets and executable bindings passed", "nested allocator and ancestor controls passed"]:
        assert marker in result.stdout, result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="nested-record-") as directory:
        work = Path(directory)
        copied = work / budget.name
        copied.write_text(budget.read_text())
        for name, body, error in [
            ("missing", "Branch { left }", "E0063"),
            ("duplicate", "Branch { left, left: right, right: other }", "E0062"),
            ("moved", "Branch { left, right: left }", "E0382"),
            ("type", "Branch { left, right: Box::new(1) }", "E0308"),
        ]:
            source = work / (name + ".rs")
            source.write_text(original + "\nfn invalid(left: Leaf, right: Leaf, other: Leaf) -> Branch { " + body + " }\n")
            result = run(probe, source, [copied])
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        for name, text, dependencies in [
            ("not_allocator", original.replace("std::alloc::System", "std::time::Duration", 1), [copied]),
            ("order", original.replace("Branch { right, left }", "Branch { left, right }", 1), [copied]),
            ("representation", original.replace("struct Leaf {", "#[repr(C)]\nstruct Leaf {", 1), [copied]),
            ("nominal", original.replace("pub fn alias(left: Alias, right: Alias) -> Branch {\n    Branch { left, right }", "pub fn alias(left: Alias, right: Alias) -> Alternate {\n    Alternate { left, right }") + "\nstruct Alternate { left: Leaf, right: Leaf }\n", [copied]),
            ("empty", "// Empty is not a successful proof.\n", []),
        ]:
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source, dependencies)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
            if name == "not_allocator":
                assert "alternate allocator must implement the actual Allocator trait" in result.stderr, result.stderr
    print("nested source controls and nonempty inventory passed")


if __name__ == "__main__":
    main()
