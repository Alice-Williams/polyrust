"""Non-vacuous read-only compiler/AST observations and byte-identical emission."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys


def contents(path):
    return {str(file.relative_to(path)): file.read_bytes() for file in path.rglob("*") if file.is_file()}


def main():
    c_probe, c, java_probe, java, leaf, root = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "wrapping-ast"
    work.mkdir()
    fixture = work / "fixture.rs"
    fixture.write_text(Path(root).read_text() + "\nmod wrapping_leaf {\n"
                       + "\n".join(line for line in Path(leaf).read_text().splitlines() if not line.startswith("//!"))
                       + "\n}\n")
    expected = Counter({"I32": 6, "I64": 6})
    source_expected = Counter({("method", "I32"): 5, ("associated", "I32"): 1,
                               ("method", "I64"): 4, ("associated", "I64"): 2})
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            destination = work / (language + "-" + label)
            destination.mkdir()
            output = destination / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            observed = Counter(line.split("\t")[2] for line in result.stderr.splitlines()
                               if line.startswith("WRAPPING_AST\t" + language + "\t"))
            source = Counter(tuple(line.split("\t")[1:]) for line in result.stderr.splitlines()
                             if line.startswith("WRAPPING_INPUT\t"))
            assert observed == (expected if label == "probe" else Counter()), (language, label, observed)
            assert source == (source_expected if label == "probe" else Counter()), (language, label, source)
            outputs.append(contents(destination))
        assert outputs[0] == outputs[1], language
    print("12 observations per target: canonical input, wrong-context/copied-HIR controls, exact widths, "
          "C guard, Java negation, original receiver calls, byte-identical production packages")


if __name__ == "__main__":
    main()
