"""Handwritten ordinary C and Java clients; compile each owner separately."""
from pathlib import Path
import os
import subprocess
from unit_trace import VALUES

def run(command):
    result = subprocess.run([str(item) for item in command], capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result

def java_runtime():
    found = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
             if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(found) == 1
    return found[0]

def java_client(work, owners):
    api, _, calls = owners["unit_root"]
    execute, tail, branch = [calls[name][0] for name in ["execute", "tail", "branch"]]
    owner, member = tail.rsplit(".", 1)
    text = 'public final class Consumer { public static void main(String[] args) throws ReflectiveOperationException {\n'
    text += f'if ({owner}.class.getDeclaredMethod("{member}", int.class).getReturnType() != void.class) throw new AssertionError();\n'
    text += 'for (int value : new int[] { Integer.MIN_VALUE, -1, 0, 1, Integer.MAX_VALUE }) {\n'
    text += 'for (boolean flag : new boolean[] { false, true }) {\n'
    text += f'System.out.println({execute}(value, flag)); {tail}(value); {branch}(flag); System.err.println();\n'
    text += '}}}}\n'
    path = work / "Consumer.java"
    path.write_text(text)
    return path

def c_client(work, owners):
    _, api, calls = owners["unit_root"]
    execute, tail, branch = [calls[name][1] for name in ["execute", "tail", "branch"]]
    text = '#include <inttypes.h>\n#include <stdio.h>\n'
    text += f'#include "{api["header"]}"\n'
    text += 'int main(void) {\nconst int32_t values[] = { INT32_MIN, -1, 0, 1, INT32_MAX };\n'
    text += f'void (*tail_call)(int32_t) = {tail};\n'
    text += 'for (unsigned int i=0; i<sizeof(values)/sizeof(values[0]); ++i) {\nfor (int flag=0;flag<2;++flag) {\n'
    text += f'printf("%" PRId32 "\\n", {execute}(values[i], flag != 0)); tail_call(values[i]); {branch}(flag != 0); (void)fputc(\'\\n\', stderr);\n'
    text += '}}return 0;}\n'
    path = work / "consumer.c"
    path.write_text(text)
    return path

def compile_java(bundle, work, owners):
    runtime = java_runtime()
    classes = work / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    for label in ["unit_leaf", "unit_relay", "unit_root"]:
        run([runtime / "javac", *flags, bundle / owners[label][0]["source"]])
    run([runtime / "javac", *flags, java_client(work, owners)])
    return [run([runtime / "java", "-cp", classes, "Consumer"])]

def compile_c(bundle, work, owners, zig):
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", bundle]
    consumer = c_client(work, owners)
    results = []
    for index, compiler in enumerate(["gcc-14", zig]):
        for optimization in ["-O0", "-O2"]:
            prefix = f"{index}{optimization}"
            objects = []
            for label in ["unit_leaf", "unit_relay", "unit_root"]:
                api = owners[label][1]
                header = work / f"{label}_header.c"
                header.write_text(f'#include "{api["header"]}"\n')
                run([compiler, *flags, optimization, "-c", header, "-o", work / f"{prefix}_{label}_header.o"])
                obj = work / f"{prefix}_{label}.o"
                run([compiler, *flags, optimization, "-c", bundle / api["implementation"], "-o", obj])
                objects.append(obj)
            obj = work / f"{prefix}_consumer.o"
            run([compiler, *flags, optimization, "-c", consumer, "-o", obj])
            binary = work / f"{prefix}_consumer"
            run([compiler, *objects, obj, "-o", binary])
            results.append(run([binary]))
    return results
