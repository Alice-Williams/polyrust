"""Compile actual two-crate bundles; compare values and execution traces."""
import os
from pathlib import Path
import subprocess
import sys

from bitwise_inventory import inspect
from bitwise_oracle import NAMES, cases, expected, expected_traces
from bitwise_mutations import mutate
from short_circuit_mutations import instrument


def run(command, inputs=None):
    result = subprocess.run([str(x) for x in command], input=inputs, capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout[:1000], result.stderr[:4000])
    return result


def consumer(calls, java, headers):
    branches = []
    for width in [32, 64]:
        expressions = [f"{calls[name + str(width)]}(a,b)" for name in NAMES]
        if java:
            outputs = "\n".join(f'System.out.print({call}); System.out.print("{chr(32) if i < 7 else chr(92) + "n"}");'
                                for i, call in enumerate(expressions))
            bindings = "int a=(int)first,b=(int)second;" if width == 32 else "long a=first,b=second;"
            branches.append(f'if(width=={width}) {{ {bindings}\n{outputs}\n}}')
        else:
            outputs = "\n".join(f'printf("%" PRId{width} "{chr(32) if i < 7 else chr(92) + "n"}", {call});'
                                for i, call in enumerate(expressions))
            branches.append(f'if(width=={width}) {{ int{width}_t a=(int{width}_t)first,b=(int{width}_t)second;\n{outputs}\n}}')
    body = " else ".join(branches)
    if java:
        return ('public final class Consumer { public static void main(String[] args) throws java.io.IOException {\n'
                'var reader=new java.io.BufferedReader(new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));\n'
                'String line; while((line=reader.readLine())!=null) { String[] parts=line.split(" ");\n'
                'int width=Integer.parseInt(parts[0]); long first=Long.parseLong(parts[1]), second=Long.parseLong(parts[2]);\n'
                + body + '\nSystem.err.println();\n} } }\n')
    return ('#include <inttypes.h>\n#include <stdio.h>\n' + headers
            + 'int main(void) { int width; int64_t first,second;\n'
            + 'while(scanf("%d %" SCNd64 " %" SCNd64, &width,&first,&second)==3) {\n'
            + body + "\n(void)fputc('\\n',stderr);\n}\nreturn 0;\n}\n")


def main():
    java_dir, c_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    root, leaf, java, c, bindings, functions, native, marker_ids = inspect(java_dir, c_dir)
    rows = cases()
    inputs = "".join(f"{w} {a} {b}\n" for w, a, b in rows)
    truth, traces = expected(rows), expected_traces(rows)
    assert run([reference], inputs).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "bitwise-native"
    work_root.mkdir()
    runtimes = [p / "bin" for p in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in p.name and (p / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, directory, owners in [(True, java_dir, java), (False, c_dir, c)]:
        def member(identity):
            if not is_java:
                return native[identity]["symbol"]
            path = functions[identity]["target"]["path"]
            return ".".join([path["package"], *path["owners"], path["member"]])

        names = {name + str(w): member(bindings[root, "value", name + str(w)][1])
                 for w in [32, 64] for name in NAMES}
        markers = {marker: member(identity).split(".")[-1] for marker, identity in marker_ids.items()}
        for variant in ["plain", "traced", "drop", "reordered"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in [leaf, root]:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if owner == root and variant != "plain":
                    trace_markers = {"A": markers["L32"], "B": markers["R32"],
                                     "C": markers["L64"], "D": markers["R64"]}
                    text = instrument(text, trace_markers, is_java)
                    for width in [32, 64]:
                        if variant == "drop":
                            text = mutate(text, names["invert" + str(width)].split(".")[-1],
                                          markers, width, is_java, "drop")
                        elif variant == "reordered":
                            for name, operator in zip(["and", "or", "xor"], ["&", "|", "^"], strict=True):
                                text = mutate(text, names[name + str(width)].split(".")[-1],
                                              markers, width, is_java, "reordered", operator)
                source_dir = work / owner.split(":")[0]
                source_dir.mkdir()
                source = source_dir / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            driver = work / ("Consumer.java" if is_java else "consumer.c")
            headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in [leaf, root])
            driver.write_text(consumer(names, is_java, headers), encoding="utf-8")
            if is_java:
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, driver]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", "-cp", classes, "Consumer"], inputs)]
            else:
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                         "-Wstrict-prototypes", "-Wmissing-prototypes", "-fno-fast-math",
                         "-ffp-contract=off", "-fsigned-char", "-fno-short-enums", "-I", c_dir]
                results = []
                for compiler in ["gcc-14", zig]:
                    for optimization in ["0", "2"]:
                        label = Path(compiler).name + optimization
                        objects = []
                        for i, source in enumerate([*sources, driver]):
                            obj = work / (label + str(i) + ".o")
                            run([compiler, *flags, "-O" + optimization, "-c", source, "-o", obj])
                            objects.append(obj)
                        executable = work / label
                        run([compiler, *objects, "-o", executable])
                        results.append(run([executable], inputs))
            for result in results:
                assert len(result.stdout.splitlines()) == len(rows)
                assert (result.stdout != truth) if variant == "drop" else (result.stdout == truth), (is_java, variant)
                expected_trace = ("\n" * len(rows) if variant == "plain" else
                                  expected_traces(rows, reordered=variant == "reordered"))
                assert result.stderr == expected_trace, (is_java, variant, "call order/count")
                if variant == "reordered":
                    assert result.stderr != traces
    print(f"{len(rows) * 8} exact Rust/C/Java results; native per-operand traces; missing-complement and six reordered-operator faults detected")


if __name__ == "__main__":
    main()
