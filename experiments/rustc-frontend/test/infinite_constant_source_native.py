"""Original Rust versus source-owned C/Java, aliases and compiling sign faults."""
import os
from pathlib import Path
import re
import sys

from constant_export_native import java_path
from constant_export_scratch import writable_copy
from infinite_constant_oracle import faulty
from infinite_constant_source_truth import CONSTANTS, READS, text
from infinite_constant_source_inventory import inspect
from finite_constant_source_consumers import run, jdk, c_observe, java_observe, privacy


def descriptors(evidence, mixed):
    root, owners, _, _, bindings, constants, jc, values, cf, jf = evidence
    entries = []
    if mixed:
        for name, bits in READS.items():
            identity = bindings[root, "value", "read_" + name]
            defining = (bindings[owners["constants"], "value", name.upper()]
                        if name.upper() in CONSTANTS else None)
            entries.append((cf[identity]["symbol"] + "()",
                            java_path(jf[identity]["target"]["path"]) + "()", bits, defining))
        expected = {name: CONSTANTS[name] for name in ["NAMED_NEGATIVE", "NAMED_POSITIVE"]}
        expected.update(RENAMED=CONSTANTS["NAMED_POSITIVE"], OWN=CONSTANTS["NAMED_NEGATIVE"])
    else:
        expected = dict(CONSTANTS, OTHER_NEGATIVE=CONSTANTS["NAMED_NEGATIVE"],
                        OTHER_SAME=CONSTANTS["NAMED_POSITIVE"], AGAIN=CONSTANTS["NAMED_POSITIVE"])
    for name, bits in expected.items():
        module = bindings[root, "type", "nested"] if name == "AGAIN" else root
        identity = bindings[module, "value", name]
        assert values[identity] == bits
        entries.append((constants[identity][1]["symbol"],
                        java_path(jc[identity][1]["target"]["path"]), bits, identity))
    return entries


def replace_constant(directory, language, api, row, bits):
    sign = "-" if bits >> 63 else ""
    exponent, fraction = (bits >> 52) & 2047, bits & ((1 << 52) - 1)
    if exponent == 2047:
        assert fraction == 0
        literal = sign + ("HUGE_VAL" if language == "c" else "Double.POSITIVE_INFINITY")
    else:
        literal = f"{sign}0x{int(exponent != 0)}.{fraction:013x}p{(exponent or 1) - 1023:+d}"
    symbol = row["symbol"] if language == "c" else row["target"]["path"]["member"]
    source = directory / api["implementation" if language == "c" else "source"]
    before = source.read_text()
    after, count = re.subn(r"(\b" + re.escape(symbol) + r"\s*=\s*)[^;]+;",
                           lambda match: match[1] + literal + ";", before)
    assert count == 1 and before != after
    source.write_text(after)


def main():
    cm, jm, cr, jr, checked, optimized, zig = [Path(path).resolve() for path in sys.argv[1:]]
    for reference in [checked, optimized]:
        assert run([reference]) == text(READS.values())
    original = {p: p.read_bytes() for folder in [cm, jm, cr, jr] for p in folder.rglob("*") if p.is_file()}
    work = Path(os.environ["TEST_TMPDIR"]) / "infinite-source-native"
    work.mkdir()
    tools = jdk()
    observations = 0
    for mixed, c_dir, java_dir in [(False, cm, jm), (True, cr, jr)]:
        evidence = inspect(c_dir, java_dir, mixed)
        root, owners, c, java, bindings, constants, jc, _, cf, jf = evidence
        entries = descriptors(evidence, mixed)
        observations += len(entries)
        order = [owners[name] for name in ["constants", "second", "middle"] + (["root"] if mixed else [])]
        for owner in order:
            implementation = (c_dir / c[owner]["implementation"]).read_text()
            assert implementation.count("#include <math.h>") == int(owner != owners["middle"])
            assert "#include <math.h>" not in (c_dir / c[owner]["header"]).read_text()
            assert c[owner].get("system_libraries", []) == (["m"] if mixed and owner == root else [])
        identity = bindings[owners["constants"], "value", "BITS_NEGATIVE"]
        for variant in ["valid", "sign_loss", "finite_clamp", "zero"]:
            changed = CONSTANTS["BITS_NEGATIVE"] if variant == "valid" else faulty(CONSTANTS["BITS_NEGATIVE"], variant)
            expected = text([changed if defining == identity else bits for _, _, bits, defining in entries])
            assert (expected == text([row[2] for row in entries])) == (variant == "valid")
            for language, source, apis, rows, index in [
                ("c", c_dir, c, constants, 0), ("java", java_dir, java, jc, 1),
            ]:
                directory = work / f"{mixed}-{language}-{variant}"
                writable_copy(source, directory)
                if variant != "valid":
                    replace_constant(directory, language, apis[owners["constants"]], rows[identity][1], changed)
                expressions = [row[index] for row in entries]
                if language == "c":
                    c_observe(directory, order, apis, expressions, expected, zig)
                else:
                    java_observe(directory, order, apis, expressions, expected, tools)
                if mixed and variant == "valid":
                    hidden, = [key for key, row in jf.items() if not row["externally_reachable"]]
                    public = bindings[root, "value", "read_private"]
                    paths = [cf[key]["symbol"] if language == "c" else java_path(jf[key]["target"]["path"])
                             for key in [public, hidden]]
                    privacy(directory, language, c[root]["header"], *paths, tools)
    assert original == {p: p.read_bytes() for p in original}
    print(f"{len(READS)} original Rust reads at both complete-crate profiles; {observations} target observations/configuration; "
          "GCC/Zig O0/O2 + UBSan; Java21 normal/-Xint; exact aliases/imports/docs/privacy; "
          "three compiling producer faults with recompiled dependents")


if __name__ == "__main__":
    main()
