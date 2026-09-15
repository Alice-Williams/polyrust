"""Fixed-corpus order probes; neither parsing nor modifying production output."""
import re

from short_circuit_mutations import definition


def markers(functions):
    result = {}
    for marker in "LRC":
        found = [item for item in functions.values()
                 if [doc.strip() for doc in item["documentation"]] == [f"Trace {marker}."]]
        assert len(found) == 1 and not found[0]["externally_reachable"]
        ty = "bool" if marker == "C" else "i64"
        assert found[0]["parameters"] == [ty] and found[0]["result"] == ty
        result[marker] = found[0]["id"]
    return result


def reorder(text, member, names, java, operator):
    match, start, end = definition(text, member)
    parameters = [re.search(r"\b(\w+)\s*$", value).group(1)
                  for value in match.group(1).split(",")]
    assert len(parameters) == 3
    ty = "long" if java else "int64_t"
    unused = "" if java else f"(void){parameters[2]};"
    body = (f'\n    {unused}\n    {ty} right = {names["R"]}({parameters[1]});\n'
            f'    {ty} left = {names["L"]}({parameters[0]});\n'
            f'    return {names["C"]}(left {operator} right);\n')
    return text[:start] + body + text[end:]
