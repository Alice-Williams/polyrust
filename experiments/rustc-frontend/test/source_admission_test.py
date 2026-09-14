"""Unused Rust surfaces still have finite, fail-closed C/Java admission budgets."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    c_adapter, java_adapter = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "source-admission"
    work.mkdir()
    prefix = "#![allow(dead_code, unused_variables)]\npub fn score(value: i32) -> i32 { value }\n"
    cases = [
        ("width-control", prefix + "\n".join(f"fn unused{i}() {{}}" for i in range(32)), None),
        ("width-limit", prefix + "\n".join(f"fn unused{i}() {{}}" for i in range(25000)),
         "Rust source admission visit budget exceeded"),
        ("depth-control", prefix + "fn unused(value: " + "&" * 64 + "i32) {}", None),
        ("depth-limit", prefix + "fn unused(value: " + "&" * 140 + "i32) {}",
         "Rust source admission depth budget exceeded"),
    ]
    for case, source, diagnostic in cases:
        path = work / (case + ".rs")
        path.write_text(source)
        for language, adapter, filename in [("c", c_adapter, "generated.c"), ("java", java_adapter, "Generated.java")]:
            output = work / case / language / filename
            output.parent.mkdir(parents=True)
            for existing in ([False, True] if diagnostic else [False]):
                if existing:
                    output.write_bytes(b"preserved\x00\xff")
                result = subprocess.run([adapter, str(path), str(output)],
                                        capture_output=True, text=True, check=False, timeout=90)
                if diagnostic:
                    assert result.returncode != 0, (case, language)
                    # This prefix can only originate after successful full rustc analysis.
                    assert "unsupported Rust:" in result.stderr, result.stderr
                    assert diagnostic in result.stderr, result.stderr
                    assert output.read_bytes() == b"preserved\x00\xff" if existing else not output.exists()
                else:
                    assert result.returncode == 0 and output.is_file(), result.stderr
    print("C/Java unused-item width and reference-depth budgets reject after rustc analysis; controls pass; outputs preserved")


if __name__ == "__main__":
    main()
