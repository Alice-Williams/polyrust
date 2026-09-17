"""Value-preserving trace faults and exact-literal faults on test-only copies."""
import re
from short_circuit_mutations import definition


def fault_comparison(text, member, markers, operator, fault):
    match, start, end = definition(text, member)
    arguments = [re.search(r"\b(\w+)\s*$", arg)[1] for arg in match[1].split(",")]
    assert len(arguments) == 2
    left = f'double left = {markers["L"]}({arguments[0]});'
    right = f'double right = {markers["R"]}({arguments[1]});'
    if fault == "reordered":
        body = right + left
    else:
        assert fault == "dropped"
        body = f'double left = {arguments[0]};' + right
    return text[:start] + "\n" + body + f"return left {operator} right;\n" + text[end:]


def fault_literal(text, member):
    _, start, end = definition(text, member)
    # No signature changes: strict native compilation still succeeds.
    return text[:start] + "\nreturn 0.0;\n" + text[end:]
