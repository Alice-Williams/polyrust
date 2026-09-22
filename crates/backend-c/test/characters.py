"""Actual certified C packages: full scalar transport and independently modeled faults."""
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys

ROOT, ZIG, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from character_oracle import BOUNDARIES, PAIRS, MAXIMUM, SCALAR_COUNT, scalar, flags

STEMS = [f"polyrust_characters_{number}" for number in (941, 942, 943)] + ["crate_api"]
HEADERS = {stem: stem + ".h" for stem in STEMS}
HEADERS["crate_api"] = "polyrust_crate_api.h"
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char",
         "-fno-short-enums"]
INPUT = struct.Struct("<II")
OUTPUT = struct.Struct("<IIIIIIII")
CLIENT = "".join(f'#include "{HEADERS[stem]}"\n' for stem in STEMS) + r"""
#include <stdio.h>
#include <stdbool.h>
static uint32_t read32(const unsigned char *bytes) {
    return (uint32_t)bytes[0] | ((uint32_t)bytes[1] << 8)
        | ((uint32_t)bytes[2] << 16) | ((uint32_t)bytes[3] << 24);
}
static int write32(uint32_t value) {
    unsigned char bytes[4];
    for (unsigned int i = 0; i < 4; ++i) bytes[i] = (unsigned char)(value >> (8 * i));
    return fwrite(bytes, 1, 4, stdout) == 4;
}
int main(void) {
LITERALS
    unsigned char bytes[8];
    size_t length;
    while ((length = fread(bytes, 1, sizeof bytes, stdin)) != 0) {
        if (length != sizeof bytes) return 2;
        uint32_t left = read32(bytes), right = read32(bytes + 4);
        uint32_t results[8] = {
            poly_char_941_identity(left), poly_char_941_local(left),
            poly_char_942_forward(left), poly_char_943_forward(left),
            poly_char_941_select(true, left, right),
            poly_char_941_select(false, left, right),
            COMPARISONS,
            RECORD(left)
        };
        for (unsigned int i = 0; i < 8; ++i) if (!write32(results[i])) return 4;
    }
    return ferror(stdin) != 0 || fflush(stdout) != 0;
}
"""
CLIENT = CLIENT.replace("LITERALS", "\n".join(
    f"    if (poly_char_941_literal_{i}() != UINT32_C({value})) return 3;"
    for i, value in enumerate(BOUNDARIES)))
CLIENT = CLIENT.replace("COMPARISONS", " | ".join(
    f"((uint32_t)poly_char_941_compare_{i}(left, right) << {i})" for i in range(6)))


def run(command, data=None, success=True):
    output = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, timeout=120)
    assert (output.returncode == 0) == success, (command, output.stdout, output.stderr)
    return output


def rows(full):
    if full:
        for value in range(MAXIMUM + 1):
            if scalar(value):
                yield value, 0
    else:
        yield from ((value, 0) for value in BOUNDARIES)
    yield from PAIRS


def expected(left, right, variant):
    transported = left
    if variant == "byte":
        transported &= 0xff
    elif variant == "utf16":
        transported &= 0xffff
    comparison = flags(right, left) if variant == "reverse" else flags(left, right)
    return transported, left, transported, transported, left, right, comparison, left


def mutate(producer, variant):
    text = producer.read_text()
    if variant in ("byte", "utf16"):
        pattern = r"(\buint32_t poly_char_941_identity\([^)]*\)\s*\{\s*return\s+)([^;]+)(;)"
        matches = list(re.finditer(pattern, text))
        assert len(matches) == 1, ("identity definition shape", text)
        ty = "uint8_t" if variant == "byte" else "uint16_t"
        text, count = re.subn(pattern, lambda m: m[1] + f"({ty})({m[2]})" + m[3], text)
        assert count == 1
    else:
        assert variant == "reverse"
        for index in range(6):
            pattern = rf"(\b(?:_Bool|bool) poly_char_941_compare_{index}\([^)]*\)\s*\{{\s*return\s+)([^;]+)(;)"
            def reverse(match):
                result = match[2]
                parameters = re.findall(r"\buint32_t\s+(\w+)", match[1])
                assert len(parameters) == 2 and parameters[0] != parameters[1]
                assert all(re.search(r"\b" + name + r"\b", result) for name in parameters)
                swap = dict(zip(parameters, reversed(parameters)))
                result = re.sub(r"\b(?:" + "|".join(parameters) + r")\b",
                                lambda found: swap[found[0]], result)
                return match[1] + result + match[3]
            text, count = re.subn(pattern, reverse, text)
            assert count == 1, ("comparison definition shape", index, text)
    assert text != producer.read_text()
    producer.write_text(text)


def main():
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == b"14.2.0"
    files = [stem + ".c" for stem in STEMS] + list(HEADERS.values())
    assert {path.name for path in ROOT.iterdir()} == set(files) | {"record_symbol.txt"}
    record_symbol = (ROOT / "record_symbol.txt").read_text()
    assert re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", record_symbol)
    originals = {name: (ROOT / name).read_bytes() for name in files}
    for variant in ["valid", "byte", "utf16", "reverse"]:
        cases = list(rows(variant == "valid"))
        assert len(cases) == (SCALAR_COUNT if variant == "valid" else len(BOUNDARIES)) + len(PAIRS)
        data = b"".join(INPUT.pack(*row) for row in cases)
        truth = b"".join(OUTPUT.pack(*expected(*row, "valid")) for row in cases)
        wanted = truth if variant == "valid" else b"".join(OUTPUT.pack(*expected(*row, variant)) for row in cases)
        assert (wanted == truth) == (variant == "valid")
        directory = ROOT / variant
        directory.mkdir()
        for name in files:
            shutil.copyfile(ROOT / name, directory / name)
        if variant != "valid":
            mutate(directory / (STEMS[0] + ".c"), variant)
        (directory / "consumer.c").write_text(CLIENT.replace("RECORD", record_symbol))
        for compiler in ["gcc-14", ZIG]:
            for optimization in ["0", "2"]:
                for sanitized in [False, True]:
                    if sanitized and (compiler != "gcc-14" or optimization != "2"):
                        continue
                    label = Path(compiler).name + optimization + str(sanitized)
                    opts = FLAGS + ["-O" + optimization]
                    if sanitized:
                        opts += ["-fsanitize=undefined", "-fno-sanitize-recover=all"]
                    objects = []
                    for stem in [*STEMS, "consumer"]:
                        obj = directory / (label + stem + ".o")
                        run([compiler, *opts, "-c", directory / (stem + ".c"), "-o", obj])
                        objects.append(obj)
                    for stem in STEMS:
                        header = directory / (stem + "_header.c")
                        header.write_text(f'#include "{HEADERS[stem]}"\n')
                        run([compiler, *opts, "-c", header, "-o", directory / (label + stem + "_header.o")])
                    binary = directory / label
                    run([compiler, *opts, *objects, "-o", binary])
                    output = run([binary], data)
                    assert output.stdout == wanted, (variant, label, "exact values/order")
                    assert not output.stderr, (variant, label, output.stderr)
                    assert run([binary], b"x", success=False).returncode == 2
    # Removing the symbol-derived dependency makes an ordinary standalone consumer fail.
    negative = ROOT / "negative"
    negative.mkdir()
    header_text = originals[STEMS[0] + ".h"].decode()
    assert "#include <stdint.h>" in header_text
    (negative / "missing.h").write_text(header_text.replace("#include <stdint.h>", ""))
    (negative / "missing.c").write_text('#include "missing.h"\n')
    # The native compiler is also exercised as a syntax-negative oracle.
    source_text = originals[STEMS[0] + ".c"].decode()
    malformed, count = re.subn(r"(\breturn\b[^;]+);", r"\1", source_text, count=1)
    assert count == 1
    (negative / (STEMS[0] + ".h")).write_bytes(originals[STEMS[0] + ".h"])
    (negative / "syntax.c").write_text(malformed)
    for compiler in ["gcc-14", ZIG]:
        for name in ["missing", "syntax"]:
            result = run([compiler, *FLAGS, "-c", negative / (name + ".c"),
                          "-o", negative / (name + ".o")], success=False)
            assert result.stderr
    assert originals == {name: (ROOT / name).read_bytes() for name in files}
    print(f"{SCALAR_COUNT} scalars + {len(PAIRS)} pairs; 19 literals; "
          "eight observations per input, three original owners and private record; GCC14/Zig O0/O2 + UBSan; "
          "three compiling faults detected; standalone headers and negative controls pass")


if __name__ == "__main__":
    main()
