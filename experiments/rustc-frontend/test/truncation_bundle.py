"""Closed serialized library identities, never user-supplied linker options."""
import json

def link_options(api):
    assert api["schema_version"] == 9
    assert api["system_libraries"] == ["m"]
    flags = {"m": "-lm"}
    return [flags[library] for library in api["system_libraries"]]

def bundle(directory, java):
    index = json.loads((directory / "bundle.json").read_text())
    assert index["schema_version"] == 1 and len(index["members"]) == 3
    owners, files = {}, {"bundle.json"}
    for member in index["members"]:
        api = json.loads((directory / member["manifest"]).read_text())
        assert api["root"] == member["root"] and api["root"] not in owners
        assert api["schema_version"] == (6 if java else 9)
        if not java:
            link_options(api)
        owners[api["root"]] = api
        files.add(member["manifest"])
        files.update([api["source"]] if java else [api["header"], api["implementation"]])
    assert files == {str(p.relative_to(directory)) for p in directory.rglob("*") if p.is_file()}
    assert len(files) == (7 if java else 10)
    return index["root"], owners

def malformed_controls(api):
    for value in [None, [], ["unknown"], ["m", "m"], ["m", "-Wl,unexpected"]]:
        changed = dict(api)
        if value is None:
            del changed["system_libraries"]
        else:
            changed["system_libraries"] = value
        try:
            link_options(changed)
        except (KeyError, AssertionError):
            continue
        raise AssertionError("malformed link inventory accepted")
