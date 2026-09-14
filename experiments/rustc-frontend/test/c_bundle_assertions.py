"""Parse actual bundle files; imports never enter the owned function inventory."""
import json
import re


def check(directory, count, expected_root_imports):
    index = json.loads((directory / "bundle.json").read_text())
    assert set(index) == {"schema_version", "root", "members"}
    assert index["schema_version"] == 1 and len(index["members"]) == count
    assert [item["root"] for item in index["members"]] == sorted(item["root"] for item in index["members"])
    manifests = {}
    expected_files = {"bundle.json"}
    all_symbols = set()
    for item in index["members"]:
        assert set(item) == {"root", "manifest"}
        root = item["root"]
        stem = "polyrust_" + root.split(":")[0]
        assert item["manifest"] == stem + ".api.json"
        manifest = json.loads((directory / item["manifest"]).read_text())
        assert set(manifest) == {"schema_version", "root", "header", "implementation", "modules", "functions", "imports"}
        assert manifest["schema_version"] == 2 and manifest["root"] == root
        assert manifest["header"] == stem + ".h"
        assert manifest["implementation"] == stem + ".c"
        expected_files.update([item["manifest"], stem + ".h", stem + ".c"])
        assert root not in manifests
        manifests[root] = manifest
        definitions = {function["id"]: function for function in manifest["functions"]}
        assert len(definitions) == len(manifest["functions"])
        assert all(identity.split(":")[0] == root.split(":")[0] for identity in definitions)
        header = (directory / manifest["header"]).read_text()
        source = (directory / manifest["implementation"]).read_text()
        for function in definitions.values():
            symbol = function["symbol"]
            assert function["implementation"] == manifest["implementation"]
            assert re.search(r"\b" + re.escape(symbol) + r"\([^;{}]*\)\s*\{", source)
            if function["linkage"] == "external":
                assert symbol not in all_symbols and symbol in header
                all_symbols.add(symbol)
                assert function["primary"] == manifest["header"]
            else:
                assert function["linkage"] == "internal" and symbol not in header
                assert function["primary"] == manifest["implementation"]
        imports = manifest["imports"]
        assert [value["id"] for value in imports] == sorted({value["id"] for value in imports})
        assert not definitions.keys() & {value["id"] for value in imports}
        includes = set(re.findall(r'^#include "([^"]+)"', source, re.MULTILINE))
        assert includes == {manifest["header"]} | {value["header"] for value in imports}
    assert index["root"] in manifests
    assert len(manifests[index["root"]]["imports"]) == expected_root_imports
    assert {path.name for path in directory.iterdir()} == expected_files
    assert len(expected_files) == 3 * count + 1
    for root, manifest in manifests.items():
        for imported in manifest["imports"]:
            assert set(imported) == {"id", "owner", "header", "symbol", "return", "parameters"}
            assert imported["owner"] != root and imported["owner"] in manifests
            owner = manifests[imported["owner"]]
            assert imported["header"] == owner["header"]
            function = next(value for value in owner["functions"] if value["id"] == imported["id"])
            assert function["linkage"] == "external" and function["symbol"] == imported["symbol"]
            assert imported["return"] in {"i32", "bool"}
            assert all(value in {"i32", "bool"} for value in imported["parameters"])
    return {path.name: path.read_bytes() for path in directory.iterdir()}
