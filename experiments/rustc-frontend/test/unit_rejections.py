"""Valid but unsupported Rust must reject atomically, never erase statements."""
import os
from pathlib import Path
import subprocess
import sys

CASES = {
    "unit_local": ("fn action(v: i32) { let _unit = (); }", "action(value);", "type mapping is not implemented"),
    "unit_field": ("struct Record { value: () } fn action(v: i32) { let _record = Record { value: () }; }", "action(value);", "only scalar record fields are implemented"),
    "unit_parameter": ("fn action(_: ()) {}", "action(());", "direct-call signatures require admitted scalar source types"),
    "tuple_result": ("fn action(v: i32) -> (i32, i32) { (v, v) }", "let pair = action(value);", "direct-call signatures require admitted scalar source types"),
    "early_return": ("fn action(_value: i32) { return; }", "action(value);", "unit effects require"),
    "nonunit_statement": ("fn action(v: i32) { v; }", "action(value);", "unit effects require"),
    "generic_call": ("fn action<T>(_: T) {}", "action(value);", "generic or mismatched direct callee identity"),
    "indirect_call": ("fn empty() {} fn action(_value: i32) { let f: fn() = empty; f(); }", "action(value);", "direct calls require resolved ordinary functions"),
    "unit_constant": ("const NOTHING: () = (); fn action(_value: i32) { NOTHING }", "action(value);", "unsupported unit effect"),
    "never_result": ("fn action(_value: i32) -> ! { loop {} }", "action(value);", "direct-call signatures require admitted scalar source types"),
    "unsafe_call": ("unsafe fn action(_value: i32) {}", "unsafe { action(value); }", "declaration of an `unsafe` function"),
    "external_effect": ("fn action(_value: i32) { std::thread::yield_now(); }", "action(value);", {"c": "foreign direct calls are not implemented", "java": "foreign Java direct calls require an authenticated dependency certificate"}),
    "mutable_local": ("fn action(v: i32) { let mut copy = v; copy = v; }", "action(value);", "only plain immutable bindings are implemented"),
}

def main():
    root = Path(os.environ["TEST_TMPDIR"]) / "unit-rejections"
    root.mkdir()
    for label, (helpers, statement, diagnostic) in CASES.items():
        source = root / (label + ".rs")
        source.write_text(helpers + f"\npub fn score(value: i32) -> i32 {{ {statement} value }}\n")
        for language, adapter, filename in [("c", sys.argv[1], "out.c"), ("java", sys.argv[2], "Generated.java")]:
            directory = root / (label + "-" + language)
            directory.mkdir()
            output = directory / filename
            for existing in [False, True]:
                if existing:
                    output.write_bytes(b"unchanged\x00\xff")
                result = subprocess.run([adapter, source, output], capture_output=True, text=True, timeout=60)
                expected_diagnostic = diagnostic[language] if isinstance(diagnostic, dict) else diagnostic
                assert result.returncode != 0 and expected_diagnostic in result.stderr, (label, language, result.stderr)
                assert "error[E" not in result.stderr and "panicked at" not in result.stderr, (label, result.stderr)
                assert set(directory.iterdir()) == ({output} if existing else set())
                if existing:
                    assert output.read_bytes() == b"unchanged\x00\xff"
    print("Unit unsupported-source cases reject in both targets, preserving absent/existing outputs")

if __name__ == "__main__":
    main()
