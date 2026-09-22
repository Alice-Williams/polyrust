"""Actual certified AST packages, separately compiled, with modular truth."""
from pathlib import Path
import subprocess
import sys

ROOT, ZIG, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from wrapping_add_oracle import CASES, result

STEMS = ["polyrust_wrapping_701", "polyrust_wrapping_702"]
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char",
         "-fno-short-enums"]
CLIENT = "".join(f'#include "{stem}.h"\n' for stem in STEMS) + r"""
#include <inttypes.h>
#include <stdio.h>
int main(void) {
    int width;
    int64_t left, right;
    while (scanf("%d %" SCNd64 " %" SCNd64, &width, &left, &right) == 3) {
        if (width == 32) {
            (void)printf("%" PRId32 "\n%" PRId32 "\n",
                poly_wrapping_701_0((int32_t)left, (int32_t)right),
                poly_wrapping_702_0((int32_t)left, (int32_t)right));
        } else if (width == 64) {
            (void)printf("%" PRId64 "\n%" PRId64 "\n",
                poly_wrapping_701_1(left, right), poly_wrapping_702_1(left, right));
        } else { return 2; }
    }
    return ferror(stdin) != 0;
}
"""


def run(command, inputs=None):
    output = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def expected(variant):
    values = []
    for width, left, right in CASES:
        value = result(left, left if variant == "wrong-operand" else right, width)
        if variant == "wrong-result" and value < 0:
            value += 1
        values.extend([str(value)] * 2)
    return "\n".join(values) + "\n"


def main():
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    inputs = "".join(f"{width} {left} {right}\n" for width, left, right in CASES)
    truth = expected("valid")
    for variant in ["valid", "boundary-literals", "wrong-operand", "wrong-result"]:
        directory = ROOT / variant
        assert {p.name for p in directory.iterdir()} == {
            stem + suffix for stem in STEMS for suffix in [".c", ".h"]
        }
        wanted = expected(variant)
        assert (wanted == truth) == (variant in ["valid", "boundary-literals"])
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
                        header_client = directory / (stem + "_header.c")
                        header_client.write_text(f'#include "{stem}.h"\n')
                        run([compiler, *flags, "-c", header_client,
                             "-o", directory / (label + stem + "_header.o")])
                    binary = directory / label
                    run([compiler, *flags, *objects, "-o", binary])
                    output = run([binary], inputs)
                    assert output.stdout == wanted, (variant, label, "modular values")
                    assert output.stderr == "", (variant, label, "sanitizer/diagnostics")
    print(f"{len(CASES)} pairs, both owners; GCC14/Zig O0/O2 + GCC UBSan; "
          "full unsigned literal boundaries; two typed safe-fault packages differ from independent truth")


if __name__ == "__main__":
    main()
