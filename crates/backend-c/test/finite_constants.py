"""Independent bits from separately compiled certified constant objects."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[-1]).resolve()))
from finite_constant_oracle import expected_values
from finite_constant_faults import faulty

VALUES = expected_values()
CONTROLS = [0, 1 << 63, 1, 0x0010000000000000, 0x3ff0000000000001,
            0x3fb999999999999a, 0x7fefffffffffffff, 0xffefffffffffffff]
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char", "-fno-short-enums"]


def run(command):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    assert output.stderr == "", (command, output.stderr)
    return output.stdout


def client(count, readers):
    header = "polyrust_constant_facade_921.h" if readers else "polyrust_constants.h"
    text = f'#include "{header}"\n'
    if readers:
        text += '#include "polyrust_constant_reader_922.h"\n'
    text += r"""
#include <inttypes.h>
#include <stdio.h>
#include <string.h>
static uint64_t bits(double value) {
    uint64_t result;
    _Static_assert(sizeof(result) == sizeof(value), "binary64");
    memcpy(&result, &value, sizeof(result));
    return result;
}
int main(void) {
    const double *const objects[] = {
"""
    text += ",\n".join(f"&poly_finite_{index}" for index in range(count))
    text += "\n};\n"
    text += f'for (size_t i = 0; i < {count}; ++i) {{ (void)printf("%016" PRIx64 "\\n", bits(*objects[i])); }}\n'
    if readers:
        for index in range(count):
            text += f'(void)printf("%016" PRIx64 "\\n", bits(poly_import_922_{index}()));\n'
    return text + "return 0;\n}\n"


def literal(bits):
    sign = "-" if bits >> 63 else ""
    exponent, fraction = (bits >> 52) & 2047, bits & ((1 << 52) - 1)
    if exponent == 2047:
        assert fraction == 0  # Only overflow to infinity in the f32 fault.
        return sign + "INFINITY"
    return f"{sign}0x{int(exponent != 0):x}.{fraction:013x}p{(exponent or 1) - 1023:+d}"


def mutate(directory, variant):
    path = directory / "constants.c"
    original = path.read_text()
    def replace(match):
        index = int(match[2])
        return match[1] + literal(faulty(CONTROLS[index], variant)) + ";"
    changed, count = re.subn(r"(\bpoly_finite_(\d+)\s*=\s*)[^;]+;", replace, original)
    assert count == len(CONTROLS) and changed != original
    # Test-only nonfinite substitutions avoid an out-of-range C float cast.
    path.write_text('#include <math.h>\n' + changed)


def observe(directory, values, zig, readers=False):
    headers = sorted(directory.glob("*.h"))
    sources = sorted(directory.glob("*.c"))
    (directory / "client.c").write_text(client(len(values), readers))
    sources.append(directory / "client.c")
    expected = "".join(f"{value:016x}\n" for value in values) * (2 if readers else 1)
    for compiler, optimization, sanitized in [
            ("gcc-14", "0", False), ("gcc-14", "2", False),
            (zig, "0", False), (zig, "2", False), ("gcc-14", "2", True)]:
        flags = [*FLAGS, "-O" + optimization]
        if sanitized:
            flags += ["-fsanitize=undefined", "-fno-sanitize-recover=all"]
        objects = []
        for index, source in enumerate(sources):
            obj = directory / f"unit-{index}.o"
            run([compiler, *flags, "-c", source, "-o", obj])
            objects.append(obj)
        for header in headers:
            probe = directory / "header-check.c"
            probe.write_text(f'#include "{header.name}"\n')
            run([compiler, *flags, "-c", probe, "-o", directory / "header-check.o"])
        binary = directory / "observe"
        run([compiler, *flags, *objects, "-o", binary])
        assert run([binary]) == expected, (directory, compiler, optimization, sanitized)


def main():
    if sys.argv[1] == "inputs":
        print("\n".join(f"{value:016x}" for value in VALUES))
        return
    root, zig = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    assert run(["gcc-14", "-dumpfullversion"]).strip() == "14.2.0"
    for index, start in enumerate(range(0, len(VALUES), 384)):
        observe(root / str(index), VALUES[start:start + 384], zig)
    controls = root / "controls"
    originals = {path.name: path.read_bytes() for path in controls.iterdir()}
    for variant in ["valid", "zero_sign", "f32", "wrong_value"]:
        directory = root / variant
        directory.mkdir()
        for name in originals:
            shutil.copyfile(controls / name, directory / name)
        expected = CONTROLS if variant == "valid" else [faulty(value, variant) for value in CONTROLS]
        assert (expected == CONTROLS) == (variant == "valid")
        if variant != "valid":
            mutate(directory, variant)
        observe(directory, expected, zig, readers=True)
    assert originals == {path.name: path.read_bytes() for path in controls.iterdir()}
    print(f"{len(VALUES)} certified finite objects; GCC14/Zig O0/O2, GCC UBSan, standalone headers; "
          "alias facade reads; generated-reader controls; three compiling faults rejected")


if __name__ == "__main__":
    main()
