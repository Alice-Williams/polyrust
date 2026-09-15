"""Require correspondence, mutation, privacy and source-boundary evidence."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "owned field proof passed"
REQUIRED = [
    "owned field correspondence passed",
    "31 record aggregate/place/cleanup corruptions rejected",
    "3 source nominal/field substitutions rejected",
]


def run(probe, fixture):
    return subprocess.run([str(probe), str(fixture)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout, result.stdout
    assert all(marker in result.stdout for marker in REQUIRED), result.stdout
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-fields-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("moved_field", "let r = One { value: Box::new(a) }; let x = r.value; let y = r.value; *x + *y", "E0382"),
            ("missing_field", "let r = One {}; *r.value", "E0063"),
            ("duplicate_field", "let r = One { value: Box::new(a), value: Box::new(a) }; *r.value", "E0062"),
            ("borrowed_field", "let mut r = One { value: Box::new(a) }; let x = &r.value; let y = &mut r.value; **y = a; **x", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("struct One { value: Box<i32> }\npub fn invalid(a: i32) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
            assert all(marker not in result.stdout for marker in REQUIRED)
        for name, text in {
            "field": original.replace("let taken = record.first;", "let taken = record.second;", 1),
            "anchor": original.replace("let x = Box::new(a);", "let x = Box::new(b);", 1),
            "inventory": original.replace("pub fn first(", "pub fn renamed_first(", 1),
            "empty": "// Empty inventory must not pass.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("record field source controls and complete proof inventory passed")


if __name__ == "__main__":
    main()
