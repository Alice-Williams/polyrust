"""Strict native proof of arithmetic inside the previously admitted operations."""
import json
import os
from pathlib import Path
import subprocess
from java_fixture_native import exports


def run(command):
    result = subprocess.run([str(item) for item in command], capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)


def check(directory, language, zig):
    paths = list(directory.rglob("*.json"))
    manifests = [json.loads(path.read_text()) for path in paths]
    api, = [item for item in manifests if "modules" in item]
    java = language == "java"
    bindings = exports(api, java)
    functions = {item["id"]: item for item in api["declarations" if java else "functions"]
                 if not java or item["kind"] == "function"}

    def call(name, argument):
        item = functions[bindings[api["root"], "value", name][1]]
        if java:
            path = item["target"]["path"]
            target = ".".join([path["package"], *path["owners"], path["member"]])
        else:
            target = item["symbol"]
        return target + "(" + argument + ")"

    conditions = []
    for name, first, second in [("negated", "-2.5", "1.5"), ("absolute", "2.5", "1.5"),
                                ("truncated", "2.0", "-1.0")]:
        conditions += [call(name, "1.5") + " == " + first, call(name, "-2.5") + " == " + second]
    conditions += ["!" + call("nan", "1.5"), call("nan", "Double.NaN" if java else "NAN")]
    condition = " && ".join("(" + item + ")" for item in conditions)
    if java:
        client = directory / "Consumer.java"
        client.write_text("public final class Consumer { private Consumer() {} "
                          "public static void main(String[] args) { if (!(" + condition +
                          ")) { throw new AssertionError(); } } }\n")
        runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                    if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
        runtime, = runtimes
        classes = directory / "classes"
        classes.mkdir()
        flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                 "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
        source, = [p for p in directory.rglob("Generated.java")]
        run([runtime / "javac", *flags, source])
        run([runtime / "javac", *flags, client])
        run([runtime / "java", "-cp", classes, "Consumer"])
    else:
        assert api["system_libraries"] == ["m"]
        header, = list(directory.rglob(api["header"]))
        source, = list(directory.rglob(api["implementation"]))
        client = directory / "consumer.c"
        client.write_text(f'#include "{header.name}"\n#include <math.h>\n'
                          "int main(void) { return (" + condition + ") ? 0 : 1; }\n")
        flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                 "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off", "-fsigned-char",
                 "-fno-short-enums", "-fno-builtin", "-I", header.parent]
        for compiler in ["gcc-14", zig]:
            for optimization in ["0", "2"]:
                executable = directory / (Path(compiler).name + optimization)
                run([compiler, *flags, "-O" + optimization, source, client, "-lm", "-o", executable])
                run([executable])
