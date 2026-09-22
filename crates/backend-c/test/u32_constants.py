"""Independent scalar/target integer truth from actual certified C constants."""
from pathlib import Path
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(Path(sys.argv[-1]).resolve()))
from character_constant_oracle import BOUNDARIES, FAULTS, expected_values, faulty

TARGET_ONLY = (0xd800, 0xdfff, 0x110000, 0x7fffffff, 0x80000000, 0xffffffff)
VALUES = (*expected_values(), *TARGET_ONLY)
FLAGS = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char", "-fno-short-enums"]


def run(command):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=90)
    assert output.returncode == 0 and output.stderr == "", (command, output.stdout, output.stderr)
    return output.stdout


def client(count):
    text = '#include "polyrust_constant_facade_921.h"\n'
    text += '#include "polyrust_constant_reader_922.h"\n'
    text += '#include <inttypes.h>\n#include <stdio.h>\nint main(void) {\n'
    text += 'const uint32_t *const objects[] = {\n'
    text += ',\n'.join(f'&poly_scalar_{index}' for index in range(count)) + '\n};\n'
    text += f'for (size_t i = 0; i < {count}; ++i) {{ (void)printf("%08" PRIx32 "\\n", *objects[i]); }}\n'
    for index in range(count):
        text += f'(void)printf("%08" PRIx32 "\\n", poly_import_922_{index}());\n'
    return text + 'return 0;\n}\n'


def configurations(zig):
    for compiler, optimization, sanitized in [
            ("gcc-14", "0", False), ("gcc-14", "2", False),
            (zig, "0", False), (zig, "2", False), ("gcc-14", "2", True)]:
        flags = [*FLAGS, "-O" + optimization]
        if sanitized:
            flags += ["-fsanitize=undefined", "-fno-sanitize-recover=all"]
        yield compiler, flags


def observe(directory, values, zig):
    headers = sorted(directory.glob("*.h"))
    sources = sorted(directory.glob("*.c"))
    assert len(headers) == len(sources) == 3
    driver = directory / "client.c"
    driver.write_text(client(len(values)))
    sources.append(driver)
    expected = "".join(f"{value:08x}\n" for value in values) * 2
    for compiler, flags in configurations(zig):
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
        assert run([binary]) == expected, (directory, compiler, flags)


def mutate(directory, fault):
    path = directory / "constants.c"
    original = path.read_text()
    def replace(match):
        value = faulty(BOUNDARIES[int(match[2])], fault)
        return match[1] + f"UINT32_C({value});"
    changed, count = re.subn(r"(\bpoly_scalar_(\d+)\s*=\s*)[^;]+;", replace, original)
    assert count == len(BOUNDARIES) and changed != original
    path.write_text(changed)


def readonly(directory, zig):
    probe = directory / "readonly.c"
    probe.write_text('#include "polyrust_constants.h"\n'
                     'int main(void) { poly_scalar_0 = UINT32_C(1); return 0; }\n')
    for compiler, flags in configurations(zig):
        result = subprocess.run([str(arg) for arg in [compiler, *flags, "-c", probe,
                                "-o", directory / "readonly.o"]], capture_output=True, timeout=90)
        assert result.returncode != 0, (compiler, "constant accepted mutation")


def main():
    assert len(VALUES) == 4133
    if sys.argv[1] == "inputs":
        print("\n".join(f"{value:08x}" for value in VALUES))
        return
    root, zig = [Path(arg).resolve() for arg in sys.argv[1:3]]
    for index, start in enumerate(range(0, len(VALUES), 64)):
        observe(root / str(index), VALUES[start:start + 64], zig)
    controls = root / "controls"
    original = {path.name: path.read_bytes() for path in controls.iterdir() if path.suffix in (".h", ".c")}
    assert len(original) == 6
    for fault in ["valid", *FAULTS]:
        directory = root / fault
        shutil.copytree(controls, directory)
        expected = list(BOUNDARIES) if fault == "valid" else [faulty(value, fault) for value in BOUNDARIES]
        assert (expected == list(BOUNDARIES)) == (fault == "valid")
        if fault != "valid":
            mutate(directory, fault)
        observe(directory, expected, zig)
    readonly(root / "valid", zig)
    assert original == {path.name: path.read_bytes() for path in controls.iterdir() if path.suffix in (".h", ".c")}
    print("4133 certified U32 objects and original-owner imported readers; "
          "GCC/Zig O0/O2 and UBSan, separate owners/headers, readonly consumers; "
          "four compiling narrowing/replacement/value faults detected")


if __name__ == "__main__":
    main()
