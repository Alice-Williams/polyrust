"""Canonical witness and actual typed dataflow probes must not change emission."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys
from addition_composition import check
from constant_export_scratch import writable_copy


def contents(path):
    return {str(file.relative_to(path)): file.read_bytes() for file in path.rglob("*") if file.is_file()}


def main():
    c_probe, c, java_probe, java, fixture, zig, c_bundle, java_bundle = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "addition-ast"
    work.mkdir()
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            destination = work / (language + "-" + label)
            destination.mkdir()
            output = destination / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            observed = Counter(line for line in result.stderr.splitlines() if line.startswith("ADDITION_"))
            wanted = Counter({f"ADDITION_INPUT\t{form}\t{width}": 4 if form == "method" else 1
                              for form in ["method", "associated"] for width in ["I32", "I64"]})
            wanted.update({f"ADDITION_AST\t{language}\t{width}": 5 for width in ["I32", "I64"]})
            wanted.update({"ADDITION_DETACHED\t" + language: 10})
            assert observed == (wanted if label == "probe" else Counter()), (language, label, observed)
            outputs.append(contents(destination))
        assert outputs[0] == outputs[1], language
        composition = work / (language + "-composition")
        writable_copy(Path(c_bundle if language == "c" else java_bundle), composition)
        check(composition, language, zig)
    print("10 observations/target: canonical builtin/context/width, copied and swapped witness controls, "
          "exact target expression and expanded dataflow, ordered original calls and byte-identical emission; native composition")


if __name__ == "__main__":
    main()
