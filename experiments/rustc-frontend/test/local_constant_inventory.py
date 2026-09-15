"""Exact two-crate API inventory; no runtime files or lost public functions."""
from i64_inventory import bundle
from java_fixture_native import exports, check_inventory_oracle
from local_constant_oracle import CASES


def inspect(java_dir, c_dir):
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    leaf, = set(java) - {root}
    assert java[root]["dependencies"] == [leaf] and java[leaf]["dependencies"] == []
    assert not c[leaf]["imports"] and len(c[root]["imports"]) == 1
    bindings = {}
    for owner in java:
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
    check_inventory_oracle(bindings, {(root, "value", name) for name, *_ in CASES} |
                           {(leaf, "value", "identity")})
    functions = {d["id"]: d for owner in java.values() for d in owner["declarations"] if d["kind"] == "function"}
    native = {d["id"]: d for owner in c.values() for d in owner["functions"]}
    assert len(functions) == len(native) == 16 and set(functions) == set(native)
    for owner, name, parameters, result in [
        *((root, name, ["bool"], ty) for name, ty, *_ in CASES),
        (leaf, "identity", ["i64", "bool"], "i64"),
    ]:
        kind, identity = bindings[owner, "value", name]
        assert kind == "declaration"
        d = functions[identity]
        assert d["parameters"] == parameters and d["result"] == result
        assert d["externally_reachable"] and native[identity]["linkage"] == "external"
    imported, = c[root]["imports"]
    assert imported["id"] == bindings[leaf, "value", "identity"][1]
    assert imported["owner"] == leaf and imported["header"] == c[leaf]["header"]
    assert imported["parameters"] == ["i64", "bool"] and imported["return"] == "i64"
    assert imported["symbol"] == native[imported["id"]]["symbol"]
    return root, leaf, java, c, bindings, functions, native
