"""Compiler-only observation and unsupported-heap controls."""
import pathlib
import subprocess
import sys
import tempfile

SUCCESS = "compiler-owned drop observation passed"


def run(*args):
    return subprocess.run(list(map(str, args)), text=True, capture_output=True,
                          check=False, timeout=60)


def main():
    probe, fixture, c_adapter, java_adapter = [pathlib.Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout, result.stdout
    print(result.stdout, end="")
    with tempfile.TemporaryDirectory(prefix="owned-probe-") as directory:
        work = pathlib.Path(directory)
        cases = {
            "move": ("let x = Box::new(1); let _moved = x; *x", "E0382"),
            "borrow": ("let mut x = Box::new(1); let r = &x; let s = &mut x; **s = 2; **r", "E0502"),
        }
        for name, (body, diagnostic) in cases.items():
            source = work / (name + ".rs")
            source.write_text("pub fn invalid() -> i32 { " + body + " }\n")
            result = run(probe, source)
            assert result.returncode != 0, name
            assert diagnostic in result.stderr, result.stderr
            assert SUCCESS not in result.stdout, result.stdout
            assert "Runtime(PostCleanup)" not in result.stdout, result.stdout
        heap = work / "heap.rs"
        heap.write_text("pub fn score(value: i32) -> i32 { let x = Box::new(value); *x }\n")
        for adapter, filename in [(c_adapter, "generated.c"), (java_adapter, "Generated.java")]:
            output = work / filename
            for existing in [False, True]:
                if existing:
                    output.write_bytes(b"preserved\x00\xff")
                result = run(adapter, heap, output)
                assert result.returncode != 0, "heap unexpectedly admitted"
                assert result.stderr.strip() == "unsupported Rust: direct calls require resolved ordinary functions", result.stderr
                if existing:
                    assert output.read_bytes() == b"preserved\x00\xff"
                else:
                    assert not output.exists()
        # The probe's closed fixture inventory must not pass vacuously.
        empty = work / "empty.rs"
        empty.write_text("// no bodies\n")
        result = run(probe, empty)
        assert result.returncode != 0 and SUCCESS not in result.stdout
        # Shape mutations must trip typed assertions, not Rust syntax/type errors.
        original = fixture.read_text()
        mutations = {
            "conditional": original.replace("selected = original;", "selected = Box::new(value);"),
            "partial": original.replace("let taken = both.first;", "let taken = both.second;")
                               .replace("*taken + *both.second", "*taken + *both.first"),
            "early": original.replace("return *owned;", "let _read = *owned;"),
            "counterfeit": original.replace("struct Box(i32);", "struct Pretender(i32);")
                                   .replace("Box(value).0", "Pretender(value).0"),
        }
        for name, text in mutations.items():
            assert text != original
            source = work / (name + "_mutation.rs")
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "assertion" in result.stderr, result.stderr
            assert "error[E" not in result.stderr, result.stderr
    print("invalid move/borrow and empty inventory reject; C/Java heap output stays absent or unchanged")


if __name__ == "__main__":
    main()
