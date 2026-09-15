"""Closed two-owner inventory and independent exact-width signature expectations."""
from copy import deepcopy
import json
from java_fixture_native import exports, check_inventory_oracle
from i64_oracle import WIDE, BOOL


def bundle(directory, java):
    index = json.loads((directory / "bundle.json").read_text())
    assert set(index) == ({"schema_version", "root", "members", "target"} if java else
                          {"schema_version", "root", "members"})
    assert index["schema_version"] == 1 and len(index["members"]) == 2
    if java:
        assert index["target"] == "org.polyrust.java"
    files, owners = {"bundle.json"}, {}
    for member in index["members"]:
        assert set(member) == ({"root", "source", "manifest"} if java else {"root", "manifest"})
        api = json.loads((directory / member["manifest"]).read_text())
        assert member["root"] == api["root"] and api["root"] not in owners
        assert set(api) == ({"schema_version", "root", "defining_key", "source", "modules", "declarations", "dependencies"}
                            if java else {"schema_version", "root", "header", "implementation", "modules", "functions", "imports"})
        assert api["schema_version"] == (1 if java else 2)
        owners[api["root"]] = api
        files.add(member["manifest"])
        if java:
            assert member["source"] == api["source"]
            files.add(api["source"])
        else:
            files.update([api["header"], api["implementation"]])
    assert index["root"] in owners
    assert len(files) == (5 if java else 7)
    assert files == {str(path.relative_to(directory)) for path in directory.rglob("*") if path.is_file()}
    return index["root"], owners


def signatures(functions, bindings, root, leaf):
    expected = {name: (["i64", "i64", "bool"], "i64") for name in WIDE}
    expected.update({name: (["i64", "i64", "bool"], "bool") for name in BOOL})
    expected["exported_identity"] = (["i64", "i64", "bool"], "i64")
    leaf_expected = {"identity": (["i64"], "i64"), "choose": (["i64", "i32", "i64", "bool"], "i64"),
                     "equal": (["i64", "i64"], "bool"), "narrow": (["i32"], "i32")}
    for owner, expectations in [(root, expected), (leaf, leaf_expected)]:
        for name, (parameters, result) in expectations.items():
            kind, identity = bindings[owner, "value", name]
            assert kind == "declaration"
            declaration = functions[identity]
            assert declaration["parameters"] == parameters and declaration["result"] == result
            assert declaration["externally_reachable"]


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert c_root == root and set(c) == set(java)
    leaf, = set(java) - {root}
    assert java[root]["dependencies"] == [leaf] and java[leaf]["dependencies"] == []
    assert c[leaf]["imports"] == [] and len(c[root]["imports"]) == 4
    bindings = {}
    for owner in java:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    keys = {(root, "value", name) for name in WIDE + BOOL + ["exported_identity"]}
    keys.update((leaf, "value", name) for name in ["identity", "choose", "equal", "narrow"])
    check_inventory_oracle(bindings, keys)
    assert bindings[root, "value", "exported_identity"] == bindings[root, "value", "identity"]
    functions = {item["id"]: item for owner in java.values() for item in owner["declarations"] if item["kind"] == "function"}
    c_functions = {item["id"]: item for owner in c.values() for item in owner["functions"]}
    assert len(functions) == len(c_functions) == 27 and set(functions) == set(c_functions)
    signatures(functions, bindings, root, leaf)
    for imported in c[root]["imports"]:
        declaration = functions[imported["id"]]
        assert imported["owner"] == leaf and imported["header"] == c[leaf]["header"]
        assert imported["return"] == declaration["result"] and imported["parameters"] == declaration["parameters"]
        assert imported["symbol"] == c_functions[imported["id"]]["symbol"]
    # A common-mode metadata narrowing cannot satisfy the independent fixture contract.
    for position in ["result", "parameters"]:
        changed = deepcopy(functions)
        identity = bindings[root, "value", "identity"][1]
        if position == "result":
            changed[identity][position] = "i32"
        else:
            changed[identity][position][0] = "i32"
        try:
            signatures(changed, bindings, root, leaf)
        except AssertionError:
            pass
        else:
            raise AssertionError("narrowed signature escaped oracle")
    return root, leaf, java, c, bindings, functions, c_functions
