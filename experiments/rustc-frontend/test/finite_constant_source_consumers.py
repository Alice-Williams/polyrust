"""External raw-bit observations of separately compiled source-owned packages."""
from pathlib import Path
import os
import subprocess


def run(command, accepted=True):
    output = subprocess.run([str(arg) for arg in command], capture_output=True,
                            text=True, timeout=120)
    assert (output.returncode == 0) == accepted, (command, output.stdout, output.stderr)
    if accepted:
        assert output.stderr == "", (command, output.stderr)
    return output.stdout


def jdk():
    candidates = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                  if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(candidates) == 1
    assert run([candidates[0] / "javac", "-version"]).startswith("javac 21.")
    return candidates[0]


def java_observe(directory, order, apis, expressions, expected, tools):
    classes = directory / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    # Fresh classes; producer BEFORE consumers after every deliberate mutation.
    for owner in order:
        run([tools / "javac", *flags, directory / apis[owner]["source"]])
    driver = directory / "Consumer.java"
    body = "\n".join('System.out.printf("%016x%n", Double.doubleToRawLongBits(' + e + '));'
                     for e in expressions)
    driver.write_text("public final class Consumer { public static void main(String[] args) {\n"
                      + body + "\n} }\n")
    run([tools / "javac", *flags, driver])
    for options in [[], ["-Xint"]]:
        assert run([tools / "java", *options, "-cp", classes, "Consumer"]) == expected


def c_observe(directory, order, apis, expressions, expected, zig):
    driver = directory / "consumer.c"
    driver.write_text('#include <stdint.h>\n#include <inttypes.h>\n#include <stdio.h>\n#include <string.h>\n'
                      + '#include "' + apis[order[-1]]["header"] + '"\n'
                      + 'int main(void) {\n'
                      + "\n".join('{ double value = ' + e + '; uint64_t bits; '
                                  'memcpy(&bits, &value, sizeof(bits)); '
                                  'printf("%016" PRIx64 "\\n", bits); }' for e in expressions)
                      + '\nreturn 0; }\n')
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", directory]
    configurations = [("gcc-14", ["-O0"]), ("gcc-14", ["-O2"]),
                      (zig, ["-O0"]), (zig, ["-O2"]),
                      ("gcc-14", ["-O2", "-fsanitize=undefined", "-fno-sanitize-recover=all"])]
    for compiler, options in configurations:
        for owner in order:
            header_probe = directory / "header-check.c"
            header_probe.write_text('#include "' + apis[owner]["header"] + '"\n')
            run([compiler, *flags, *options, "-c", header_probe, "-o", directory / "header-check.o"])
        objects = []
        for index, unit in enumerate([directory / apis[owner]["implementation"] for owner in order] + [driver]):
            obj = directory / f"unit{index}.o"
            run([compiler, *flags, *options, "-c", unit, "-o", obj])
            objects.append(obj)
        binary = directory / "consumer"
        libraries = sorted({library for api in apis.values() for library in api.get("system_libraries", [])})
        assert set(libraries) <= {"m"}
        run([compiler, *options, *objects, *["-l" + library for library in libraries], "-o", binary])
        assert run([binary]) == expected


def replace_constant(directory, language, api, row, bits):
    sign = "-" if bits >> 63 else ""
    exponent, fraction = (bits >> 52) & 2047, bits & ((1 << 52) - 1)
    assert exponent != 2047
    literal = f"{sign}0x{int(exponent != 0)}.{fraction:013x}p{(exponent or 1) - 1023:+d}"
    symbol = row["symbol"] if language == "c" else row["target"]["path"]["member"]
    source = directory / api["implementation" if language == "c" else "source"]
    import re
    before = source.read_text()
    after, count = re.subn(r"(\b" + re.escape(symbol) + r"\s*=\s*)[^;]+;",
                           lambda m: m[1] + literal + ";", before)
    assert count == 1 and before != after
    source.write_text(after)


def privacy(directory, language, header, public, private, tools):
    if language == "java":
        source = directory / "PrivacyProbe.java"
        for expression, accepted in [(public, True), (private, False)]:
            source.write_text("public final class PrivacyProbe { public static void main(String[] a) { "
                              + expression + "(); } }\n")
            run([tools / "javac", "--release", "21", "-Xlint:all", "-Werror",
                 "-implicit:none", "-sourcepath", "", "-cp", directory / "classes",
                 "-d", directory / "classes", source], accepted=accepted)
    else:
        source = directory / "privacy.c"
        for expression, accepted in [(public, True), (private, False)]:
            source.write_text('#include "' + header + '"\nint main(void) { (void)'
                              + expression + '(); return 0; }\n')
            run(["gcc-14", "-std=c17", "-Wall", "-Wextra", "-Werror", "-Wpedantic",
                 "-I", directory, "-fsyntax-only", source], accepted=accepted)
