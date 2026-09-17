"""Probe all unit operations and prove observation does not change output."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import sys

def contents(path):
    return {str(file.relative_to(path)): file.read_bytes() for file in path.rglob("*") if file.is_file()}

def main():
    c_probe, c, java_probe, java, leaf, relay, values = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "unit-ast"
    work.mkdir()
    fixture = work / "fixture.rs"
    fixture.write_text(Path(values).read_text()
                       + "\nmod unit_leaf {\n" + Path(leaf).read_text() + "\n}\n"
                       + "\nmod unit_relay {\nuse crate::unit_leaf;\n" + Path(relay).read_text().replace("//! Transitive effect calls retain producer identities.", "") + "\n}\n")
    expected = Counter({"Empty": 2, "Call": 13, "Block": 2, "Conditional": 5})
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        outputs = []
        for label, adapter in [("probe", probe), ("production", production)]:
            root = work / (language + "-" + label)
            root.mkdir()
            output = root / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, fixture, output, "--package"], capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            observed = Counter(line.split("\t")[2] for line in result.stderr.splitlines()
                               if line.startswith("UNIT_AST\t" + language + "\t"))
            assert observed == (expected if label == "probe" else Counter()), (language, label, observed)
            completed = [line for line in result.stderr.splitlines() if line == "UNIT_COMPLETION\t" + language]
            assert len(completed) == (7 if label == "probe" else 0), (language, label, completed)
            outputs.append(contents(root))
        assert outputs[0] == outputs[1], language
    print("22 unit AST observations per target: exact callable identity, argument temporaries, 7 bare-return completions, no effect returns, identical production bytes")

if __name__ == "__main__":
    main()
