"""Exact scalar values across separately compiled runtime-free packages."""
import os
from pathlib import Path
import subprocess
import sys

from local_constant_inventory import inspect
from local_constant_oracle import CASES, expected


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout[:1000], result.stderr[:4000])
    return result


def consumer(calls, java, headers):
    if java:
        statements = "\n".join(
            f"System.out.println({calls[name]}(flag)" + (" ? 1 : 0" if ty == "bool" else "") + ");"
            for name, ty, *_ in CASES)
        return ("public final class Consumer { public static void main(String[] args) {\n"
                "for (boolean flag : new boolean[]{false,true}) {\n" + statements + "\n} } }\n")
    statements = "\n".join(f'printf("%lld\\n", (long long){calls[name]}(flag));' for name, *_ in CASES)
    return ('#include <stdio.h>\n#include <stdbool.h>\n' + headers +
            'int main(void) { for(int i=0;i<2;i++) { bool flag=i!=0;\n' +
            statements + '\n} return 0; }\n')


def main():
    java_dir, c_dir, reference, zig = [Path(p).resolve() for p in sys.argv[1:]]
    root, leaf, java, c, bindings, functions, native = inspect(java_dir, c_dir)
    truth = expected()
    assert len(truth.splitlines()) == 30
    assert run([reference]).stdout == truth
    work_root = Path(os.environ["TEST_TMPDIR"]) / "local-constant-native"
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

        names = {name: member(bindings[root, "value", name][1]) for name, *_ in CASES}
        for variant in ["plain", "wrong"]:
            work = work_root / (("java-" if is_java else "c-") + variant)
            work.mkdir()
            sources = []
            for owner in [leaf, root]:
                text = (directory / owners[owner]["source" if is_java else "implementation"]).read_text()
                if variant == "wrong" and owner == root:
                    # Replace the uniquely occurring computed constant; all other values stay intact.
                    import re
                    text, count = re.subn(r"(?<![\w])62(?![\w])", "17", text)
                    assert count == 1, (is_java, count)
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
            for result in results:
                assert result.stdout == expected(variant == "wrong"), (is_java, variant, result.stdout)
                assert not result.stderr
                assert (result.stdout == truth) == (variant == "plain")
    print("30 exact Rust/C/Java constant results, two crates, GCC/Zig O0/O2; wrong-value faults detected")



if __name__ == "__main__":
    main()
