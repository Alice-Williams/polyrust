"""Source-owned C/Java against complete Unicode scalar and original Rust truth."""
import os
from pathlib import Path
import struct
import subprocess
import sys

from character_source_clients import c_client, java_client
from character_source_examples import export
from character_source_inventory import inspect
from character_source_privacy import verify as verify_privacy
from character_source_truth import corpus
from constant_export_scratch import writable_copy
from finite_constant_source_consumers import jdk, run


def observe(command, data, expected):
    result = subprocess.run([str(arg) for arg in command], input=data, capture_output=True, timeout=240)
    assert result.returncode == 0 and result.stdout == expected and not result.stderr, (
        command, result.returncode, result.stderr, len(result.stdout), len(expected))
    # These are consumer protocol checks, not a claim that target integers
    # enforce Unicode validity on foreign callers.
    for packet in [bytes(n) for n in [1, 4, 8, 11, 13]] + [
        struct.pack("<IIi", bad, 0, 0) for bad in [0xd800, 0xdfff, 0x110000, 0xffffffff]
    ]:
        rejected = subprocess.run([str(arg) for arg in command], input=packet,
                                  capture_output=True, timeout=30)
        assert rejected.returncode != 0, command


def compile_c(directory, apis, order, zig):
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", directory]
    for compiler, options in [("gcc-14", ["-O0"]), ("gcc-14", ["-O2"]),
                              (zig, ["-O0"]), (zig, ["-O2"]),
                              ("gcc-14", ["-O2", "-fsanitize=undefined", "-fno-sanitize-recover=all"])]:
        for owner in order:
            header = directory / "header-check.c"
            header.write_text('#include "' + apis[owner]["header"] + '"\n')
            run([compiler, *flags, *options, "-c", header, "-o", directory / "header-check.o"])
        objects = []
        for i, source in enumerate([directory / apis[owner]["implementation"] for owner in order]
                                   + [directory / "consumer.c"]):
            obj = directory / f"unit{i}.o"
            run([compiler, *flags, *options, "-c", source, "-o", obj])
            objects.append(obj)
        binary = directory / "consumer"
        run([compiler, *options, *objects, "-o", binary])
        yield [binary]


def compile_java(directory, apis, order, tools):
    classes = directory / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    for source in [directory / apis[owner]["source"] for owner in order] + [directory / "Consumer.java"]:
        run([tools / "javac", *flags, source])
    return [[tools / "java", *options, "-cp", classes, "Consumer"] for options in [[], ["-Xint"]]]


def main():
    c_dir, j_dir, checked, optimized, zig = [Path(p).resolve() for p in sys.argv[1:]]
    data, expected, count = corpus()
    for reference in [checked, optimized]:
        observe([reference], data, expected)
    evidence = inspect(c_dir, j_dir)
    _, owners, c, java, _, _, _ = evidence
    order = [owners[name] for name in ["leaf", "middle", "root"]]
    original = {p: p.read_bytes() for d in [c_dir, j_dir] for p in d.rglob("*") if p.is_file()}
    work = Path(os.environ["TEST_TMPDIR"]) / "character-source-native"
    work.mkdir()
    cd, jd = work / "c", work / "java"
    writable_copy(c_dir, cd)
    writable_copy(j_dir, jd)
    (cd / "consumer.c").write_text(c_client(evidence))
    (jd / "Consumer.java").write_text(java_client(evidence))
    for command in compile_c(cd, c, order, zig):
        observe(command, data, expected)
    tools = jdk()
    for command in compile_java(jd, java, order, tools):
        observe(command, data, expected)
    verify_privacy(cd, jd, evidence, tools)
    assert original == {p: p.read_bytes() for p in original}
    export(c_dir, j_dir, cd, jd)
    print(f"{count} original/C/Java rows, 19 literal observations, exact char/i32 metadata; "
          "GCC/Zig O0/O2 and UBSan, Java21 normal/-Xint, two Rust profiles; "
          "separately compiled original owners, aliases, documentation and privacy inventory")


if __name__ == "__main__":
    main()
