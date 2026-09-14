"""Independent native Rust/C/Java execution of the same source fixture."""
import os
from pathlib import Path
import re
import subprocess
import sys


def run(command, *, inputs=None):
    result = subprocess.run(command, input=inputs, capture_output=True, text=True, check=False)
    assert result.returncode == 0, (command, result.returncode, result.stdout, result.stderr)
    return result.stdout


def main():
    adapter, rust, c0, c2, source, seed, consumer, artifact, *extras = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "java-source"
    work.mkdir()
    flags = [argument for extra in extras for argument in ["--input", extra]]
    generated = work / "Generated.java"
    run([adapter, source, str(generated), *flags])
    second = work / "again" / "Generated.java"
    second.parent.mkdir()
    run([adapter, source, str(second), *flags])
    assert generated.read_bytes() == second.read_bytes(), "cross-invocation output changed"
    assert generated.read_bytes() == Path(artifact).read_bytes(), "Bazel generation artifact differs"
    text = generated.read_text(encoding="utf-8")
    matches = re.findall(r"^package (org\.polyrust\.generated\.r[0-9a-f]{16});$", text, re.MULTILINE)
    assert len(matches) == 1, "expected exactly one canonical crate package"
    consumer_source = work / "JavaSourceConsumer.java"
    consumer_source.write_text(Path(consumer).read_text().replace("__PACKAGE__", matches[0]), encoding="utf-8")
    runtimes = [path for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(runtimes) == 1, runtimes
    runtime = runtimes[0] / "bin"
    classes = work / "classes"
    classes.mkdir()
    run([str(runtime / "javac"), "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
         "-d", str(classes), str(generated), str(consumer_source)])
    values = Path(seed).read_text().splitlines() + [str(value) for value in range(-4096, 4097)]
    assert len(values) == 8204
    inputs = "\n".join(values) + "\n"
    expected = run([rust], inputs=inputs)
    assert len(expected.splitlines()) == len(values)
    if Path(source).stem == "boolean_order":
        predicates = [lambda a, b: a == b, lambda a, b: a != b, lambda a, b: a < b,
                      lambda a, b: a <= b, lambda a, b: a > b, lambda a, b: a >= b]
        answers = dict(zip(values, expected.splitlines()))
        index = 0
        for predicate in predicates:
            for left in [False, True]:
                for right in [False, True]:
                    result = index + 1 if predicate(left, right) else -(index + 1)
                    assert answers[str(index)] == str(result), (index, left, right)
                    index += 1
        assert index == 24
    for executable in [c0, c2]:
        assert run([executable], inputs=inputs) == expected, executable
    actual = run([str(runtime / "java"), "-cp", str(classes), "JavaSourceConsumer"], inputs=inputs)
    assert actual == expected, "native Java behavior differs from native Rust"
    print("8204 native Rust/C-O0/C-O2/Java results agree; Java 21 lint and deterministic generation pass")


if __name__ == "__main__":
    main()
