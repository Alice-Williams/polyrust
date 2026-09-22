"""Real call/field traces and actual compiling faults, compared to independent truth."""
import os
from pathlib import Path
import subprocess
import sys

from character_source_clients import c_client, java_client
from character_source_faults import expected, instrument, mutate
from character_source_inventory import inspect
from character_source_native import compile_c, compile_java
from character_source_truth import corpus
from constant_export_scratch import writable_copy
from finite_constant_source_consumers import jdk


def main():
    c_dir, j_dir, zig = [Path(p).resolve() for p in sys.argv[1:]]
    evidence = inspect(c_dir, j_dir)
    _, owners, c, java, _, _, _ = evidence
    order = [owners[name] for name in ["leaf", "middle", "root"]]
    original = {p: p.read_bytes() for d in [c_dir, j_dir] for p in d.rglob("*") if p.is_file()}
    work = Path(os.environ["TEST_TMPDIR"]) / "character-source-observations"
    work.mkdir()
    data, truth, _ = corpus(False)
    baseline, baseline_trace = expected("trace")
    assert truth == baseline
    tools = jdk()
    for variant in ["trace", "byte", "utf16", "reverse", "field_order"]:
        wanted, wanted_trace = expected(variant)
        assert (wanted == baseline and wanted_trace == baseline_trace) == (variant == "trace")
        if variant == "field_order":
            assert wanted == baseline and wanted_trace != baseline_trace
        for language, source, apis in [("c", c_dir, c), ("java", j_dir, java)]:
            directory = work / f"{language}-{variant}"
            writable_copy(source, directory)
            instrument(directory, language, evidence)
            if variant != "trace":
                mutate(directory, language, evidence, variant)
            client, code = ("consumer.c", c_client(evidence)) if language == "c" else ("Consumer.java", java_client(evidence))
            (directory / client).write_text(code)
            commands = compile_c(directory, apis, order, zig) if language == "c" else compile_java(directory, apis, order, tools)
            for command in commands:
                result = subprocess.run([str(arg) for arg in command], input=data,
                                        capture_output=True, timeout=180)
                assert result.returncode == 0, (language, variant, result.stderr)
                assert result.stdout == wanted, (language, variant, "value mismatch")
                assert result.stderr == wanted_trace, (language, variant, "trace mismatch", result.stderr[:500], wanted_trace[:500])
    assert original == {p: p.read_bytes() for p in original}
    print("Measured source-owned calls/field order: all original bodies retained; "
          "byte/UTF16 narrowing, reversed comparisons and reordered field evaluation all compile, "
          "and all differ from the original semantic contract on GCC/Zig and Java21")


if __name__ == "__main__":
    main()
