"""Exact source-owned artifact inventories; not a semantic-equivalence proof."""
import copy
import json
from pathlib import Path, PurePosixPath
import re
import sys


def relative(name):
    path = PurePosixPath(name)
    assert name and not path.is_absolute() and ".." not in path.parts
    assert "\\" not in name and str(path) == name
    return name


def verify(files, language):
    for name in files:
        relative(name)
    index = json.loads(files["bundle.json"])
    assert set(index) == {"schema_version", "root", "members"} | ({"target"} if language == "java" else set())
    assert index["schema_version"] == 1
    if language == "java":
        assert index["target"] == "org.polyrust.java"
    members = index["members"]
    roots = [member["root"] for member in members]
    assert len(roots) == 4 and len(set(roots)) == 4 and index["root"] in roots
    manifests = {member["root"]: json.loads(files[member["manifest"]]) for member in members}
    expected = {"bundle.json"}
    for member in members:
        assert set(member) == {"root", "manifest"} | ({"source"} if language == "java" else set())
        root = member["root"]
        assert re.fullmatch(r"[0-9a-f]{16}:[0-9a-f]{16}", root)
        crate = root.split(":")[0]
        manifest_path = relative(member["manifest"])
        assert manifest_path not in expected and manifest_path == f"polyrust_{crate}.api.json"
        expected.add(manifest_path)
        manifest = manifests[root]
        assert manifest["root"] == root
        if language == "c":
            assert set(manifest) == {"schema_version", "root", "header", "implementation", "modules", "functions", "imports"}
            assert manifest["schema_version"] == 2
            artifacts = [f"polyrust_{crate}.h", f"polyrust_{crate}.c"]
            assert manifest["header"] == artifacts[0]
            assert manifest["implementation"] == artifacts[1]
            imported_headers = set()
            imports = manifest["imports"]
            assert len({item["id"] for item in imports}) == len(imports)
            for item in imports:
                assert set(item) == {"id", "owner", "header", "symbol", "return", "parameters"}
                assert item["owner"] in roots and item["owner"] != root
                owner = manifests[item["owner"]]
                assert item["header"] == owner["header"]
                definitions = [function for function in owner["functions"] if function["id"] == item["id"]]
                assert len(definitions) == 1
                definition = definitions[0]
                assert definition["symbol"] == item["symbol"] and definition["linkage"] == "external"
                assert definition["primary"] == owner["header"]
                imported_headers.add(item["header"])
        else:
            assert set(manifest) == {"schema_version", "root", "defining_key", "source", "modules", "declarations", "dependencies"}
            assert manifest["schema_version"] == 1
            artifacts = [f"src/main/java/org/polyrust/generated/r{crate}/Generated.java"]
            assert member["source"] == manifest["source"] == artifacts[0]
            dependencies = manifest["dependencies"]
            assert len(set(dependencies)) == len(dependencies)
            assert root not in dependencies and set(dependencies) <= set(roots)
        for artifact in artifacts:
            assert artifact not in expected
            expected.add(artifact)
            text = files[artifact].decode()
            assert text.strip(), "empty generated source"
            assert "org.polyrust.generated.Runtime" not in text
            if language == "c":
                includes = re.findall(r"(?m)^\s*#\s*include\b([^\r\n]*)", text)
                actual_includes = [include.strip() for include in includes]
                required = ({"<stdint.h>", "<stddef.h>"} if artifact.endswith(".h") else
                            {"<stdint.h>"} | {'"' + header + '"' for header in imported_headers | {manifest["header"]}})
                assert len(actual_includes) == len(set(actual_includes))
                assert set(actual_includes) == required
            else:
                # This fixed scalar corpus needs no imports. Fully qualified
                # expression bindings remain the typed/native gates' job.
                assert not re.search(r"(?m)^\s*import\b", text)
    assert set(files) == expected, (language, set(files) ^ expected)
    return expected


def reject(files, language, label):
    try:
        verify(files, language)
    except (AssertionError, KeyError):
        return
    raise AssertionError((language, label, "artifact corruption accepted"))


def check(root, language):
    files = {path.relative_to(root).as_posix(): path.read_bytes()
             for path in root.rglob("*") if path.is_file()}
    verify(files, language)
    suffix = "c" if language == "c" else "java"
    for name in [f"runtime.{suffix}", f"UnrelatedSupport.{suffix}"]:
        assert name not in files
        reject(dict(files, **{name: b"/* unexpected support file */"}), language, name)
    source = next(name for name in files if name.endswith("." + suffix))
    missing = dict(files)
    del missing[source]
    reject(missing, language, "missing owned source")
    duplicate = dict(files)
    index = json.loads(duplicate["bundle.json"])
    index["members"].append(copy.deepcopy(index["members"][0]))
    duplicate["bundle.json"] = json.dumps(index).encode()
    reject(duplicate, language, "duplicate source crate")
    stale = dict(files)
    stale[source] += (b'\n#include "runtime.h"\n' if language == "c"
                      else b"\n// org.polyrust.generated.Runtime\n")
    reject(stale, language, "legacy runtime reference")
    dependency_controls(files, language, source)
    print(f"{language}: exact four-crate artifacts, schemas and declared dependency closure; fault controls rejected")


def dependency_controls(files, language, source):
    index = json.loads(files["bundle.json"])
    member = index["members"][0]
    manifest_path = member["manifest"]
    for level in ["index", "member", "manifest"]:
        changed = dict(files)
        path = manifest_path if level == "manifest" else "bundle.json"
        value = json.loads(changed[path])
        target = value["members"][0] if level == "member" else value
        target["runtime_dependency"] = "RenamedSupport"
        changed[path] = json.dumps(value).encode()
        reject(changed, language, "unknown " + level + " field")
    changed = dict(files)
    manifest = json.loads(changed[manifest_path])
    manifest["schema_version"] = 999
    changed[manifest_path] = json.dumps(manifest).encode()
    reject(changed, language, "unknown manifest version")
    if language == "c":
        for field, value in [("header", "RenamedSupport.h"), ("symbol", "renamed_support"),
                             ("owner", "ffffffffffffffff:ffffffffffffffff")]:
            changed = dict(files)
            path = next(member["manifest"] for member in index["members"]
                        if json.loads(files[member["manifest"]])["imports"])
            manifest = json.loads(files[path])
            manifest["imports"][0][field] = value
            changed[path] = json.dumps(manifest).encode()
            reject(changed, language, "forged C " + field)
        additions = [b'\n#include <runtime.h>\n', b'\n#include "RenamedSupport.h"\n']
    else:
        for dependencies in [[member["root"]], ["ffffffffffffffff:ffffffffffffffff"],
                             [index["members"][1]["root"]] * 2]:
            changed = dict(files)
            manifest = json.loads(files[manifest_path])
            manifest["dependencies"] = dependencies
            changed[manifest_path] = json.dumps(manifest).encode()
            reject(changed, language, "invalid Java dependency owners")
        additions = [b"\nimport org.example.PolyrustSupport;\n",
                     b"\nimport static org.example.RenamedSupport.run;\n"]
    for addition in additions:
        changed = dict(files)
        changed[source] += addition
        reject(changed, language, "unexpected source dependency declaration")


def main():
    c, java = [Path(value).resolve() for value in sys.argv[1:]]
    check(c, "c")
    check(java, "java")


if __name__ == "__main__":
    main()
