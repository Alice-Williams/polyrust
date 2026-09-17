"""Independent Rust/C/Java values and exact source-alias publication inventories."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

NAMES = ["FALSE", "TRUE", "I32_MIN", "I32_MAX", "I64_MIN", "I64_MAX",
         "FORWARD", "WIDE", "NEGATIVE_WIDE", "COMPUTED", "EXTRA",
         "RENAMED", "OWN", "read"]
TRUTH = [0, 1, -2147483648, 2147483647, -9223372036854775808,
         9223372036854775807, 9007199254740993, 9007199254740993,
         -9007199254740993, 62, -17, 62, 42, 62]


def run(command):
    result = subprocess.run([str(value) for value in command], capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout


def inventory(directory):
    index = json.loads((directory / "bundle.json").read_text())
    assert index["schema_version"] == 1
    return index["root"], {item["root"]: json.loads((directory / item["manifest"]).read_text())
                          for item in index["members"]}


def java_path(value):
    return ".".join([value["package"], *value["owners"], value["member"]])


def exact_c_aliases(directory, owner, manifest, constants):
    expected = []
    for module in sorted(manifest["modules"], key=lambda item: item["id"]):
        for binding in sorted(module["bindings"], key=lambda item: (item["namespace"], item["name"])):
            declaration = binding["target"]
            if binding["namespace"] != "value" or declaration not in constants:
                continue
            producer, value = constants[declaration]
            if producer == owner:
                continue
            expected.append(dict(module=module["id"], namespace="value", name=binding["name"],
                                 id=declaration, owner=producer,
                                 header="polyrust_" + producer.split(":")[0] + ".h",
                                 symbol=value["symbol"], type=value["type"], value=value["value"],
                                 readonly=True))
    assert manifest["constant_exports"] == expected
    raw = (directory / ("polyrust_" + owner.split(":")[0] + ".api.json")).read_text()
    exact = '"constant_exports":' + json.dumps(expected, ensure_ascii=False, separators=(",", ":"))
    assert exact in raw


def evidence(c_dir, j_dir, mixed):
    root, c = inventory(c_dir)
    jroot, java = inventory(j_dir)
    assert root == jroot and set(c) == set(java)
    assert len(c) == (4 if mixed else 3)
    constants = {value["id"]: (owner, value) for owner, manifest in c.items()
                 for value in manifest.get("constants", [])}
    jconstants = {value["id"]: (owner, value) for owner, manifest in java.items()
                  for value in manifest["declarations"] if value["kind"] == "constant"}
    assert set(constants) == set(jconstants)
    assert len(constants) == (12 if mixed else 11)
    for owner in c:
        cc, jj = c[owner], java[owner]
        ce = {(value["module"], value["namespace"], value["name"]): value
              for value in cc.get("constant_exports", [])}
        je = {(value["module"], value["namespace"], value["name"]): value
              for value in jj.get("constant_exports", [])}
        assert set(ce) == set(je)
        if ce:
            assert cc["schema_version"] == 6 and jj["schema_version"] == 4
            assert len(ce) == 12  # Eleven constants plus renamed/nested binding.
            assert not {value["id"] for value in cc.get("constants", [])} & {value["id"] for value in ce.values()}
            assert not {value["id"] for value in jj["declarations"]} & {value["id"] for value in je.values()}
            exact_c_aliases(c_dir, owner, cc, constants)
            marker = ("Mixed source owner: naïve Δ" if mixed and owner == root
                      else "Alias-only crate: café λ")
            markers = [marker]
            if not (mixed and owner == root):
                markers.append("Finite local module-alias cycle: façade 日本語.")
            for marker in markers:
                # C's documented portable-comment profile displays UTF-8 bytes
                # as [0xNN]; Java preserves these Unicode characters directly.
                c_marker = "".join(chr(byte) if 32 <= byte < 127 else f"[0x{byte:02X}]"
                                   for byte in marker.encode("utf-8"))
                assert c_marker in (c_dir / cc["header"]).read_text()
                assert marker in (j_dir / jj["source"]).read_text()
            assert len(cc.get("constants", [])) == int(mixed and owner == root)
            assert len(cc["functions"]) == int(mixed and owner == root)
        else:
            assert cc["schema_version"] == 4 and jj["schema_version"] == 2
        cb = {(module["id"], value["namespace"], value["name"]): value["target"]
              for module in cc["modules"] for value in module["bindings"]}
        jb = {(module["id"], value["namespace"], value["name"]): value["target"]["id"]
              for module in jj["modules"] for value in module["bindings"]}
        assert cb == jb
        for key, alias in ce.items():
            original_owner, original = constants[alias["id"]]
            java_owner, java_original = jconstants[alias["id"]]
            assert cb[key] == alias["id"] == je[key]["id"]
            assert alias["owner"] == je[key]["owner"] == original_owner == java_owner
            assert alias["header"] == c[original_owner]["header"]
            assert alias["symbol"] == original["symbol"]
            assert alias["type"] == je[key]["scalar"] == original["type"]
            assert alias["value"] == je[key]["value"] == original["value"]
            assert alias["readonly"] is je[key]["readonly"] is True
            assert je[key]["path"] == java_original["target"]["path"]
        ci = {value["id"] for value in cc.get("constant_imports", [])}
        ji = {value["id"] for value in jj.get("constant_imports", [])}
        assert ci == ji
        expected_reads = {cb[(owner, "value", "COMPUTED")]} if mixed and owner == root else set()
        assert ci == expected_reads
    bindings = {value["name"]: value["target"] for module in c[root]["modules"]
                if module["id"] == root for value in module["bindings"]
                if value["namespace"] == "value"}
    assert set(bindings) == set(NAMES if mixed else NAMES[:11])
    cfunctions = {value["id"]: value for manifest in c.values() for value in manifest["functions"]}
    jfunctions = {value["id"]: value for manifest in java.values() for value in manifest["declarations"]
                  if value["kind"] == "function"}
    cexpressions, jexpressions = [], []
    for name in NAMES if mixed else NAMES[:11]:
        declaration = bindings[name]
        if name == "read":
            cexpressions.append(cfunctions[declaration]["symbol"] + "()")
            jexpressions.append(java_path(jfunctions[declaration]["target"]["path"]) + "()")
        else:
            cexpressions.append(constants[declaration][1]["symbol"])
            jexpressions.append(java_path(jconstants[declaration][1]["target"]["path"]))
    computed_owner = constants[bindings["COMPUTED"]][0]
    return root, c, java, cexpressions, jexpressions, computed_owner


def main():
    cm, jm, cr, jr, reference, zig = [Path(value).resolve() for value in sys.argv[1:]]
    assert run([reference]).splitlines() == list(map(str, TRUTH))
    jdks = [path / "bin" for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
            if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(jdks) == 1
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-exports-native"
    work.mkdir()
    for mixed, c_dir, j_dir in [(False, cm, jm), (True, cr, jr)]:
        root, c, java, cexpr, jexpr, computed = evidence(c_dir, j_dir, mixed)
        truth = TRUTH if mixed else TRUTH[:11]
        for target, original, expressions in [("c", c_dir, cexpr), ("java", j_dir, jexpr)]:
            assert not any("runtime" in path.name.lower() for path in original.rglob("*"))
            for mutant in [False, True]:
                trial = work / f"{target}-{mixed}-{mutant}"
                shutil.copytree(original, trial)
                source = trial / (c[computed]["implementation"] if target == "c" else java[computed]["source"])
                if mutant:
                    changed, count = re.subn(r"(?<![\w])62(?![\w])", "17", source.read_text())
                    assert count == 1
                    source.write_text(changed)
                results = []
                if target == "java":
                    classes = trial / "classes"
                    classes.mkdir()
                    flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                             "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
                    order = sorted(java, key=lambda owner: {"constants": 0, "second": 1, "middle": 2, "root": 3}[
                        java[owner]["defining_key"].rsplit(".", 1)[-1]])
                    for owner in order:
                        run([jdks[0] / "javac", *flags, trial / java[owner]["source"]])
                    lines = [f"System.out.println({expression}" + (" ? 1 : 0" if i < 2 else "") + ");"
                             for i, expression in enumerate(expressions)]
                    alias_owner = next(owner for owner in java if java[owner]["defining_key"].endswith(".middle"))
                    alias_class = "org.polyrust.generated.r" + alias_owner.split(":")[0] + ".Generated"
                    lines += [f'Class<?> alias = Class.forName("{alias_class}");',
                              'if (alias.getDeclaredFields().length != 0 || alias.getDeclaredMethods().length != 0) throw new AssertionError("copied alias storage");']
                    consumer = trial / "Consumer.java"
                    consumer.write_text("public final class Consumer { public static void main(String[] args) throws Exception {\n"
                                        + "\n".join(lines) + "\n} }\n")
                    # Recompile readers and consumer after every producer mutation.
                    run([jdks[0] / "javac", *flags, consumer])
                    results.append(run([jdks[0] / "java", "-cp", classes, "Consumer"]))
                else:
                    consumer = trial / "consumer.c"
                    consumer.write_text('#include <stdio.h>\n#include "' + c[root]["header"] + '"\nint main(void) {\n'
                                        + "\n".join(f'printf("%lld\\n", (long long)({expression}));' for expression in expressions)
                                        + "\nreturn 0;\n}\n")
                    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
                             "-Wstrict-prototypes", "-Wmissing-prototypes", "-I", trial]
                    for compiler in ["gcc-14", zig]:
                        for optimization in ["0", "2"]:
                            for owner in c:
                                run([compiler, *flags, "-x", "c", "-c", trial / c[owner]["header"], "-o", trial / "header-check.o"])
                            objects = []
                            for index, unit in enumerate([trial / c[owner]["implementation"] for owner in sorted(c)] + [consumer]):
                                obj = trial / f"unit{index}.o"
                                run([compiler, *flags, "-O" + optimization, "-c", unit, "-o", obj])
                                objects.append(obj)
                            binary = trial / "consumer"
                            run([compiler, *objects, "-o", binary])
                            results.append(run([binary]))
                expected = [17 if mutant and value == 62 else value for value in truth]
                for result in results:
                    assert result.splitlines() == list(map(str, expected))
                    assert (result.splitlines() == list(map(str, truth))) != mutant
                if not mutant and os.environ.get("TEST_UNDECLARED_OUTPUTS_DIR"):
                    artifact = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / f"{target}-aliases-{mixed}"
                    shutil.copytree(original, artifact)
                    shutil.copy2(consumer, artifact / consumer.name)
    print("Alias-only/mixed source graphs: exact manifests; Rust, GCC/Zig O0/O2, Java21; independent producer mutation")


if __name__ == "__main__":
    main()
