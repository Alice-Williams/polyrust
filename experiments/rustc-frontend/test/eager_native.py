"""Native Boolean values and exact traces across separately compiled packages."""
import os
from pathlib import Path
import subprocess
import sys

from eager_inventory import inspect
from eager_mutations import mutate
from eager_oracle import NAMES, expected
from short_circuit_mutations import instrument


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout[:1000], result.stderr[:4000])
    return result


def consumer(calls, java, headers):
    if java:
        calls = "\n".join(f"System.out.println({call}(a,b,c) ? 1 : 0); System.err.println();" for call in calls)
        return ("public final class Consumer { public static void main(String[] args) {\n"
                "for(int ia=0;ia<2;ia++) for(int ib=0;ib<2;ib++) for(int ic=0;ic<2;ic++) {\n"
                "boolean a=ia!=0,b=ib!=0,c=ic!=0;\n" + calls + "\n} } }\n")
    calls = "\n".join(f'printf("%d\\n", {call}(a,b,c) ? 1 : 0); (void)fputc(\'\\n\',stderr);' for call in calls)
    return ('#include <stdio.h>\n#include <stdbool.h>\n' + headers +
            'int main(void) {\nfor(int ia=0;ia<2;ia++) { for(int ib=0;ib<2;ib++) { for(int ic=0;ic<2;ic++) {\n'
            'bool a=ia!=0,b=ib!=0,c=ic!=0;\n' + calls + '\n} } } return 0; }\n')


def main():
    java_dir, c_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    root, leaf, java, c, bindings, functions, native, marker_ids = inspect(java_dir, c_dir)
    truth, traces = expected()
    assert len(truth.splitlines()) == len(traces.splitlines()) == 104
    assert run([reference]).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "eager-native"
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

        names = {name: member(bindings[root, "value", name][1]) for name in NAMES}
        markers = {mark: member(identity).split(".")[-1] for mark, identity in marker_ids.items()}
        for variant in ["plain", "traced", "lazy", "duplicate", "reordered", "wrong"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in [leaf, root]:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if variant != "plain":
                    trace_markers = {m: markers[m] for m in ("D" if owner == leaf else "ABC")}
                    text = instrument(text, trace_markers, is_java)
                    if owner == root and variant not in ["traced", "plain"]:
                        count = 2 if variant == "lazy" else 3
                        for name, operator in list(zip(["and", "or", "xor"], ["&", "|", "^"], strict=True))[:count]:
                            text = mutate(text, names[name].split(".")[-1], markers, is_java, variant, operator)
                source_dir = work / owner.split(":")[0]
                source_dir.mkdir()
                source = source_dir / ("Generated.java" if is_java else "generated.c")
                source.write_text(text, encoding="utf-8")
                sources.append(source)
            driver = work / ("Consumer.java" if is_java else "consumer.c")
            headers = "".join(f'#include "{c[owner]["header"]}"\n' for owner in [leaf, root])
            driver.write_text(consumer([names[name] for name in NAMES], is_java, headers), encoding="utf-8")
            if is_java:
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                for source in [*sources, driver]:
                    run([runtime / "javac", *flags, source])
                results = [run([runtime / "java", "-cp", classes, "Consumer"])]
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
                        results.append(run([executable]))
            expected_values, expected_trace = expected(variant)
            for result in results:
                assert result.stdout == expected_values, (is_java, variant, "values")
                assert result.stderr == expected_trace, (is_java, variant, "order/count", result.stderr, expected_trace)
                if variant in ["lazy", "duplicate", "reordered"]:
                    assert result.stdout == truth and result.stderr != traces
                elif variant == "wrong":
                    assert result.stdout != truth and result.stderr == traces
    print("104 exhaustive Rust/C/Java truths and traces; lazy/duplicate/reordered/wrong-operator faults detected")


if __name__ == "__main__":
    main()
