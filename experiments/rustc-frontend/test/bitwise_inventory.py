"""Exact bitwise fixture interfaces and transitive library provenance."""
from i64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from bitwise_oracle import NAMES


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    leaf, = set(java) - {root}
    assert java[root]["dependencies"] == [leaf] and java[leaf]["dependencies"] == []
    assert not c[leaf]["imports"] and len(c[root]["imports"]) == 2
    bindings = {}
    for owner in java:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False)
        assert not set(found) & set(bindings)
        bindings.update(found)
    keys = {(root, "value", name + str(w)) for w in [32, 64] for name in NAMES}
    keys.update((leaf, "value", "invert" + str(w)) for w in [32, 64])
    check_inventory_oracle(bindings, keys)
    functions = {d["id"]: d for owner in java.values() for d in owner["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for owner in c.values() for d in owner["functions"]}
    assert len(functions) == len(native) == 22 and set(functions) == set(native)
    for owner, names, count in [(root, NAMES, 2), (leaf, ["invert"], 1)]:
        for w in [32, 64]:
            for name in names:
                kind, identity = bindings[owner, "value", name + str(w)]
                assert kind == "declaration"
                d = functions[identity]
                assert d["parameters"] == ["i" + str(w)] * count and d["result"] == "i" + str(w)
                assert d["externally_reachable"] and native[identity]["linkage"] == "external"
    markers = {}
    for w in [32, 64]:
        for side in "LR":
            marker = side + str(w)
            found = [d for d in functions.values() if [s.strip() for s in d["documentation"]] == [f"Trace {marker}."]]
            assert len(found) == 1 and not found[0]["externally_reachable"]
            assert found[0]["parameters"] == [f"i{w}"] and found[0]["result"] == f"i{w}"
            markers[marker] = found[0]["id"]
    for imported in c[root]["imports"]:
        d = functions[imported["id"]]
        assert imported["owner"] == leaf and imported["header"] == c[leaf]["header"]
        assert imported["parameters"] == d["parameters"] and imported["return"] == d["result"]
        assert imported["symbol"] == native[d["id"]]["symbol"]
    return root, leaf, java, c, bindings, functions, native, markers
