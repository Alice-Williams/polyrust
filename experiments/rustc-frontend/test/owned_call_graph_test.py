"""Runtime graph oracle; fixtures stay declared and compiler-checked."""
from pathlib import Path
import subprocess
import sys
import tempfile

SUCCESS = "owned-call graph correspondence passed"
REQUIRED = "ten closed graphs and twenty-eight rejected entry bodies passed"


def run(probe, fixture, source):
    return subprocess.run([str(probe), str(fixture), "--input", str(source)],
                          capture_output=True, text=True, check=False, timeout=60)


def main():
    probe, fixture, source = [Path(p).resolve() for p in sys.argv[1:]]
    result = run(probe, fixture, source)
    assert result.returncode == 0, result.stdout + result.stderr
    assert SUCCESS in result.stdout and REQUIRED in result.stdout
    assert "owned-call corruption cases: 475" in result.stdout
    print(result.stdout)
    original, leaf = fixture.read_text(), source.read_text()
    with tempfile.TemporaryDirectory(prefix="owned-call-graph-") as directory:
        root = Path(directory)
        fixture = root / "graph.rs"
        source = root / "owned_calls.rs"
        source.write_text(leaf)
        for name, text, error in [
            ("type", "pub fn invalid(v: i32) -> i32 { let owner = source::produce(v); source::consume(v) }", "E0308"),
            ("move", "pub fn invalid(v: i32) -> i32 { let owner = source::produce(v); source::consume(owner); source::consume(owner) }", "E0382"),
            ("borrow", "pub fn invalid(mut v: Box<i32>) -> i32 { let a = &v; let b = &mut v; **b = 1; **a }", "E0502"),
            ("unsafe", "unsafe fn danger() {} pub fn invalid() { danger(); }", "E0133"),
        ]:
            fixture.write_text(original + "\n" + text)
            result = run(probe, fixture, source)
            assert result.returncode != 0 and error in result.stderr, (name, result.stderr)
            assert SUCCESS not in result.stdout and REQUIRED not in result.stdout
        for name, root_text, source_text in [
            ("callee", original, leaf.replace("let owner = produce(value);", "let owner = produce_return(value);", 1)),
            ("inventory", original.replace("pub fn producer_moved(", "pub fn renamed("), leaf),
            ("leaf_effect", original, leaf.replace("pub fn relay_direct(owner: Box<i32>) -> Box<i32> {\n    owner\n}", "pub fn relay_direct(owner: Box<i32>) -> Box<i32> { Box::new(*owner) }")),
            ("empty", "// No graphs\n", "// No leaf\n"),
            ("missing_leaf", original, leaf.replace("pub fn produce(value: i32) -> Box<i32> {\n    Box::new(value)\n}", "pub fn produce(value: i32) -> Box<i32> { Box::new(value + 1) }")),
            ("return_form", original, leaf.replace("return *moved;", "return *moved + 1;", 1)),
        ]:
            assert (root_text, source_text) != (original, leaf), name
            fixture.write_text(root_text)
            source.write_text(source_text)
            result = run(probe, fixture, source)
            assert result.returncode != 0 and SUCCESS not in result.stdout, name
            assert "panicked at" in result.stderr and "error[E" not in result.stderr, result.stderr
        fixture.write_text(original)
        old = "pub fn aliased(value: i32) -> i32 {\n    let owner = build(value);\n    *owner\n}"
        assert old in leaf
        for count, accepted in [(128, True), (129, False)]:
            body = "pub fn aliased(value: i32) -> i32 { let v0 = build(value);\n"
            body += "\n".join(f"let v{i} = v{i-1};" for i in range(1, count))
            body += f"\n*v{count-1}\n}}"
            source.write_text(leaf.replace(old, body))
            result = run(probe, fixture, source)
            assert (result.returncode == 0) == accepted, result.stdout + result.stderr
            assert (SUCCESS in result.stdout) == accepted
    print("owned-call source and budget controls passed")


if __name__ == "__main__":
    main()
