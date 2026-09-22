"""Exact original declarations, exports, dependencies, docs and package payloads."""
from collections import Counter
from copy import deepcopy
import json
import re
from java_fixture_native import exports, check_inventory_oracle


def reject(check):
    try:
        check()
    except (AssertionError, KeyError):
        return
    raise AssertionError("inventory mutation escaped")


def bundle(directory, java):
    index = json.loads((directory / "bundle.json").read_text())
    assert index["schema_version"] == 1 and len(index["members"]) == 3
    owners, files = {}, {"bundle.json"}
    for member in index["members"]:
        api = json.loads((directory / member["manifest"]).read_text())
        assert api["root"] == member["root"] and api["root"] not in owners
        assert api["schema_version"] == (1 if java else 2)
        assert not api.get("system_libraries", [])
        owners[api["root"]] = api
        files.add(member["manifest"])
        files.update([api["source"]] if java else [api["header"], api["implementation"]])
    assert files == {str(p.relative_to(directory)) for p in directory.rglob("*") if p.is_file()}
    assert len(files) == (7 if java else 10)
    return index["root"], owners


def inspect(java_dir, c_dir, operation="addition"):
    assert operation in ["addition", "subtraction"]
    root, java = bundle(java_dir, True)
    c_root, c = bundle(c_dir, False)
    assert root == c_root and set(java) == set(c)
    middle, = java[root]["dependencies"]
    leaf, = java[middle]["dependencies"]
    assert java[leaf]["dependencies"] == []
    order, bindings = [leaf, middle, root], {}
    for owner in order:
        assert {d["owner"] for d in c[owner]["imports"]} == set(java[owner]["dependencies"])
        found = exports(java[owner], True)
        assert found == exports(c[owner], False) and not set(found) & set(bindings)
        bindings.update(found)
        for path in [c_dir / c[owner]["implementation"], java_dir / java[owner]["source"]]:
            text = path.read_text()
            assert "runtime" not in text.lower() and "Math." not in text and "goto " not in text
    kind, module = bindings[middle, "type", "operations"]
    assert kind == "module"
    wanted = {(leaf, "value", side + str(width)) for width in [32, 64] for side in ["left", "right"]}
    wanted.add((middle, "type", "operations"))
    wanted.update((owner, "value", operation + str(width)) for owner in [module, root] for width in [32, 64])
    check_inventory_oracle(bindings, wanted)
    records = [d for api in java.values() for d in api["declarations"]]
    c_records = [d for api in c.values() for d in api["functions"]]

    def unique(rows):
        assert len(rows) == len({d["id"] for d in rows}) == 10

    for rows in [records, c_records]:
        unique(rows)
        reject(lambda: unique([*rows, rows[0]]))
        reject(lambda: unique([rows[0], *rows[:-1]]))
    functions = {d["id"]: d for d in records}
    native = {d["id"]: d for d in c_records}
    assert set(functions) == set(native)
    public = {value[1] for value in bindings.values() if value[0] == "declaration"}
    private = set(functions) - public
    assert len(private) == 2
    signatures = {}
    for width in [32, 64]:
        for side in ["left", "right"]:
            signatures[bindings[leaf, "value", side + str(width)][1]] = ([f"i{width}"], f"i{width}")
        for owner in [module, root]:
            signatures[bindings[owner, "value", operation + str(width)][1]] = ([f"i{width}"] * 2, f"i{width}")
        hidden, = [identity for identity in private if functions[identity]["result"] == f"i{width}"]
        signatures[hidden] = ([f"i{width}"], f"i{width}")

    def declarations(actual, c_actual):
        for identity, (parameters, result) in signatures.items():
            d, n = actual[identity], c_actual[identity]
            assert d["kind"] == "function" and d["parameters"] == parameters
            assert d["result"] == result
            assert d["externally_reachable"] == (identity in public)
            assert n["linkage"] == ("external" if identity in public else "internal")

    declarations(functions, native)
    for identity in functions:
        for field, wrong in [("kind", "field"), ("result", "bool"), ("parameters", []),
                             ("externally_reachable", not functions[identity]["externally_reachable"])]:
            changed = deepcopy(functions)
            changed[identity][field] = wrong
            reject(lambda: declarations(changed, native))
    # Scalar schema 2 records ownership/linkage, not owned-function signatures.
    # Check actual header/implementation declarations independently instead.
    for owner in order:
        header = (c_dir / c[owner]["header"]).read_text()
        implementation = (c_dir / c[owner]["implementation"]).read_text()
        for declaration in c[owner]["functions"]:
            identity, symbol = declaration["id"], declaration["symbol"]
            parameters, result = signatures[identity]
            ty = lambda name: "int" + name[1:] + "_t"
            def signature(text, is_header):
                rows = re.findall(r"^(static )?(int32_t|int64_t) " + re.escape(symbol) + r"\(([^()]*)\)\s*[;{]", text, re.MULTILINE)
                count = (1 if identity in public else 0) if is_header else (1 if identity in public else 2)
                assert len(rows) == count
                for storage, actual_result, actual_parameters in rows:
                    assert bool(storage) == (identity in private)
                    assert actual_result == ty(result)
                    assert [value.strip().split()[0] for value in actual_parameters.split(",")] == [ty(value) for value in parameters]
            signature(header, True)
            signature(implementation, False)
            reject(lambda: signature(implementation.replace(ty(result) + " " + symbol, "_Bool " + symbol), False))
    for owner, dependency, names in [(middle, leaf, [s + str(w) for w in [32, 64] for s in ["left", "right"]]),
                                     (root, module, [operation + str(w) for w in [32, 64]])]:
        wanted_ids = Counter(bindings[dependency, "value", name][1] for name in names)
        def imports(rows):
            assert Counter(d["id"] for d in rows) == wanted_ids
            for d in rows:
                assert d["parameters"] == functions[d["id"]]["parameters"] and d["return"] == functions[d["id"]]["result"]
                assert d["symbol"] == native[d["id"]]["symbol"] and d["header"] == c[d["owner"]]["header"]
        rows = c[owner]["imports"]
        imports(rows)
        reject(lambda: imports(rows[1:]))
        reject(lambda: imports([*rows, rows[0]]))
    docs = {leaf: ["Original signed operand producers and private implementation details."],
            middle: [f"Checked wrapping {operation} in its original public module."],
            root: ["Public forwarding without copying dependency implementations."],
            module: [f"{operation.capitalize()} operations preserve source module identity."]}
    docs.update({identity: [] for identity in private})
    for width in [32, 64]:
        for side in ["left", "right"]:
            docs[bindings[leaf, "value", side + str(width)][1]] = [f"Original {side} signed {width}-bit operand."]
        docs[bindings[module, "value", operation + str(width)][1]] = [f"Exact signed {width}-bit wrapping {operation} with ordered operands."]
        docs[bindings[root, "value", operation + str(width)][1]] = []
    for owner in order:
        records = [*java[owner]["modules"], *java[owner]["declarations"]]
        def metadata(rows):
            assert {d["id"]: [line.strip() for line in d["documentation"]] for d in rows} == {d["id"]: docs[d["id"]] for d in records}
        metadata(records)
        for index in range(len(records)):
            changed = deepcopy(records)
            changed[index]["documentation"] = ["changed"]
            reject(lambda: metadata(changed))
        wanted_docs = Counter(line for d in records for line in docs[d["id"]])
        for path, pattern in [(java_dir / java[owner]["source"], r"^\s*\*\s+(.+)$"),
                              (c_dir / c[owner]["header"], r"^/\*\s+(.+?)\s+\*/$")]:
            def rendered(text):
                assert Counter(line.strip() for line in re.findall(pattern, text, re.MULTILINE)) == wanted_docs
            text = path.read_text()
            rendered(text)
            for document in wanted_docs:
                reject(lambda: rendered(text.replace(document, "changed")))
    return order, module, java, c, bindings, functions, native, private
