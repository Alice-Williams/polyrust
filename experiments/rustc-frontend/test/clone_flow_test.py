"""Runtime proof inventory; target heap generation is still disabled."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "clone-flow identity proof passed"
REQUIRED = "eleven clone bodies and thirteen source rejections passed"


def run(probe, fixture):
    return subprocess.run([str(probe), str(fixture)], capture_output=True,
                          text=True, check=False, timeout=60)


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture)
    assert result.returncode == 0, result.stderr
    assert SUCCESS in result.stdout and REQUIRED in result.stdout
    assert "clone-flow corruption cases: 528" in result.stdout
    print(result.stdout)
    original = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="clone-flow-") as directory:
        source = Path(directory) / "fixture.rs"
        for name, text, error in [
            ("type", "pub fn invalid(v: Box<bool>) -> Box<i32> { v.clone() }", "E0308"),
            ("move", "pub fn invalid(v: Box<i32>) -> Box<i32> { let moved = v; v.clone() }", "E0382"),
            ("borrow", "pub fn invalid(mut v: Box<i32>) -> Box<i32> { let a = &mut v; let b = v.clone(); **a = 1; b }", "E0502"),
            ("unsafe", "unsafe fn danger() {} pub fn invalid() { danger(); }", "E0133"),
        ]:
            source.write_text(original + "\n" + text)
            result = run(probe, source)
            assert result.returncode != 0 and error in result.stderr, (name, result.stderr)
            assert SUCCESS not in result.stdout and REQUIRED not in result.stdout
        for name, text in {
            "clone_identity": original.replace("let cloned = original.clone();", "let cloned = Box::new(*original);", 1),
            "read_owner": original.replace("    *cloned\n", "    *original\n", 1),
            "exit_kind": original.replace("    *cloned\n", "    return *cloned;\n", 1),
            "borrow_form": original.replace("let cloned = original.clone();", "let cloned = Clone::clone(&original);", 1),
            "drop_order": original.replace("    let owner = owner;\n", "", 1),
            "inventory": original.replace("pub fn bounded(", "pub fn renamed("),
            "empty": "// Empty proof must not succeed.\n",
        }.items():
            assert text != original, name
            source.write_text(text)
            result = run(probe, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
        old = "pub fn bounded(value: i32) -> i32 {\n    let original = Box::new(value);\n    let cloned = original.clone();\n    *cloned\n}"
        assert old in original
        for count, accepted in [(128, True), (129, False)]:
            replacement = "pub fn bounded(value: i32) -> i32 { let v0 = Box::new(value); let v1 = v0.clone();\n"
            replacement += "\n".join(f"let v{i} = v{i-1};" for i in range(2, count))
            replacement += f"\n*v{count-1}\n}}"
            source.write_text(original.replace(old, replacement))
            result = run(probe, source)
            assert (result.returncode == 0) == accepted, result.stdout + result.stderr
            assert (SUCCESS in result.stdout) == accepted
    print("clone-flow source and budget controls passed")


if __name__ == "__main__":
    main()
