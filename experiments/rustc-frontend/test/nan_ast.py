"""Canonical checked inputs, exact target inequality nodes and unchanged emission."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys


def contents(path):
    return {str(file.relative_to(path)): file.read_bytes() for file in path.rglob("*") if file.is_file()}


def main():
    c_probe, c, java_probe, java, fixture = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "nan-ast"
    work.mkdir()
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            destination = work / (language + "-" + label)
            destination.mkdir()
            output = destination / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            observed = Counter(line for line in result.stderr.splitlines() if line.startswith("NAN_"))
            wanted = Counter({"NAN_AST\t" + language + "\tF64": 7,
                              "NAN_INPUT\tmethod\tF64": 5, "NAN_INPUT\tassociated\tF64": 2,
                              "NAN_DETACHED\t" + language: 2})
            assert observed == (wanted if label == "probe" else Counter()), (language, label, observed)
            outputs.append(contents(destination))
        assert outputs[0] == outputs[1], language
    print("7 observations per target: canonical HIR/context, copied-HIR controls, exact self-comparison "
          "types/precedence, once-only original calls and byte-identical production output")


if __name__ == "__main__":
    main()
