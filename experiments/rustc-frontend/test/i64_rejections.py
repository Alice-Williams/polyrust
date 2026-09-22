"""Typed i64 source boundary and Java long-slot limits remain fail-closed."""
import os
from pathlib import Path
import subprocess
import sys


def run(command):
    return subprocess.run([str(arg) for arg in command], capture_output=True, text=True, timeout=90)


def main():
    c, java = sys.argv[1:]
    root = Path(os.environ["TEST_TMPDIR"]) / "i64-rejections"
    root.mkdir()
    signature = "direct-call signatures require admitted scalar source types"
    cases = {
        "u64": ("pub fn value(v: u64) -> u64 { v }", signature, True),
        "i128": ("pub fn value(v: i128) -> i128 { v }", signature, True),
        "float": ("pub fn value(v: f32) -> f32 { v }", signature, True),
        "cast": ("pub fn value(v: i64) -> i64 { (v as i32) as i64 }", "signed widening supports only an unadjusted i32 operand cast to i64", True),
        "arithmetic": ("pub fn value(v: i64) -> i64 { v + 1 }", "only comparison binary operators", True),
        "negate": ("pub fn value(v: i64) -> i64 { -v }", "only negative scalar literals", True),
        "mutable": ("pub fn value(v: i64) -> i64 { let mut x = v; x = 2; x }", "only plain immutable bindings", True),
        "positive_overflow": ("pub fn value() -> i64 { 9223372036854775808i64 }", "literal out of range for", False),
        "negative_overflow": ("pub fn value() -> i64 { -9223372036854775809i64 }", "literal out of range for", False),
        "wrong_suffix": ("pub fn value() -> i64 { 1u64 }", "error[E0308]", False),
        "mixed_compare": ("pub fn value(v: i64, n: i32) -> bool { v == n }", "error[E0308]", False),
    }
    for label, (text, diagnostic, valid) in cases.items():
        source = root / (label + ".rs")
        source.write_text(text + "\n")
        for language, adapter in [("c", c), ("java", java)]:
            for existing in [False, True]:
                work = root / (label + language + str(existing))
                work.mkdir()
                output = work / ("package" if language == "c" else "Generated.java")
                if existing:
                    if language == "c":
                        output.mkdir()
                    sentinel = output / "sentinel" if language == "c" else output
                    sentinel.write_bytes(b"preserved\x00\xff")
                before = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                result = run([adapter, source, output, "--package"])
                assert result.returncode != 0 and diagnostic in result.stderr, (label, language, result.stderr)
                if valid:
                    assert "error[E" not in result.stderr, (label, "must be valid unsupported Rust", result.stderr)
                after = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    runtimes = [p / "bin/javac" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    for wide_count, extra_bool, accepted in [(127, True, True), (128, False, False)]:
        label = "slots" + str(wide_count)
        source = root / (label + ".rs")
        parameters = ["value: i64"] + [f"_v{i}: i64" for i in range(1, wide_count)]
        if extra_bool:
            parameters.append("_flag: bool")
        source.write_text("pub fn value(" + ",".join(parameters) + ") -> i64 { value }\n")
        work = root / label
        work.mkdir()
        output = work / "Generated.java"
        result = run([java, source, output, "--package"])
        if accepted:
            assert result.returncode == 0, result.stderr
            classes = work / "classes"
            classes.mkdir()
            compiled = run([runtimes[0], "--release", "21", "-Xlint:all", "-Werror", "-d", classes, output])
            assert compiled.returncode == 0, compiled.stderr
        else:
            assert result.returncode != 0 and "parameter" in result.stderr.lower(), result.stderr
            assert "error[E" not in result.stderr and not output.exists()
    print("44 atomic i64 boundary rejections; Java 255-slot method compiles and 256-slot method rejects")


if __name__ == "__main__":
    main()
