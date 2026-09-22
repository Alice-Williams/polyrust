"""Source-native exact bits, aliases and compiling producer-value faults."""
import os
from pathlib import Path
import sys

from constant_export_native import java_path
from constant_export_scratch import writable_copy
from finite_constant_faults import faulty
from finite_constant_source_truth import CONSTANTS, READS, text
from finite_constant_source_inventory import inspect
from finite_constant_source_consumers import run, jdk, c_observe, java_observe, replace_constant, privacy


def descriptors(evidence, mixed):
    root, owners, _, _, bindings, constants, jc, values, cf, jf = evidence
    entries = []
    if mixed:
        for name, bits in READS.items():
            identity = bindings[root, "value", "read_" + name]
            defining = None
            if name.upper() in CONSTANTS:
                defining = bindings[owners["constants"], "value", name.upper()]
            elif name == "other_tenth":
                defining = bindings[owners["second"], "value", "TENTH"]
            elif name == "other_same":
                defining = bindings[owners["second"], "value", "SAME_VALUE"]
            elif name == "alias":
                defining = bindings[owners["constants"], "value", "TENTH"]
            elif name in ["own", "unused"]:
                defining = bindings[root, "value", "OWN"]
            entries.append((cf[identity]["symbol"] + "()",
                            java_path(jf[identity]["target"]["path"]) + "()", bits, defining))
        names = ["NEGATIVE_ZERO", "POSITIVE_ZERO", "TENTH", "RENAMED", "OWN"]
        expected = {"NEGATIVE_ZERO": CONSTANTS["NEGATIVE_ZERO"], "POSITIVE_ZERO": CONSTANTS["POSITIVE_ZERO"],
                    "TENTH": CONSTANTS["TENTH"], "RENAMED": CONSTANTS["TENTH"],
                    "OWN": CONSTANTS["INDEXED"]}
    else:
        names = [*CONSTANTS, "OTHER_TENTH", "OTHER_SAME", "AGAIN"]
        expected = dict(CONSTANTS, OTHER_TENTH=CONSTANTS["TENTH"], OTHER_SAME=CONSTANTS["TENTH"], AGAIN=CONSTANTS["TENTH"])
    for name in names:
        module = bindings[root, "type", "nested"] if name == "AGAIN" else root
        identity = bindings[module, "value", name]
        assert values[identity] == expected[name]
        entries.append((constants[identity][1]["symbol"],
                        java_path(jc[identity][1]["target"]["path"]), expected[name], identity))
    return entries


def main():
    cm, jm, cr, jr, checked, optimized, zig = [Path(p).resolve() for p in sys.argv[1:]]
    for reference in [checked, optimized]:
        assert run([reference]) == text(READS.values())
    originals = {p: p.read_bytes() for folder in [cm, jm, cr, jr] for p in folder.rglob("*") if p.is_file()}
    work = Path(os.environ["TEST_TMPDIR"]) / "finite-source-native"
    work.mkdir()
    tools = jdk()
    for mixed, c_dir, java_dir in [(False, cm, jm), (True, cr, jr)]:
        evidence = inspect(c_dir, java_dir, mixed)
        root, owners, c, java, bindings, constants, jc, _, cf, jf = evidence
        entries = descriptors(evidence, mixed)
        order = [owners[name] for name in ["constants", "second", "middle"] + (["root"] if mixed else [])]
        for variant in ["valid", "zero_sign", "f32", "wrong_value"]:
            name = {"valid": "TENTH", "zero_sign": "NEGATIVE_ZERO", "f32": "TENTH",
                    "wrong_value": "ONE_ULP"}[variant]
            identity = bindings[owners["constants"], "value", name]
            changed = CONSTANTS[name] if variant == "valid" else faulty(CONSTANTS[name], variant)
            expected = text([changed if original == identity else bits for _, _, bits, original in entries])
            assert (expected == text([row[2] for row in entries])) == (variant == "valid")
            for language, original, apis, rows, index in [
                ("c", c_dir, c, constants, 0), ("java", java_dir, java, jc, 1),
            ]:
                directory = work / f"{mixed}-{language}-{variant}"
                writable_copy(original, directory)
                if variant != "valid":
                    replace_constant(directory, language, apis[owners["constants"]], rows[identity][1], changed)
                expressions = [row[index] for row in entries]
                if language == "c":
                    c_observe(directory, order, apis, expressions, expected, zig)
                else:
                    java_observe(directory, order, apis, expressions, expected, tools)
                if mixed and variant == "valid":
                    hidden, = [identity for identity, row in jf.items() if not row["externally_reachable"]]
                    public = bindings[root, "value", "read_private"]
                    paths = [cf[identity]["symbol"] if language == "c" else java_path(jf[identity]["target"]["path"])
                             for identity in [public, hidden]]
                    privacy(directory, language, c[root]["header"], *paths, tools)
    assert originals == {p: p.read_bytes() for p in originals}
    print("34 original Rust reads; 62 target observations/configuration; alias-only and mixed crates; "
          "GCC/Zig O0/O2 + UBSan, Java21 normal/-Xint, three compiling producer faults")


if __name__ == "__main__":
    main()
