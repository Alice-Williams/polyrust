"""Separate native compilation, independent truth and actual producer faults."""
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

from character_constant_source_inventory import inspect
from character_constant_source_truth import CONSTANTS, READS, text
from constant_export_native import java_path, run
from constant_export_scratch import writable_copy


def jdk():
    matches = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
               if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(matches) == 1
    return matches[0]


def descriptors(evidence, mixed):
    root, owners, _, _, bindings, constants, jc, values, cf, jf = evidence
    entries = []
    if mixed:
        for name, value in READS.items():
            identity = bindings[root, "value", "read_" + name]
            defining = (bindings[owners["constants"], "value", name.upper()]
                        if name.upper() in CONSTANTS else None)
            if name == "alias":
                defining = bindings[owners["constants"], "value", "MAXIMUM"]
            expression = java_path(jf[identity]["target"]["path"]) + "()"
            if name == "comparison":
                expression = "(" + expression + " ? 1 : 0)"
            entries.append((cf[identity]["symbol"] + "()", expression, value, defining))
    for identity, value in values.items():
        entries.append((constants[identity][1]["symbol"],
                        java_path(jc[identity][1]["target"]["path"]), value, identity))
    return entries


def c_observe(directory, order, apis, expressions, expected, zig):
    consumer = directory / "consumer.c"
    consumer.write_text('#include <stdio.h>\n' + "".join(
        '#include "' + apis[owner]["header"] + '"\n' for owner in order) +
        'int main(void) {\n' + "".join(
        'printf("%lld\\n", (long long)(' + expression + '));\n' for expression in expressions) +
        'return 0;\n}\n')
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", directory]
    for compiler, options in [("gcc-14", ["-O0"]), ("gcc-14", ["-O2"]),
                              (zig, ["-O0"]), (zig, ["-O2"]),
                              ("gcc-14", ["-O2", "-fsanitize=undefined", "-fno-sanitize-recover=all"])]:
        for owner in order:
            run([compiler, *flags, *options, "-x", "c", "-c",
                 directory / apis[owner]["header"], "-o", directory / "header.o"])
        objects = []
        sources = [directory / apis[owner]["implementation"] for owner in order] + [consumer]
        for index, source in enumerate(sources):
            obj = directory / f"unit{index}.o"
            run([compiler, *flags, *options, "-c", source, "-o", obj])
            objects.append(obj)
        binary = directory / "consumer"
        run([compiler, *options, *objects, "-o", binary])
        assert run([binary]) == expected


def java_observe(directory, order, apis, expressions, expected, tools):
    classes = directory / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    consumer = directory / "Consumer.java"
    consumer.write_text("public final class Consumer { public static void main(String[] args) {\n" +
                        "".join("System.out.println(" + expression + ");\n" for expression in expressions) +
                        "} }\n")
    # Fresh output directory and every dependent recompiled after each mutation.
    for source in [directory / apis[owner]["source"] for owner in order] + [consumer]:
        run([tools / "javac", *flags, source])
    for options in [[], ["-Xint"]]:
        assert run([tools / "java", *options, "-cp", classes, "Consumer"]) == expected


def replace_constant(directory, language, api, row, value):
    source = directory / api["implementation" if language == "c" else "source"]
    symbol = row["symbol"] if language == "c" else row["target"]["path"]["member"]
    before = source.read_text()
    after, count = re.subn(r"(\b" + re.escape(symbol) + r"\s*=\s*)[^;]+;",
                           lambda match: match[1] + str(value) + ";", before)
    assert count == 1 and after != before
    source.write_text(after)


def privacy(directory, language, evidence, tools):
    root, owners, c, java, bindings, constants, jc, _, cf, jf = evidence
    hidden, = [identity for identity in jf if not jf[identity]["externally_reachable"]]
    constant = bindings[owners["constants"], "value", "ASCII"]
    for kind in ["private", "readonly"]:
        if language == "c":
            code = (cf[hidden]["symbol"] + "();" if kind == "private"
                    else constants[constant][1]["symbol"] + " = 66;")
            client = directory / "negative.c"
            client.write_text('#include "' + c[root]["header"] + '"\nint main(void) {\n' + code + '\nreturn 0; }\n')
            command = ["gcc-14", "-std=c17", "-Wall", "-Wextra", "-Werror", "-I", directory,
                       "-c", client, "-o", directory / "negative.o"]
            marker = "implicit declaration" if kind == "private" else "read-only variable"
        else:
            code = (java_path(jf[hidden]["target"]["path"]) + "();" if kind == "private"
                    else java_path(jc[constant][1]["target"]["path"]) + " = 66;")
            client = directory / "Negative.java"
            client.write_text("public final class Negative { public static void run() { " + code + " } }")
            command = [tools / "javac", "--release", "21", "-Xlint:all", "-Werror", "-implicit:none",
                       "-sourcepath", "", "-cp", directory / "classes", "-d", directory / "classes", client]
            marker = "private access" if kind == "private" else "cannot assign a value to static final variable"
        result = subprocess.run([str(p) for p in command], capture_output=True, text=True, timeout=120)
        assert result.returncode != 0 and marker in result.stderr, result.stderr


def main():
    cm, jm, cr, jr, checked, optimized, zig = [Path(p).resolve() for p in sys.argv[1:]]
    for reference in [checked, optimized]:
        assert run([reference]) == text(READS.values())
    original = {p: p.read_bytes() for d in [cm, jm, cr, jr] for p in d.rglob("*") if p.is_file()}
    work = Path(os.environ["TEST_TMPDIR"]) / "character-constant-source"
    work.mkdir()
    tools = jdk()
    count = 0
    for mixed, c_dir, j_dir in [(False, cm, jm), (True, cr, jr)]:
        evidence = inspect(c_dir, j_dir, mixed)
        _, owners, c, java, bindings, constants, jc, _, _, _ = evidence
        entries = descriptors(evidence, mixed)
        count += len(entries)
        order = [owners[name] for name in ["constants", "second", "middle"] + (["root"] if mixed else [])]
        identity = bindings[owners["constants"], "value", "MAXIMUM"]
        for variant, changed in [("valid", 0x10ffff), ("byte", 255), ("code_unit", 65535),
                                  ("replacement", 0xfffd), ("changed", 0x10fffe)]:
            expected = text([changed if defining == identity else value for _, _, value, defining in entries])
            assert (expected == text(row[2] for row in entries)) == (variant == "valid")
            for language, source, apis, rows, index in [
                ("c", c_dir, c, constants, 0), ("java", j_dir, java, jc, 1),
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
                    privacy(directory, language, evidence, tools)
                if variant == "valid" and os.environ.get("TEST_UNDECLARED_OUTPUTS_DIR"):
                    output = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / f"{language}-character-constants-{mixed}"
                    writable_copy(source, output)
                    client = "consumer.c" if language == "c" else "Consumer.java"
                    shutil.copy2(directory / client, output / client)
    assert original == {p: p.read_bytes() for p in original}
    print(f"{len(READS)} original Rust reads/profile; {count} target observations/configuration; "
          "exact source kinds/values and aliases; GCC/Zig O0/O2 + UBSan; "
          "Java21 normal/-Xint; four compiling faults with recompiled dependents")


if __name__ == "__main__":
    main()
