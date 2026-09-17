"""Fixed scalar test-body faults, never production rewriting."""
import re
from short_circuit_mutations import definition


def mutate(text, member, marker, java, variant):
    match, start, end = definition(text, member)
    parameter = re.search(r"\b(\w+)\s*$", match.group(1)).group(1)
    if variant == "no_negation":
        body = f"\nreturn {parameter};\n"
    elif variant == "zero_minus":
        body = f"\nreturn 0.0 - {parameter};\n"
    elif variant == "dropped":
        body = f"\nreturn -{parameter};\n"
    else:
        assert variant == "duplicated"
        prefix = "" if java else "(void)"
        body = f"\n{prefix}{marker}({parameter});\n" + text[start:end]
    return text[:start] + body + text[end:]
