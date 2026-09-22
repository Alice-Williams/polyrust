"""Both call forms, nested/grouped addition and an ordinary same-spelled function."""
import os
from pathlib import Path
import subprocess
from java_fixture_native import manifest, exports, check_inventory_oracle
from addition_source_consumers import consumer
from wrapping_add_oracle import CASES, inputs, result, signed


def run(command, data=None):
    output = subprocess.run(list(map(str, command)), input=data, capture_output=True, text=True, timeout=120)
    assert output.returncode == 0, (command, output.stdout[:1000], output.stderr[:4000])
    return output


def check(directory, language, zig):
    java = language == "java"
    api = manifest(directory, 3 if java else 4)
    bindings = exports(api, java)
    names = {width: [prefix + str(width) for prefix in ["method", "associated", "nested", "grouped"]]
             for width in [32, 64]}
    names[32].append("ordinary_name")
    check_inventory_oracle(bindings, {(api["root"], "value", name) for group in names.values() for name in group})
    declarations = {d["id"]: d for d in api["declarations" if java else "functions"]}
    def member(name):
        declaration = declarations[bindings[api["root"], "value", name][1]]
        if not java:
            return declaration["symbol"]
        path = declaration["target"]["path"]
        return ".".join([path["package"], *path["owners"], path["member"]])
    calls = {width: [member(name) for name in group] for width, group in names.items()}
    wanted = []
    for width, left, right in CASES:
        value = result(left, right, width)
        wanted.extend([value, value, result(value, 1, width), signed(~value, width)])
        if width == 32:
            wanted.append(left)
    truth = "".join(f"{value}\n" for value in wanted)
    driver = directory / ("Consumer.java" if java else "consumer.c")
    driver.write_text(consumer(calls, java, "" if java else f'#include "{api["header"]}"\n'))
    if java:
        runtime, = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                    if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
        classes = directory / "classes"
        classes.mkdir()
        flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
        for source in [directory / api["source"], driver]:
            run([runtime / "javac", *flags, source])
        results = [run([runtime / "java", "-cp", classes, "Consumer"], inputs())]
    else:
        flags = ["-std=c17", "-O2", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                 "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char", "-fno-short-enums", "-I", directory]
        results = []
        for index, compiler in enumerate(["gcc-14", zig]):
            objects = []
            for part, source in enumerate([directory / api["implementation"], driver]):
                obj = directory / f"{index}-{part}.o"
                run([compiler, *flags, "-c", source, "-o", obj])
                objects.append(obj)
            executable = directory / f"composition{index}"
            run([compiler, *objects, "-o", executable])
            results.append(run([executable], inputs()))
    for output in results:
        assert output.stdout == truth and output.stderr == "", (language, "composition values")
