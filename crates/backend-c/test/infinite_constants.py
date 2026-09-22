"""Bit-for-bit infinity storage/reads; integer-only oracle and compiling faults."""
from pathlib import Path
import re
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[3]).resolve()))
from infinite_constant_oracle import INFINITY, SIGN, faulty

VALUES = [INFINITY, SIGN | INFINITY]
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char",
         "-fno-short-enums", "-fno-fast-math", "-ffp-contract=off"]
CLIENT = r'''
#include "polyrust_constant_facade_921.h"
#include "polyrust_constant_reader_922.h"
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
    (void)printf("%016" PRIx64 "\n", bits(poly_infinite_0));
    (void)printf("%016" PRIx64 "\n", bits(poly_infinite_1));
    (void)printf("%016" PRIx64 "\n", bits(poly_import_922_0()));
    (void)printf("%016" PRIx64 "\n", bits(poly_import_922_1()));
    return 0;
}
'''


def run(command):
    result = subprocess.run([str(arg) for arg in command], text=True,
                            capture_output=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    assert not result.stderr, (command, result.stderr)
    return result.stdout


def expected(values):
    return "".join(f"{value:016x}\n" for value in values) * 2


def observe(directory, compiler, flags):
    (directory / "client.c").write_text(CLIENT)
    objects = []
    for index, source in enumerate(sorted(directory.glob("*.c"))):
        obj = directory / f"unit-{index}.o"
        run([compiler, *flags, "-c", source, "-o", obj])
        objects.append(obj)
    for header in sorted(directory.glob("*.h")):
        probe = directory / "header-check.c"
        probe.write_text(f'#include "{header.name}"\n')
        run([compiler, *flags, "-c", probe, "-o", directory / "header-check.o"])
        probe.unlink()
    binary = directory / "observe"
    # No libm: a standard constant is not a math-library function call.
    run([compiler, *flags, *objects, "-o", binary])
    return run([binary])


def mutate(original, fault):
    tokens = {"sign_loss": ["HUGE_VAL", "HUGE_VAL"],
              "finite_clamp": ["0x1.fffffffffffffp1023", "(-0x1.fffffffffffffp1023)"],
              "zero": ["0x0p0", "0x0p0"]}[fault]
    changed, count = re.subn(r"(\bpoly_infinite_(\d+)\s*=\s*)[^;]+;",
                             lambda match: match[1] + tokens[int(match[2])] + ";", original)
    assert count == 2 and changed != original
    return changed


def main():
    root, zig = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    assert run(["gcc-14", "-dumpfullversion"]).strip() == "14.2.0"
    originals = {path.name: path.read_bytes() for path in root.iterdir() if path.is_file()}
    source = originals["constants.c"].decode()
    assert source.count("#include <math.h>") == 1
    assert re.search(r"\bpoly_infinite_0\s*=\s*HUGE_VAL;", source)
    assert re.search(r"\bpoly_infinite_1\s*=\s*\(-HUGE_VAL\);", source)
    for name, text in originals.items():
        if name != "constants.c":
            assert b"#include <math.h>" not in text
    cases = [("gcc-14", "0", False), ("gcc-14", "2", False),
             (zig, "0", False), (zig, "2", False), ("gcc-14", "2", True)]
    for index, (compiler, optimization, sanitized) in enumerate(cases):
        flags = [*FLAGS, "-O" + optimization]
        if sanitized:
            flags += ["-fsanitize=undefined", "-fno-sanitize-recover=all"]
        for fault in ["valid", "sign_loss", "finite_clamp", "zero"]:
            directory = root / f"{index}-{fault}"
            directory.mkdir()
            for name, text in originals.items():
                (directory / name).write_bytes(text)
            if fault != "valid":
                (directory / "constants.c").write_text(mutate(source, fault))
            actual = observe(directory, compiler, flags)
            assert (actual == expected(VALUES)) == (fault == "valid"), (compiler, fault, actual)
            if fault != "valid":
                assert actual == expected([faulty(value, fault) for value in VALUES])
    assert originals == {path.name: path.read_bytes() for path in root.iterdir() if path.is_file()}
    print("Both exact infinity signs; GCC14/Zig O0/O2 + GCC UBSan; standalone headers; "
          "separate producers and alias readers; no libm; three compiling faults rejected")


if __name__ == "__main__":
    main()
