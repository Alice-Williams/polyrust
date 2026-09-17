"""Exact files, source identity, visibility and signature inventory."""
from copy import deepcopy
import json
from binary64_oracle import LITERALS, TRANSPORT, COMPARISONS
from java_fixture_native import exports, check_inventory_oracle


def bundle(directory, java):
    index = json.loads((directory / "bundle.json").read_text())
    assert index["schema_version"] == 1 and len(index["members"]) == 3
    owners, files = {}, {"bundle.json"}
    for member in index["members"]:
        api = json.loads((directory / member["manifest"]).read_text())
        assert api["root"] == member["root"] and api["root"] not in owners
        assert api["schema_version"] == (6 if java else 8)
        owners[api["root"]] = api
        files.add(member["manifest"])
        files.update([api["source"]] if java else [api["header"], api["implementation"]])
    assert files == {str(p.relative_to(directory)) for p in directory.rglob("*") if p.is_file()}
    assert len(files) == (7 if java else 10)
    return index["root"], owners


def signatures(functions, bindings, owners):
    leaf, middle, root = owners
    expected = {
        leaf: {**{name: ([], "f64") for name in LITERALS}, "identity": (["f64"], "f64")},
        middle: {"relay": (["f64"], "f64"), "tenth": ([], "f64")},
        root: {**{name: (["f64"], "f64") for name in TRANSPORT},
               **{name: (["f64", "f64"], "bool") for name in COMPARISONS}, "tenth": ([], "f64")},
    }
    keys = {(owner, "value", name) for owner, items in expected.items() for name in items}
    check_inventory_oracle(bindings, keys)
    for owner, items in expected.items():
        for name, (parameters, result) in items.items():
            function = functions[bindings[owner, "value", name][1]]
            assert function["parameters"] == parameters and function["result"] == result
            assert function["externally_reachable"]


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order = [leaf, middle, root]
    bindings = {}
    for owner in order:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False)
        assert not set(found) & set(bindings)
        bindings.update(found)
    functions = {d["id"]: d for api in java.values() for d in api["declarations"] if d["kind"] == "function"}
    c_functions = {d["id"]: d for api in c.values() for d in api["functions"]}
    assert len(functions) == 30 and set(functions) == set(c_functions)
    signatures(functions, bindings, order)
    for identity, declaration in functions.items():
        assert c_functions[identity]["parameters"] == declaration["parameters"]
        assert c_functions[identity]["return"] == declaration["result"]
        assert c_functions[identity]["linkage"] == ("external" if declaration["externally_reachable"] else "internal")
    for owner in order:
        for imported in c[owner]["imports"]:
            declaration = functions[imported["id"]]
            assert imported["parameters"] == declaration["parameters"] and imported["return"] == declaration["result"]
            assert imported["header"] == c[imported["owner"]]["header"]
            assert imported["symbol"] == c_functions[imported["id"]]["symbol"]
    fields = [d for d in java[root]["declarations"] if d["kind"] == "field"]
    assert len(fields) == 2
    assert all(d["scalar"] == "f64" and not d["externally_reachable"] for d in fields)
    markers = {}
    for marker in "LR":
        found = [d for d in functions.values() if [s.strip() for s in d["documentation"]] == [f"Trace {marker}."]]
        assert len(found) == 1 and not found[0]["externally_reachable"]
        markers[marker] = found[0]["id"]
    for position in ["parameters", "result"]:
        changed = deepcopy(functions)
        declaration = changed[bindings[root, "value", "identity"][1]]
        declaration[position] = ["i64"] if position == "parameters" else "i64"
        try:
            signatures(changed, bindings, order)
        except AssertionError:
            pass
        else:
            raise AssertionError("f64-to-i64 metadata substitution escaped")
    return order, java, c, bindings, functions, c_functions, markers
