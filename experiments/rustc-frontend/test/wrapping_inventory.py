"""Independent public/private, signature and original-owner expectations."""
from i64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from wrapping_oracle import NAMES


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    leaf, = set(java) - {root}
    assert java[root]["dependencies"] == [leaf] and java[leaf]["dependencies"] == []
    assert not c[leaf]["imports"] and len(c[root]["imports"]) == 4
    bindings = {}
    for owner in java:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    keys = {(root, "value", name + str(w)) for w in [32, 64] for name in NAMES}
    keys.update((leaf, "value", name + str(w)) for w in [32, 64] for name in ["identity", "negate"])
    check_inventory_oracle(bindings, keys)
    functions = {d["id"]: d for owner in java.values() for d in owner["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for owner in c.values() for d in owner["functions"]}
    assert len(functions) == len(native) == 20 and set(functions) == set(native)
    for owner, names in [(root, NAMES), (leaf, ["identity", "negate"])]:
        for width in [32, 64]:
            for name in names:
                kind, identity = bindings[owner, "value", name + str(width)]
                assert kind == "declaration"
                d = functions[identity]
                assert d["parameters"] == [f"i{width}"] and d["result"] == f"i{width}"
                assert d["externally_reachable"] and native[identity]["linkage"] == "external"
    markers = {}
    for width in [32, 64]:
        found = [d for d in functions.values() if [s.strip() for s in d["documentation"]] == [f"Trace W{width}."]]
        assert len(found) == 1
        d = found[0]
        assert not d["externally_reachable"] and native[d["id"]]["linkage"] == "internal"
        assert d["parameters"] == [f"i{width}"] and d["result"] == f"i{width}"
        markers[width] = d["id"]
    for imported in c[root]["imports"]:
        d = functions[imported["id"]]
        assert imported["owner"] == leaf and imported["header"] == c[leaf]["header"]
        assert imported["parameters"] == d["parameters"] and imported["return"] == d["result"]
        assert imported["symbol"] == native[d["id"]]["symbol"]
    return root, leaf, java, c, bindings, functions, native, markers
