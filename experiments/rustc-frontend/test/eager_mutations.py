"""Closed fixture faults, never a production code rewriter."""
import re
from short_circuit_mutations import definition


def mutate(text, entry, markers, java, fault, operator):
    match, start, end = definition(text, entry)
    params = [re.search(r"\b(\w+)\s*$", p).group(1) for p in match.group(1).split(",")]
    assert len(params) == 3
    left, right = f'{markers["A"]}({params[0]})', f'{markers["B"]}({params[1]})'
    unused = "" if java else f"(void){params[2]};"
    ty = "boolean" if java else "_Bool"
    if fault == "lazy":
        assert operator in ["&", "|"]
        body = f"{unused}\nreturn {left} {operator * 2} {right};"
    else:
        first, second = [f"{ty} l={left};", f"{ty} r={right};"]
        if fault == "reordered":
            first, second = second, first
        duplicate = (("" if java else "(void)") + right + ";") if fault == "duplicate" else ""
        if fault == "wrong":
            operator = {"&": "|", "|": "^", "^": "&"}[operator]
        body = f"{unused}\n{first}\n{second}\n{duplicate}\nreturn l {operator} r;"
    return text[:start] + "\n" + body + "\n" + text[end:]
