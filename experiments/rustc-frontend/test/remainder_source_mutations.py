"""Compiling faults affect only disposable copies of generated source."""
import re
from short_circuit_mutations import definition
from arithmetic_source_mutations import mutate as trace_mutation


def mutate(text, member, left, right, java, variant):
    if variant in ("dropped", "duplicated", "reversed"):
        return trace_mutation(text, member, left, right, java, variant)
    match, start, end = definition(text, member)
    parameters = [re.search(r"\b(\w+)\s*$", value).group(1) for value in match.group(1).split(",")]
    a, b = parameters
    body = text[start:end]
    returns = list(re.finditer(r"return\s+([^;]+);", body))
    assert len(returns) == 1, (variant, body)
    old = returns[0]
    nearest = "java.lang.Math.IEEEremainder" if java else "remainder"
    replacement = {
        "nearest": f"{nearest}({a}, {b})",
        "swapped": f"{b} % {a}" if java else f"fmod({b}, {a})",
        "zero_sign": f"({old.group(1)}) == 0.0 ? 0.0 : ({old.group(1)})",
    }[variant]
    body = body[:old.start()] + "return " + replacement + ";" + body[old.end():]
    if not java:
        locals_ = re.findall(r"\bdouble\s+(\w+)\s*=", body)
        position = body.index("return ")
        body = body[:position] + "".join(f"(void){local};\n" for local in locals_) + body[position:]
    return text[:start] + body + text[end:]
