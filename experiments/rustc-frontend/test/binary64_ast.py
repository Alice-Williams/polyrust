"""Literal bit witnesses and non-vacuous identical probe/production output."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys
from binary64_oracle import LITERALS, COMPARISONS, OPERATORS


def contents(path):
    return {str(p.relative_to(path)): p.read_bytes() for p in path.rglob("*") if p.is_file()}


def main():
    c_probe, c, java_probe, java, source = sys.argv[1:]
    root = Path(os.environ["TEST_TMPDIR"]) / "binary64-ast"
    root.mkdir()
    fixture = root / "fixture.rs"
    fixture.write_text(Path(source).read_text() + "\nfn left(v:f64)->f64 {v}\nfn right(v:f64)->f64 {v}\n"
                       + "\n".join(f"pub fn {name}(a:f64,b:f64)->bool {{left(a) {operator} right(b)}}"
                                   for name, operator in zip(COMPARISONS, OPERATORS, strict=True)))
    expected = Counter(f"{bits:016x}" for bits in LITERALS.values())
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            destination = root / (language + label)
            destination.mkdir()
            output = destination / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            target = Counter(line.split("\t")[2] for line in result.stderr.splitlines()
                             if line.startswith("BINARY64_AST\t" + language + "\t"))
            compiler = Counter(line.split("\t")[1] for line in result.stderr.splitlines()
                               if line.startswith("BINARY64_INPUT\t"))
            assert target == compiler == (expected if label == "probe" else Counter())
            comparisons = Counter(line.split("\t")[2] for line in result.stderr.splitlines()
                                  if line.startswith("BINARY64_COMPARE\t" + language + "\t"))
            assert comparisons == (Counter(["Eq", "Ne", "Lt", "Le", "Gt", "Ge"]) if label == "probe" else Counter())
            outputs.append(contents(destination))
        assert outputs[0] == outputs[1]
    print("11 exact literal/compiler observations per target; wrong-context/copied-HIR controls; identical production bytes")


if __name__ == "__main__":
    main()
