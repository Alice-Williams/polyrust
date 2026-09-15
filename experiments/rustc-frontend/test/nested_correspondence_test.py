"""Invalid, unsupported and empty source inventories must never publish proof."""
from pathlib import Path
import subprocess
import re
import sys
import tempfile

SUCCESS = "nested correspondence proof passed"


def run(probe, source):
    return subprocess.run([str(probe), str(source)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout, result.stdout
    assert "nested consumer projections passed" in result.stdout, result.stdout
    assert "nested corruption inventory passed" in result.stdout, result.stdout
    counts = [int(value) for value in re.findall(r"nested body corruptions rejected: (\d+)", result.stdout)]
    assert len(counts) == 14 and min(counts) >= 100, result.stdout
    assert result.stdout.count("nested complete owner-local bijection accepted") == 14
    assert result.stdout.count("nested canonical source owner substitution rejected") == 14
    print(f"14 nested bodies: {sum(counts)} MIR corruptions, 14 source substitutions rejected; 14 local bijections accepted")
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="nested-correspondence-") as directory:
        work = Path(directory)
        empty = work / "empty.rs"
        empty.write_text("// No functions: a proof inventory must not pass vacuously.\n")
        result = run(probe, empty)
        assert result.returncode != 0 and SUCCESS not in result.stdout
        assert "panicked at" in result.stderr and "error[E" not in result.stderr
        for name, body, error in [
            ("reuse", "let inner = Inner { first: Box::new(a), second: Box::new(b) }; let outer = Outer { nested: inner, spare: Box::new(c) }; drop(inner); 0", "E0382"),
            ("twice", "let inner = Inner { first: Box::new(a), second: Box::new(b) }; let one = inner.first; let two = inner.first; *one", "E0382"),
            ("uninitialized", "let inner; if a > 0 { inner = Inner { first: Box::new(a), second: Box::new(b) }; } drop(inner); 0", "E0381"),
        ]:
            source = work / (name + ".rs")
            source.write_text(original + "\nfn invalid(a: i32, b: i32, c: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
        for name, before, after in [
            ("mutable_parameter", "fn first(a: i32", "fn first(mut a: i32"),
            ("mutable_binding", "let x = Box::new(a);", "let mut x = Box::new(a);"),
            ("computed_argument", "let z = Box::new(c);", "let z = Box::new(c + 1);"),
            ("wrong_constructor", "let x = Box::new(a);", "let x = Box::from(a);"),
            ("extra_call", "let y = Box::new(b);", "let y = Box::new(b); let duplicate = x.clone(); drop(duplicate);"),
            ("borrow", "let taken = outer.nested.first;\n    *taken", "let taken = &outer.nested.first;\n    **taken"),
            ("branch", "let taken = outer.nested.first;", "if c > 0 { return 1; } let taken = outer.nested.first;"),
            ("nested_block", "let taken = outer.nested.first;", "let taken = { outer.nested.first };"),
            ("scalar_return", "*taken", "1"),
            ("whole_outer", "let taken = outer.nested.first;", "let moved = outer; let taken = moved.nested.first;"),
        ]:
            assert before in original, name
            text = original.replace(before, after, 1)
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("nested correspondence source controls passed")


if __name__ == "__main__":
    main()
