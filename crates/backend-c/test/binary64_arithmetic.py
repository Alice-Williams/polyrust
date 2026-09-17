"""Native typed C operators against the shared exact arithmetic oracle."""
from pathlib import Path
import subprocess
import sys

root, zig, oracle = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(oracle))
from arithmetic_oracle import MAGNITUDE, Operation as Op, rational, encode, result, observed
from arithmetic_cases import PAIRS
from short_circuit_mutations import instrument
from arithmetic_mutations import LEFT, RIGHT, mutate

FMA_PAIR = (0x3ff0000000000000 + (1 << 25), 0x3ff0000000000000 - (1 << 26))
PAIRS = PAIRS + [FMA_PAIR]
NEGATIVE_ONE = 0xbff0000000000000


def results(left, right):
    basic = [result(left, right, operation) for operation in Op]
    return basic + [result(basic[0], right, Op.MULTIPLY),
                    result(basic[2], NEGATIVE_ONE, Op.ADD)]


def expected(variant):
    values = []
    for left, right in PAIRS:
        wanted = results(left, right)
        if variant == "operator":
            wanted[0] = result(left, right, Op.SUBTRACT)
        elif variant == "swapped":
            wanted[1] = result(right, left, Op.SUBTRACT)
        elif variant == "zero_sign" and wanted[0] & MAGNITUDE == 0:
            wanted[0] = 0
        elif variant == "grouping":
            wanted[4] = result(left, result(right, right, Op.MULTIPLY), Op.ADD)
        elif variant == "fused" and (left, right) == FMA_PAIR:
            # This fault probes contraction, not correctness of the entire foreign
            # fma library. Keep every ordinary arithmetic input fully checked.
            wanted[5] = encode(rational(left) * rational(right) - 1)
        # Middle then root, each calls the very same original operator owner.
        values.extend(observed(bits) for bits in wanted * 2)
    return "\n".join(values) + "\n"


def trace(variant):
    if variant == "plain":
        return ""
    first = {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB")
    return ((first + "AB" * 5) * 2) * len(PAIRS)


def run(command, inputs=None):
    output = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


consumer = """#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
#include "polyrust_arithmetic_502.h"
#include "polyrust_arithmetic_503.h"
static double decode(uint64_t bits) { double value; memcpy(&value, &bits, sizeof(value)); return value; }
static void observe(double value) {
    uint64_t bits; memcpy(&bits, &value, sizeof(bits));
    if ((bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000)) (void)puts("nan");
    else (void)printf("%016" PRIx64 "\\n", bits);
}
int main(void) {
    uint64_t left_bits, right_bits;
    while (scanf("%" SCNx64 " %" SCNx64, &left_bits, &right_bits) == 2) {
        double left = decode(left_bits), right = decode(right_bits);
""" + "".join(f"observe(poly_arithmetic_{owner}_{index}(left, right));\n"
              for owner in [502, 503] for index in range(6)) + "} return 0; }\n"


def main():
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    inputs = "".join(f"{left:016x} {right:016x}\n" for left, right in PAIRS)
    truth = expected("plain")
    for variant in ["plain", "traced", "operator", "swapped", "zero_sign",
                    "grouping", "fused", "dropped", "duplicated", "reversed"]:
        directory = root / variant
        directory.mkdir()
        sources = []
        for stem in ["polyrust_dep_501", "polyrust_arithmetic_502", "polyrust_arithmetic_503"]:
            text = (root / (stem + ".c")).read_text()
            assert "math.h" not in text
            if variant != "plain" and stem == "polyrust_dep_501":
                text = instrument(text, {"A": LEFT, "B": RIGHT}, False)
            if variant not in ("plain", "traced") and stem == "polyrust_arithmetic_502":
                text = mutate(text, variant)
            path = directory / (stem + ".c")
            path.write_text(text)
            sources.append(path)
        client = directory / "consumer.c"
        client.write_text(consumer)
        wanted = expected(variant)
        if variant in ("operator", "swapped", "zero_sign", "grouping", "fused"):
            assert wanted != truth, variant
        else:
            assert wanted == truth
        if variant in ("dropped", "duplicated", "reversed"):
            assert trace(variant) != trace("traced")
        for compiler in ["gcc-14", zig]:
            for optimization in ["0", "2"]:
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fno-fast-math",
                         "-ffp-contract=off", "-fsigned-char", "-fno-short-enums",
                         "-fno-builtin", "-O" + optimization, "-I", root]
                objects = []
                for index, source in enumerate([*sources, client]):
                    obj = directory / (str(index) + ".o")
                    run([compiler, *flags, "-c", source, "-o", obj])
                    objects.append(obj)
                executable = directory / "consumer"
                # Only the deliberate fused-library mutant needs libm.
                run([compiler, *objects, *(["-lm"] if variant == "fused" else []), "-o", executable])
                output = run([executable], inputs)
                if output.stdout != wanted:
                    actual_lines, wanted_lines = output.stdout.splitlines(), wanted.splitlines()
                    mismatch = next((index for index, pair in enumerate(zip(actual_lines, wanted_lines))
                                     if pair[0] != pair[1]), min(len(actual_lines), len(wanted_lines)))
                    raise AssertionError((variant, compiler, optimization, "values", mismatch,
                                          PAIRS[mismatch // 12], actual_lines[mismatch:mismatch + 1],
                                          wanted_lines[mismatch:mismatch + 1]))
                assert output.stderr == trace(variant), (variant, compiler, optimization, "trace")
    print(f"{len(PAIRS) * 12} exact results per GCC/Zig O0/O2 run; "
          "original/importing owners, no baseline libm, eight compiling value/trace faults")


if __name__ == "__main__":
    main()
