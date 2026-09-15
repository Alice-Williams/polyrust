"""Unsupported/invalid Not inputs must not create or overwrite target files."""
import os
from pathlib import Path
import subprocess
import sys


def invoke(command):
    return subprocess.run([str(arg) for arg in command], capture_output=True,
                          text=True, timeout=60)


def main():
    c, java = map(Path, sys.argv[1:])
    work = Path(os.environ["TEST_TMPDIR"]) / "boolean-rejections"
    work.mkdir()
    negation = "Boolean negation requires an unadjusted built-in bool operand"
    cases = {
        "integer_reference": ("let reference = &value; if !reference == 0 { 1 } else { 0 }", "integer bitwise requires an unadjusted built-in"),
        "bool_reference": ("let flag = value > 0; let reference = &flag; if !reference { 1 } else { 0 }", negation),
        "bitand_reference": ("if (&(value > 0)) & (value < 10) { 1 } else { 0 }", "eager Boolean input requires"),
        "bitor_reference": ("if (value > 0) | (&(value < 10)) { 1 } else { 0 }", "eager Boolean input requires"),
        "integer_condition": ("if !value { 1 } else { 0 }", "error[E0308]"),
        "float": ("if !1.0 { value } else { 0 }", "error[E0600]"),
    }
    sources = {name: (f"pub fn score(value: i32) -> i32 {{ {body} }}\n", diagnostic)
               for name, (body, diagnostic) in cases.items()}
    sources["overloaded"] = (
        "struct Flag { value: bool }\n"
        "impl std::ops::Not for Flag { type Output = bool; fn not(self) -> bool { self.value } }\n"
        "pub fn score(value: i32) -> i32 { let flag = Flag { value: value > 0 }; if !flag { 1 } else { 0 } }\n",
        negation)
    for case, (source, diagnostic) in sources.items():
        fixture = work / (case + ".rs")
        fixture.write_text(source)
        for language, adapter, filename in [("c", c, "output.c"), ("java", java, "Generated.java")]:
            directory = work / (case + "-" + language)
            directory.mkdir()
            output = directory / filename
            for existing in [False, True]:
                if existing:
                    output.write_bytes(b"preserved\x00\xff")
                result = invoke([adapter, fixture, output])
                assert result.returncode != 0 and diagnostic in result.stderr, (case, language, result.stderr)
                if not diagnostic.startswith("error["):
                    assert "error[E" not in result.stderr, (case, "must be valid but unsupported Rust", result.stderr)
                assert set(directory.iterdir()) == ({output} if existing else set()), (case, "partial publication")
                if existing:
                    assert output.read_bytes() == b"preserved\x00\xff"
    for case, condition in [("not", "!(value > 0)"), ("and", "(value > 0) & (value < 10)"),
                            ("or", "(value > 0) | (value < 10)")]:
        fixture = work / (case + ".rs")
        fixture.write_text(f"pub fn score(value: i32) -> i32 {{ if {condition} {{ 1 }} else {{ 0 }} }}\n")
        for language, adapter, filename in [("c", c, "output.c"), ("java", java, "Generated.java")]:
            directory = work / (case + "-" + language)
            directory.mkdir()
            output = directory / filename
            result = invoke([adapter, fixture, output])
            assert result.returncode == 0 and output.is_file(), (case, language, result.stderr)
    print("Seven invalid/unsupported cases x two targets x absent/existing output reject; six valid controls publish; integer complement is covered by bitwise native tests")


if __name__ == "__main__":
    main()
