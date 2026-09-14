"""Independent full-corpus Rust/C/Java consumers of actual single-owner bundles."""
import json
import os
from pathlib import Path
import subprocess
import sys


def run(command, *, inputs=None):
    result = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout


def manifest(bundle, count):
    index = json.loads((bundle / "bundle.json").read_text())
    assert len(index["members"]) == 1
    owner = index["members"][0]
    api = json.loads((bundle / owner["manifest"]).read_text())
    assert index["root"] == owner["root"] == api["root"]
    payloads = {"bundle.json", owner["manifest"]}
    payloads.update([api["source"]] if count == 3 else [api["header"], api["implementation"]])
    actual = {str(path.relative_to(bundle)) for path in bundle.rglob("*") if path.is_file()}
    assert actual == payloads and len(actual) == count
    return api


def exports(api, java):
    result = {}
    for module in api["modules"]:
        for binding in module["bindings"]:
            key = (module["id"], binding["namespace"], binding["name"])
            assert key not in result
            result[key] = ((binding["target"]["kind"], binding["target"]["id"]) if java else
                           ("declaration" if binding["kind"] == "function" else binding["kind"], binding["target"]))
    return result


def exact_keys(bindings, expected):
    assert set(bindings) == expected, "source export inventory differs from the fixture"


def check_inventory_oracle(bindings, expected):
    exact_keys(bindings, expected)
    # Both backends could share a projection bug. Prove a common-mode extra
    # alias in every exported module cannot pass this independent fixture list.
    target = next(value for value in bindings.values() if value[0] == "declaration")
    for module in {key[0] for key in expected}:
        changed = dict(bindings)
        changed[module, "value", "unexpected_alias"] = target
        try:
            exact_keys(changed, expected)
        except AssertionError:
            pass
        else:
            raise AssertionError("extra-alias mutation passed the inventory oracle")


def main():
    case = sys.argv[1]
    java_bundle, c_bundle, reference, seeds, zig = [Path(arg).resolve() for arg in sys.argv[2:]]
    java, c = manifest(java_bundle, 3), manifest(c_bundle, 4)
    assert java["root"] == c["root"] and java["dependencies"] == [] and c["imports"] == []
    bindings = exports(java, True)
    assert bindings == exports(c, False), "C and Java disagree on source export identities"
    root = java["root"]

    def binding(module, namespace, name, kind):
        actual_kind, identity = bindings[module, namespace, name]
        assert actual_kind == kind
        return identity

    def function(name, module=root):
        return binding(module, "value", name, "declaration")

    if case == "public_package":
        api = binding(root, "type", "api", "module")
        assert binding(api, "type", "again", "module") == api, "cyclic module alias lost"
        assert function("exported") == function("alias") == function("invoke", api)
        expected_bindings = {(root, "value", name) for name in ["zero", "choose", "positive", "exported", "alias"]}
        expected_bindings.update([(root, "type", "api"), (api, "type", "again"), (api, "value", "invoke")])
        calls = [(function("zero"), []), (function("exported"), ["value"]),
                 (function("alias"), ["value"]), (function("invoke", api), ["value"]),
                 (function("positive"), ["value"]),
                 (function("choose"), ["value", "true", "-1", "false"]),
                 (function("choose"), ["value", "false", "-1", "true"]),
                 (function("choose"), ["value", "false", "-1", "false"])]
    else:
        assert case == "same_spelling"
        first = binding(root, "type", "first", "module")
        second = binding(root, "type", "second", "module")
        assert first != second
        assert function("first_value") == function("value", first)
        assert function("second_value") == function("value", second)
        assert function("first_value") != function("second_value")
        expected_bindings = {(root, "type", "first"), (root, "type", "second"),
                             (root, "value", "first_value"), (root, "value", "second_value"),
                             (first, "value", "value"), (second, "value", "value")}
        calls = [(function(name), ["value"]) for name in ["first_value", "second_value"]]
    check_inventory_oracle(bindings, expected_bindings)
    declarations = {item["id"]: item for item in java["declarations"]}
    functions = {item["id"]: item for item in c["functions"]}
    assert len(declarations) == len(java["declarations"])
    assert len(functions) == len(c["functions"])
    assert set(functions) == {key for key, item in declarations.items() if item["kind"] == "function"}
    java_calls, c_calls = [], []
    for identity, arguments in calls:
        declaration, native = declarations[identity], functions[identity]
        assert declaration["kind"] == "function" and declaration["externally_reachable"]
        assert native["linkage"] == "external"
        assert len(arguments) == len(declaration["parameters"])
        path = declaration["target"]["path"]
        name = ".".join([path["package"], *path["owners"], path["member"]])
        expression = f'{name}({", ".join(arguments)})'
        if declaration["result"] == "bool":
            expression = f"({expression} ? 1 : 0)"
        java_calls.append(expression)
        c_calls.append(f'(int32_t){native["symbol"]}({", ".join(arguments)})')
    if case == "same_spelling":
        assert java_calls[0] != java_calls[1] and c_calls[0] != c_calls[1]

    work = Path(os.environ["TEST_TMPDIR"]) / case
    work.mkdir()
    java_consumer = work / "Consumer.java"
    java_consumer.write_text(
        "public final class Consumer {\n"
        " public static void main(String[] args) throws java.io.IOException {\n"
        "  var reader = new java.io.BufferedReader(new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));\n"
        "  String line; while ((line = reader.readLine()) != null) {\n"
        "   int value = Integer.parseInt(line); System.out.println("
        + ' + " " + '.join(java_calls) + ");\n  }\n }\n}\n", encoding="utf-8")
    runtimes = [path for path in Path(os.environ["RUNFILES_DIR"]).iterdir()
                if "remotejdk21" in path.name and (path / "bin/javac").is_file()]
    assert len(runtimes) == 1
    runtime = runtimes[0] / "bin"
    classes = work / "classes"
    classes.mkdir()
    compile_flags = ["--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
                     "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes]
    run([runtime / "javac", *compile_flags, java_bundle / java["source"]])
    run([runtime / "javac", *compile_flags, java_consumer])
    c_consumer = work / "consumer.c"
    formats = ' " " '.join(['"%" PRId32'] * len(c_calls))
    c_consumer.write_text(
        '#include <inttypes.h>\n#include <stdbool.h>\n#include <stdio.h>\n'
        + f'#include "{c["header"]}"\n'
        + 'int main(void) { int32_t value; while (scanf("%" SCNd32, &value) == 1) {\n'
        + f'printf({formats} "\\n", {", ".join(c_calls)});\n'
        + '} return 0; }\n', encoding="utf-8")
    values = seeds.read_text().splitlines() + [str(value) for value in range(-4096, 4097)]
    assert len(values) == 8204
    inputs = "\n".join(values) + "\n"
    expected = run([reference], inputs=inputs)
    # Fixture truth is independent of both compilers and manifest-driven consumers.
    truth = [([42, int(value), int(value), int(value), int(int(value) > 0), int(value), -1, 42]
              if case == "public_package" else [int(value), int(value)]) for value in values]
    assert expected == "".join(" ".join(map(str, row)) + "\n" for row in truth)
    assert run([runtime / "java", "-cp", classes, "Consumer"], inputs=inputs) == expected
    assert run(["gcc-14", "-dumpfullversion"]).strip() == "14.2.0"
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror", "-Wstrict-prototypes",
             "-Wmissing-prototypes", "-fno-fast-math", "-ffp-contract=off", "-fsigned-char",
             "-fno-short-enums", "-I", c_bundle]
    for compiler in ["gcc-14", zig]:
        for optimization in ["0", "2"]:
            executable = work / (Path(compiler).name + optimization)
            run([compiler, *flags, "-O" + optimization, c_consumer,
                 c_bundle / c["implementation"], "-o", executable])
            assert run([executable], inputs=inputs) == expected
    print(f"{case}: 8204 x {len(calls)} Rust/Java/GCC-O0/O2/Zig-O0/O2 values agree; exact export identities checked")


if __name__ == "__main__":
    main()
