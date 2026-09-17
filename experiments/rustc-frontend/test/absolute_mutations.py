"""Test-only scalar body faults; production rendering is not involved."""
import re
from short_circuit_mutations import definition

def mutate(text, member, marker, java, variant):
    match, start, end = definition(text, member)
    parameter = re.search(r"\b(\w+)\s*$", match.group(1)).group(1)
    p = parameter
    if variant == "missing_zero":
        body = f"\nreturn {p} < 0.0 ? -{p} : {p};\n"
    elif variant == "wrong_condition":
        body = f"\nreturn {p} == 0.0 ? 0.0 : ({p} > 0.0 ? -{p} : {p});\n"
    elif variant == "wrong_sign":
        body = f"\nreturn -{p};\n"
    elif variant == "dropped":
        body = f"\nreturn {p} == 0.0 ? 0.0 : ({p} < 0.0 ? -{p} : {p});\n"
    else:
        assert variant == "duplicated"
        prefix = "" if java else "(void)"
        body = f"\n{prefix}{marker}({p});\n" + text[start:end]
    return text[:start] + body + text[end:]
