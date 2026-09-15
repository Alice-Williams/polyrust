"""Separate native library/consumer compilation with exact wide integer results."""
import os
from pathlib import Path
import re
import subprocess
import sys

from i64_inventory import inspect
from i64_oracle import VALUES, WIDE, BOOL, expected, expected_traces
from i64_order_mutations import markers, reorder
from short_circuit_mutations import definition, instrument


def run(command):
    result = subprocess.run([str(arg) for arg in command], capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result


def narrow(text, member, java):
    _, start, end = definition(text, member)
    cast = "(long)(int)" if java else "(int64_t)(int32_t)"
    body, count = re.subn(r"\breturn\s+([^;]+);", lambda match: f"return {cast}({match[1]});", text[start:end])
    assert count == 1
    return text[:start] + body + text[end:]


def c_literal(value):
    if value == -(1 << 63):
        return "(-9223372036854775807LL - 1LL)"
    return str(value) + "LL"


def main():
    java_dir, c_dir, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    root, leaf, java, c, bindings, functions, c_functions = inspect(java_dir, c_dir)
    truth = expected()
    traces = expected_traces()
    assert run([reference]).stdout == truth
    marker_ids = markers(functions)
    work_root = Path(os.environ["TEST_TMPDIR"]) / "i64-native"
    work_root.mkdir()
    runtimes = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(identity):
            if not is_java:
                return c_functions[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        calls = []
        for name in WIDE + BOOL + ["exported_identity"]:
            identity = bindings[root, "value", name][1]
            arguments = "a,b,flag"
            call = f"{member(identity)}({arguments})"
            if name in BOOL:
                call = f"({call} ? 1 : 0)"
            calls.append((call, name in BOOL))
        marker_names = {marker: member(identity).split(".")[-1] for marker, identity in marker_ids.items()}
        for variant in ["plain", "traced", "narrow", "reordered"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in [leaf, root]:
                source_text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == root and variant != "plain":
                    source_text = instrument(source_text, marker_names, is_java)
                    if variant == "narrow":
                        identity = bindings[root, "value", "identity"][1]
                        source_text = narrow(source_text, member(identity).split(".")[-1], is_java)
                    elif variant == "reordered":
                        for name, operator in zip(BOOL[:-1], ["==", "!=", "<", "<=", ">", ">="], strict=True):
                            identity = bindings[root, "value", name][1]
                            source_text = reorder(source_text, member(identity).split(".")[-1],
                                                  marker_names, is_java, operator)
                source_dir = work / owner.split(":")[0]
                source_dir.mkdir()
                source = source_dir / ("Generated.java" if is_java else "generated.c")
                source.write_text(source_text, encoding="utf-8")
                sources.append(source)
            if is_java:
                consumer = work / "Consumer.java"
                consumer.write_text("public final class Consumer { public static void main(String[] args) {\n"
                                    "long[] values = {" + ",".join(str(value) + "L" for value in VALUES) + "};\n"
                                    "for(long a: values) { for(long b: values) { for(int f=0;f<2;f++) {\n"
                                    "boolean flag=f!=0;\n" + "\n".join(f"System.out.println({call}); System.err.println();" for call, _ in calls)
                                    + "\n} } } } }\n")
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror", "-implicit:none",
                         "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, consumer]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", "-cp", classes, "Consumer"])]
            else:
                consumer = work / "consumer.c"
                lines = [f'printf("%d\\n", {call});' if boolean else f'printf("%" PRId64 "\\n", {call});'
                         for call, boolean in calls]
                lines = [line + " (void)fputc('\\n', stderr);" for line in lines]
                headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in [leaf, root])
                consumer.write_text('#include <stdio.h>\n#include <inttypes.h>\n#include <stdbool.h>\n' + headers
                                    + 'int main(void) {\nconst int64_t values[] = {'
                                    + ",".join(c_literal(value) for value in VALUES) + '};\n'
                                    + f'for(int ia=0;ia<{len(VALUES)};ia++) {{\nfor(int ib=0;ib<{len(VALUES)};ib++) {{\n'
                                    + 'for(int f=0;f<2;f++) {\nint64_t a=values[ia],b=values[ib]; bool flag=f!=0;\n'
                                    + "\n".join(lines) + '\n}\n}\n}\nreturn 0;\n}\n')
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off", "-fsigned-char",
                         "-fno-short-enums", "-I", c_dir]
                results = []
                for compiler in ["gcc-14", zig]:
                    for optimization in ["0", "2"]:
                        label = Path(compiler).name + optimization
                        objects = []
                        for index, source in enumerate([*sources, consumer]):
                            obj = work / (label + str(index) + ".o")
                            run([compiler, *flags, "-O" + optimization, "-c", source, "-o", obj])
                            objects.append(obj)
                        executable = work / label
                        run([compiler, *objects, "-o", executable])
                        results.append(run([executable]))
            for result in results:
                assert len(result.stdout.splitlines()) == 17640
                assert (result.stdout != truth) if variant == "narrow" else (result.stdout == truth), (is_java, variant, "wide integer oracle")
                if variant == "plain":
                    assert result.stderr == "\n" * 17640
                elif variant == "reordered":
                    assert result.stderr != traces, (is_java, "reordering escaped trace oracle")
                    actual = result.stderr.splitlines()
                    expected_lines = traces.splitlines()
                    for offset in range(17640):
                        expected_trace = expected_lines[offset] if offset % 20 not in range(12, 18) else "RLC"
                        assert actual[offset] == expected_trace
                else:
                    assert result.stderr == traces, (is_java, variant, "evaluation-order oracle")
    print("17640 exact Rust/Java/GCC-O0/O2/Zig-O0/O2 results and call traces across two crates; narrowing and six reordered comparison faults detected")


if __name__ == "__main__":
    main()
