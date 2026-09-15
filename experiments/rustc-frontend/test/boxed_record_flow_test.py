"""Compiler-only boxed scalar-record correspondence proof."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "boxed-record flow correspondence passed"
REQUIRED = ["40 boxed-record MIR corruptions rejected", "3 boxed-record source substitutions rejected", "same-type parameter and Copy payload substitutions rejected"]


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
    with tempfile.TemporaryDirectory(prefix="boxed-record-flow-") as directory:
        work = Path(directory)
        for name, body, error in [
            ("moved", "let r = Record { number: value, enabled: flag }; let x = Box::new(r); let y = x; (*x).number", "E0382"),
            ("missing", "let r = Record { number: value }; let x = Box::new(r); (*x).number", "E0063"),
            ("duplicate", "let r = Record { number: value, number: value, enabled: flag }; let x = Box::new(r); (*x).number", "E0062"),
            ("borrowed", "let mut x = Box::new(Record { number: value, enabled: flag }); let a = &x.number; let b = &mut x.number; *b = 3; *a", "E0502"),
        ]:
            source = work / (name + ".rs")
            source.write_text("struct Record { number: i32, enabled: bool }\npub fn invalid(value: i32, flag: bool) -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, result.stderr
            assert SUCCESS not in result.stdout
            assert all(marker not in result.stdout for marker in REQUIRED)
        for name, text in {
            "selected": original.replace("(*owner).number", "i32::from((*owner).enabled)", 1),
            "return_to_tail": original.replace("return (*owner).number;", "(*owner).number", 1),
            "constant": original.replace("number: value,", "number: 7,", 1),
            "order": original.replace("number: value,\n        enabled: flag,", "enabled: flag,\n        number: value,", 1),
            "inventory": original.replace("pub fn number(", "pub fn renamed_number(", 1),
            "missing_historical": original.replace("pub fn tail(value: i32) -> i32 {\n    let owner = Box::new(value);\n    *owner\n}\n", "", 1),
            "nominal": original.replace("pub mod right {\n    pub struct Record {\n        pub number: i32,\n        pub enabled: bool,\n    }", "pub mod right {\n    use super::left::Record;", 1),
            "empty": "// Empty proof inventory must fail.\n",
        }.items():
            assert text != original, name
            source = work / (name + ".rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
    print("boxed-record source controls and complete proof inventory passed")


if __name__ == "__main__":
    main()
