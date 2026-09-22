"""Independent native truth for records, literals, borrows and scalar composition."""
import os
from pathlib import Path
import subprocess
import sys
from java_fixture_native import manifest, exports, check_inventory_oracle
from widening_source_consumers import consumer
from widening_oracle import CASES, inputs
from constant_export_scratch import writable_copy


def run(command, data=None):
    output = subprocess.run(list(map(str, command)), input=data, capture_output=True, text=True, timeout=120)
    assert output.returncode == 0, (command, output.stdout[:1000], output.stderr[:4000])
    return output


def main():
    java_bundle, c_bundle, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    names = ["direct", "literal", "nested", "borrowed", "record", "composed", "grouped"]
    def signed32(v):
        return (v + (1 << 31)) % (1 << 32) - (1 << 31)
    truth = "".join(f"{result}\n" for v in CASES
                    for result in [v, -(1 << 31), v, v, v, signed32(v * 3) + 1, ~v - 7])
    data = inputs()
    original = run([reference], data)
    assert original.stdout == truth and original.stderr == ""
    for java, bundle in [(True, java_bundle), (False, c_bundle)]:
        directory = Path(os.environ["TEST_TMPDIR"]) / ("java-composition" if java else "c-composition")
        writable_copy(bundle, directory)
        api = manifest(directory, 3 if java else 4)
        bindings = exports(api, java)
        check_inventory_oracle(bindings, {(api["root"], "value", name) for name in names})
        declarations = {d["id"]: d for d in api["declarations" if java else "functions"]}
        calls = []
        for name in names:
            declaration = declarations[bindings[api["root"], "value", name][1]]
            if java:
                path = declaration["target"]["path"]
                calls.append(".".join([path["package"], *path["owners"], path["member"]]))
            else:
                calls.append(declaration["symbol"])
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
            results = [run([runtime / "java", *options, "-cp", classes, "Consumer"], data)
                       for options in [[], ["-Xint"]]]
        else:
            flags = ["-std=c17", "-O2", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                     "-Wstrict-prototypes", "-Wmissing-prototypes", "-fsigned-char", "-fno-short-enums", "-I", directory]
            results = []
            for index, (compiler, extra) in enumerate([("gcc-14", []), (zig, []),
                    ("gcc-14", ["-fsanitize=undefined", "-fno-sanitize-recover=all"])]):
                objects = []
                for part, source in enumerate([directory / api["implementation"], driver]):
                    obj = directory / f"{index}-{part}.o"
                    run([compiler, *flags, *extra, "-c", source, "-o", obj])
                    objects.append(obj)
                executable = directory / f"composition{index}"
                run([compiler, *extra, *objects, "-o", executable])
                results.append(run([executable], data))
        for output in results:
            assert output.stdout == truth and output.stderr == "", (java, "composition")
    print(f"{len(CASES)} native Rust inputs, seven compositions, separately compiled C and Java")


if __name__ == "__main__":
    main()
