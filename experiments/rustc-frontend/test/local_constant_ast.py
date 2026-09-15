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
    root = Path(os.environ["TEST_TMPDIR"]) / "local-constant-ast"
    root.mkdir()
    fixture = root / "fixture.rs"
    fixture.write_text(Path(values).read_text() + "\nmod local_constant_leaf {\n" + Path(leaf).read_text() + "\n}\n")
    for language, probe, production in [("c", c_probe, c), ("java", java_probe, java)]:
        results = []
        for label, adapter in [("probe", probe), ("production", production)]:
            work = root / (language + "-" + label)
            work.mkdir()
            output = work / ("package" if language == "c" else "Generated.java")
            result = subprocess.run([adapter, str(fixture), str(output), "--package"],
                                    capture_output=True, text=True, timeout=90)
            assert result.returncode == 0, (language, label, result.stderr)
            reads = [line.split("\t")[2:] for line in result.stderr.splitlines()
                     if line.startswith("CONSTANT_AST\t" + language + "\t")]
            declarations = [line.split("\t")[2:] for line in result.stderr.splitlines()
                            if line.startswith("LOCAL_CONSTANT_AST\t" + language + "\t")]
            assert len(reads) == (20 if label == "probe" else 0), (language, result.stderr)
            assert len(declarations) == (21 if label == "probe" else 0), (language, result.stderr)
            if label == "probe":
                assert len({origin for origin, _ in declarations}) == 21, declarations
                shadowed = [(origin, value) for origin, value in declarations if "::shadow::" in origin]
                assert len(shadowed) == 3 and {v for _, v in shadowed} == {"I32(47)", "I32(41)", "I32(43)"}, shadowed
                assert any("unused::UNUSED" in origin and value == "I64(-9223372036854775808)" for origin, value in declarations)
                assert not any("unused::UNUSED" in origin for origin, _ in reads)
                assert {value for origin, value in reads if "shadow::" in origin} == {"I32(47)", "I32(41)", "I32(43)"}, reads
            results.append(contents(work))
        assert results[0] == results[1], (language, "probe changed generated artifacts")
    print("21 declaration and 20 read projections per target preserve origin, runtime state and production bytes")


if __name__ == "__main__":
    main()
