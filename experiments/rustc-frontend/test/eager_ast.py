"""Read-only typed probes must leave the actual generated package unchanged."""
import os
from pathlib import Path
import subprocess
import sys


def contents(directory):
    return {str(path.relative_to(directory)): path.read_bytes()
            for path in directory.rglob("*") if path.is_file()}


def main():
    c_probe, c, java_probe, java, values, leaf = sys.argv[1:]
    root = Path(os.environ["TEST_TMPDIR"]) / "eager-ast"
    root.mkdir()
    fixture = root / "fixture.rs"
    fixture.write_text(Path(values).read_text() + "\nmod eager_leaf {\n" + Path(leaf).read_text() + "\n}\n")
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        results = []
        for label, adapter in [("probe", probe), ("production", production)]:
            work = root / (language + "-" + label)
            work.mkdir()
            output = work / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, str(fixture), str(output), "--package"],
                                    capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            markers = [line for line in result.stderr.splitlines() if line == "EAGER_AST\t" + language]
            assert len(markers) == (15 if label == "probe" else 0), (language, label, result.stderr)
            results.append(contents(work))
        assert results[0] == results[1], (language, "probe changed generated artifacts")
    print("15 eager AST projections per target preserve operators, types, promotions/precedence and exact production bytes")


if __name__ == "__main__":
    main()
