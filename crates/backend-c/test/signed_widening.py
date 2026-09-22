"""Separately compiled certified owners, exact integer truth and safe faults."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT, ZIG, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from widening_oracle import CASES, result, faulty, inputs

STEMS = ["polyrust_dep_811", "polyrust_widening_812", "polyrust_widening_813"]
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char",
         "-fno-short-enums"]
CLIENT = "".join(f'#include "{stem}.h"\n' for stem in STEMS) + r"""
#include <inttypes.h>
#include <stdio.h>
int main(void) {
    int32_t input;
    while (scanf("%" SCNd32, &input) == 1) {
        (void)printf("%" PRId64 " %" PRId64 "\n",
            poly_widen_812(input), poly_widen_813(input));
    }
    return ferror(stdin) != 0;
}
"""


def run(command, data=None):
    output = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def expected(variant):
    values = [result(value) if variant == "valid" else faulty(value, variant)
              for value in CASES]
    return "".join(f"{value} {value}\n" for value in values)


def mutate(producer, variant):
    text = producer.read_text()
    locals_ = re.findall(r"\bint32_t\s+(\w+)\s*=", text)
    assert len(locals_) == 1, ("exact materialized operand", text)
    operand = locals_[0]
    pattern = r"\breturn\s+([^;]+);"
    original = re.findall(pattern, text)
    assert len(original) == 1 and "int64_t" in original[0] and operand in original[0]
    if variant == "zero_extend":
        replacement = f"(int64_t)(uint32_t){operand}"
    elif variant == "narrow":
        # Both unsigned-to-signed casts are representable; subtraction is i16-range.
        replacement = (f"(int64_t)((int32_t)(((uint32_t){operand}) & 32767u) - "
                       f"(int32_t)(((uint32_t){operand}) & 32768u))")
    else:
        assert variant == "zero"
        replacement = f"((int64_t){operand} & INT64_C(0))"
    changed, count = re.subn(pattern, "return " + replacement + ";", text)
    assert count == 1 and changed != text
    producer.write_text(changed)


def main():
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    files = [stem + suffix for stem in STEMS for suffix in [".c", ".h"]]
    assert {path.name for path in ROOT.iterdir()} == set(files)
    originals = {name: (ROOT / name).read_bytes() for name in files}
    truth, data = expected("valid"), inputs()
    for variant in ["valid", "zero_extend", "narrow", "zero"]:
        directory = ROOT / variant
        directory.mkdir()
        for name in files:
            shutil.copyfile(ROOT / name, directory / name)
        if variant != "valid":
            mutate(directory / (STEMS[1] + ".c"), variant)
        wanted = expected(variant)
        assert (wanted == truth) == (variant == "valid")
        (directory / "consumer.c").write_text(CLIENT)
        for compiler in ["gcc-14", ZIG]:
            for optimization in ["0", "2"]:
                for sanitized in [False, True]:
                    if sanitized and (compiler != "gcc-14" or optimization != "2"):
                        continue
                    label = Path(compiler).name + optimization + str(sanitized)
                    flags = FLAGS + ["-O" + optimization]
                    if sanitized:
                        flags += ["-fsanitize=undefined", "-fno-sanitize-recover=all"]
                    objects = []
                    for stem in [*STEMS, "consumer"]:
                        obj = directory / (label + stem + ".o")
                        run([compiler, *flags, "-c", directory / (stem + ".c"), "-o", obj])
                        objects.append(obj)
                    for stem in STEMS:
                        header = directory / (stem + "_header.c")
                        header.write_text(f'#include "{stem}.h"\n')
                        run([compiler, *flags, "-c", header,
                             "-o", directory / (label + stem + "_header.o")])
                    binary = directory / label
                    run([compiler, *flags, *objects, "-o", binary])
                    output = run([binary], data)
                    assert output.stdout == wanted, (variant, label, "exact signed values")
                    assert output.stderr == "", (variant, label, "sanitizer/diagnostics")
    assert originals == {name: (ROOT / name).read_bytes() for name in files}
    print(f"{len(CASES)} inputs, both owners; GCC14/Zig O0/O2 + GCC UBSan; "
          "standalone headers; three safe compiling faults killed; originals unchanged")


if __name__ == "__main__":
    main()
