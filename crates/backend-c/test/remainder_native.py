"""Certified C fmod packages, exact values and independently observed call traces."""
from pathlib import Path
import re
import subprocess
import sys

ROOT, ZIG, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from arithmetic_oracle import observed
from remainder_oracle import result
from remainder_faults import changed
from remainder_cases import PAIRS
from short_circuit_mutations import instrument, definition

LEFT = "poly_fn_0000000000000259_000000000000000a"
RIGHT = "poly_fn_0000000000000259_000000000000000b"
STEMS = ["polyrust_dep_601", "polyrust_remainder_602", "polyrust_remainder_603"]
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off", "-fno-builtin",
         "-fsigned-char", "-fno-short-enums"]


def run(command, inputs=None):
    output = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def mutate(text, variant):
    _, start, end = definition(text, "poly_remainder_602_0")
    body = text[start:end]
    if variant in ["dropped", "duplicated", "reversed"]:
        declarations = []
        for target in [LEFT, RIGHT]:
            found = list(re.finditer(r"double\s+(\w+)\s*=\s*" + target + r"\((\w+)\);", body))
            assert len(found) == 1, (variant, body)
            declarations.append(found[0])
        a, b = declarations
        if variant == "dropped":
            body = body[:a.start()] + f"double {a.group(1)} = {a.group(2)};" + body[a.end():]
        elif variant == "duplicated":
            body = f"\n(void){LEFT}({a.group(2)});\n" + body
        else:
            assert a.end() < b.start()
            body = body[:a.start()] + b.group(0) + body[a.end():b.start()] + a.group(0) + body[b.end():]
    else:
        found = list(re.finditer(r"return\s+fmod\((\w+),\s*(\w+)\);", body))
        assert len(found) == 1, (variant, body)
        item = found[0]
        a, b = item.groups()
        call = f"fmod({a}, {b})"
        replacement = {
            "nearest": f"remainder({a}, {b})",
            "swapped": f"fmod({b}, {a})",
            "zero_sign": f"({call} == 0.0 ? 0.0 : {call})",
        }[variant]
        body = body[:item.start()] + "return " + replacement + ";" + body[item.end():]
    return text[:start] + body + text[end:]


def consumer():
    return "".join(f'#include "{stem}.h"\n' for stem in STEMS) + r"""
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
static void observe(double value) {
    uint64_t bits; (void)memcpy(&bits, &value, sizeof bits);
    if ((bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000)) (void)puts("nan");
    else (void)printf("%016" PRIx64 "\n", bits);
}
int main(void) {
    uint64_t a, b;
    while (scanf("%" SCNx64 " %" SCNx64, &a, &b) == 2) {
        double left, right;
        (void)memcpy(&left, &a, sizeof left);
        (void)memcpy(&right, &b, sizeof right);
        observe(poly_remainder_602_0(left, right));
        observe(poly_remainder_603_0(left, right));
    }
    return 0;
}
"""


def main():
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    inputs = "".join(f"{a:016x} {b:016x}\n" for a, b in PAIRS)
    truth = "\n".join(observed(result(a, b)) for a, b in PAIRS for _ in range(2)) + "\n"
    for variant in ["plain", "traced", "nearest", "swapped", "zero_sign", "dropped", "duplicated", "reversed"]:
        directory = ROOT / variant
        directory.mkdir()
        wanted = truth
        if variant in ["nearest", "swapped", "zero_sign"]:
            wanted = "\n".join(observed(changed(a, b, variant)) for a, b in PAIRS for _ in range(2)) + "\n"
            assert wanted != truth, variant
        trace = "" if variant == "plain" else {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB") * 2 * len(PAIRS)
        if variant in ["dropped", "duplicated", "reversed"]:
            assert wanted == truth and trace != "AB" * 2 * len(PAIRS)
        for index, stem in enumerate(STEMS):
            text = (ROOT / (stem + ".c")).read_text()
            if index == 0 and variant != "plain":
                text = instrument(text, {"A": LEFT, "B": RIGHT}, False)
            if index == 1 and variant not in ["plain", "traced"]:
                text = mutate(text, variant)
            (directory / (stem + ".c")).write_text(text)
        (directory / "consumer.c").write_text(consumer())
        for compiler in ["gcc-14", ZIG]:
            for optimization in ["0", "2"]:
                label = Path(compiler).name + optimization
                objects = []
                for stem in [*STEMS, "consumer"]:
                    obj = directory / (label + stem + ".o")
                    run([compiler, *FLAGS, "-O" + optimization, "-I", ROOT,
                         "-c", directory / (stem + ".c"), "-o", obj])
                    objects.append(obj)
                binary = directory / label
                if compiler == "gcc-14" and variant == "plain":
                    missing = subprocess.run([compiler, *objects, "-o", str(binary)], capture_output=True, text=True, timeout=90)
                    assert missing.returncode != 0 and "fmod" in missing.stderr
                run([compiler, *objects, "-lm", "-o", binary])
                output = run([binary], inputs)
                assert output.stdout == wanted, (variant, label, "exact remainder values")
                assert output.stderr == trace, (variant, label, "original operand evaluation")
    print(f"{len(PAIRS)*2} exact results/run; GCC14/Zig O0/O2; original imports; "
          "three value faults and three value-preserving trace faults; missing-link control")


if __name__ == "__main__":
    main()
