"""Actual bundles: Rust truth plus independently observed native lazy calls."""
import os
from pathlib import Path
import subprocess
import sys

from java_fixture_native import manifest, exports, check_inventory_oracle
from short_circuit_mutations import instrument, mutate
from short_circuit_oracle import NAMES, expected


def run(command):
    result = subprocess.run([str(item) for item in command], capture_output=True,
                            text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result


def main():
    java_bundle, c_bundle, reference, zig = [Path(arg).resolve() for arg in sys.argv[1:]]
    java, c = manifest(java_bundle, 3), manifest(c_bundle, 4)
    assert java["root"] == c["root"] and java["dependencies"] == [] and c["imports"] == []
    bindings = exports(java, True)
    assert bindings == exports(c, False)
    check_inventory_oracle(bindings, {(java["root"], "value", name) for name in NAMES})
    declarations = {item["id"]: item for item in java["declarations"] if item["kind"] == "function"}
    functions = {item["id"]: item for item in c["functions"]}
    assert len(declarations) == len(functions) == 18 and set(declarations) == set(functions)
    markers = {}
    for marker in "ABC":
        found = [item for item in declarations.values()
                 if [doc.strip() for doc in item["documentation"]] == [f"Trace {marker}."]]
        assert len(found) == 1 and not found[0]["externally_reachable"]
        assert found[0]["parameters"] == ["bool"] and found[0]["result"] == "bool"
        markers[marker] = found[0]["id"]
    truth, traces = expected()
    assert len(truth.splitlines()) == len(traces.splitlines()) == 112
    assert run([reference]).stdout == truth
    empty_trace = "\n" * 112
    root = Path(os.environ["TEST_TMPDIR"]) / "short-circuit"
    root.mkdir()
    runtimes = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0]
    assert run(["gcc-14", "-dumpfullversion"]).stdout.strip() == "14.2.0"
    for is_java, bundle, api in [(True, java_bundle, java), (False, c_bundle, c)]:
        member = lambda identity: (declarations[identity]["target"]["path"]["member"] if is_java else functions[identity]["symbol"])
        marker_names = {marker: member(identity) for marker, identity in markers.items()}
        call_names = []
        for name in NAMES:
            kind, identity = bindings[java["root"], "value", name]
            assert kind == "declaration"
            assert declarations[identity]["externally_reachable"]
            assert declarations[identity]["parameters"] == ["bool"] * 3
            assert declarations[identity]["result"] == "bool" and functions[identity]["linkage"] == "external"
            path = declarations[identity]["target"]["path"]
            call_names.append(".".join([path["package"], *path["owners"], path["member"]]) if is_java else member(identity))
        plain = (bundle / (api["source"] if is_java else api["implementation"])).read_text()
        traced = instrument(plain, marker_names, is_java)
        entry = member(bindings[java["root"], "value", "and"][1])
        variants = {"plain": plain, "traced": traced}
        variants.update({fault: mutate(traced, entry, marker_names, is_java, fault)
                         for fault in ["eager", "duplicate", "reordered"]})
        for label, source_text in variants.items():
            work = root / (("java-" if is_java else "c-") + label)
            work.mkdir()
            source = work / ("Generated.java" if is_java else "generated.c")
            source.write_text(source_text, encoding="utf-8")
            consumer = work / ("Consumer.java" if is_java else "consumer.c")
            if is_java:
                calls = "\n".join(f"System.out.println({name}(a,b,c) ? 1 : 0); System.err.println();" for name in call_names)
                consumer.write_text("public final class Consumer { public static void main(String[] args) {\n"
                                    "for(int ia=0;ia<2;ia++) for(int ib=0;ib<2;ib++) for(int ic=0;ic<2;ic++) {\n"
                                    "boolean a=ia!=0,b=ib!=0,c=ic!=0;\n" + calls + "\n} } }\n")
                classes = work / "classes"
                classes.mkdir()
                flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                run([runtime / "javac", *flags, source])
                run([runtime / "javac", *flags, consumer])
                results = [run([runtime / "java", "-cp", classes, "Consumer"])]
            else:
                calls = "\n".join(f'printf("%d\\n", {name}(a,b,c) ? 1 : 0); (void)fputc(\'\\n\', stderr);' for name in call_names)
                consumer.write_text('#include <stdio.h>\n#include <stdbool.h>\n' + f'#include "{c["header"]}"\n'
                                    'int main(void) {\nfor(int ia=0;ia<2;ia++) {\nfor(int ib=0;ib<2;ib++) {\nfor(int ic=0;ic<2;ic++) {\n'
                                    'bool a=ia!=0,b=ib!=0,c=ic!=0;\n' + calls + '\n}\n}\n}\nreturn 0;\n}\n')
                results = []
                flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
                         "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off", "-fsigned-char",
                         "-fno-short-enums", "-I", c_bundle]
                for compiler in ["gcc-14", zig]:
                    for optimization in ["0", "2"]:
                        executable = work / (Path(compiler).name + optimization)
                        run([compiler, *flags, "-O" + optimization, consumer, source, "-o", executable])
                        results.append(run([executable]))
            for result in results:
                assert result.stdout == truth, (is_java, label, "truth mismatch")
                if label == "plain":
                    assert result.stderr == empty_trace
                elif label == "traced":
                    assert result.stderr == traces, (is_java, label, result.stderr, traces)
                else:
                    assert result.stderr != traces, (is_java, label, "bad evaluation order escaped trace oracle")
    print("112 Rust/Java/GCC-O0/O2/Zig-O0/O2 truths and exact call traces agree; eager/duplicate/reordered faults detected")


if __name__ == "__main__":
    main()
