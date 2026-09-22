"""Valid unsupported source fails atomically; exact f64 slot accounting."""
import os
from pathlib import Path
import subprocess
import sys


def run(command):
    return subprocess.run([str(arg) for arg in command], capture_output=True, text=True, timeout=90)


def main():
    c, java = sys.argv[1:]
    root = Path(os.environ["TEST_TMPDIR"]) / "binary64-rejections"
    root.mkdir()
    cases = {
        "f32": ("pub fn value(v:f32)->f32 {v}", "signatures require admitted scalar source types", True),
        "euclidean_remainder": ("pub fn value(v:f64)->f64 {v.rem_euclid(1.0)}", "expression mapping is not implemented", True),
        "integer_negate": ("pub fn value(v:i64)->i64 {-v}", "only negative scalar literals", True),
        "cast": ("pub fn value(v:i64)->f64 {v as f64}", "signed widening supports only an unadjusted i32 operand cast to i64", True),
        "narrow": ("pub fn value(v:f64)->i64 {v as i64}", "signed widening supports only an unadjusted i32 operand cast to i64", True),
        "method": ("pub fn value(v:f64)->f64 {v.floor()}", "expression mapping is not implemented", True),
        "nonfinite": ("#![allow(overflowing_literals)]\npub fn value()->f64 {1e400}", "nonfinite f64 literals", True),
        "constant": ("pub const VALUE:f64=f64::NAN; pub fn value()->f64 {VALUE}", "NaN f64 constants", True),
        "local_constant": ("pub fn value()->f64 {const V:f64=f64::NAN; V}", "NaN f64 constants", True),
        "mutable": ("pub fn value(v:f64)->f64 {let mut x=v; x=2.0; x}", "only plain immutable bindings", True),
        "mixed": ("pub fn value(a:f64,b:f32)->bool {a==b}", "error[E0308]", False),
        "reference_comparison": ("pub fn value(a:f64,b:f64)->bool {&a == &b}", "overloaded operators", True),
        "reference_abi": ("pub fn value(v:&f64)->f64 {*v}", "signatures require admitted scalar source types", True),
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
                    assert "error[E" not in result.stderr, (label, "must be valid Rust", result.stderr)
                after = {str(p.relative_to(work)): p.read_bytes() for p in work.rglob("*") if p.is_file()}
                assert before == after and output.exists() == existing
    runtimes = [p / "bin/javac" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    for count, accepted in [(127, True), (128, False)]:
        source = root / f"slots{count}.rs"
        parameters = ["value:f64"] + [f"_v{i}:f64" for i in range(1, count)]
        if accepted:
            parameters.append("_flag:bool")
        source.write_text("pub fn value(" + ",".join(parameters) + ")->f64 {value}\n")
        work = root / f"slots{count}"
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
    print("52 atomic f64 boundary rejections; exact Java 255/256-slot boundary")


if __name__ == "__main__":
    main()
