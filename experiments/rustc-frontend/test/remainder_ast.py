"""Canonical source and exact target nodes; probe never changes emitted bytes."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys
from remainder_composition import check
from constant_export_scratch import writable_copy


def contents(path):
    return {str(file.relative_to(path)): file.read_bytes() for file in path.rglob("*") if file.is_file()}


def main():
    c_probe, c, java_probe, java, fixture, zig, c_bundle, java_bundle = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "remainder-ast"
    work.mkdir()
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            destination = work / (language + "-" + label)
            destination.mkdir()
            output = destination / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            observed = Counter(line for line in result.stderr.splitlines() if line.startswith("REMAINDER_"))
            wanted = Counter({"REMAINDER_DETACHED\t" + language: 13,
                              "REMAINDER_INPUT\tcanonical": 13,
                              "REMAINDER_AST\t" + language: 13})
            assert observed == (wanted if label == "probe" else Counter()), (language, label, observed)
            outputs.append(contents(destination))
        assert outputs[0] == outputs[1], language
        composition = work / (language + "-composition")
        writable_copy(Path(c_bundle if language == "c" else java_bundle), composition)
        check(composition, language, zig)
    print("13 observations/target: canonical HIR/context, copied-node controls, exact operators/precedence, "
          "expanded dataflow and original once-only ordered calls; byte-identical emission")


if __name__ == "__main__":
    main()
