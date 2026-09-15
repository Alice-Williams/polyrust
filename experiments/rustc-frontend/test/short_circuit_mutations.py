"""Fixed scalar fixture instrumentation/faults, not a general source parser.

Only test copies are instrumented; production bundles remain untouched. The
trace functions are pure bool identities, so eager/duplicated/reordered calls
can preserve final values but must fail the separate native trace oracle.
"""
import re


def definition(text, member):
    pattern = r"\b" + re.escape(member) + r"\s*\(([^()]*)\)\s*\{"
    found = list(re.finditer(pattern, text))
    assert len(found) == 1, (member, "expected one fixture definition")
    match = found[0]
    start, depth, end = match.end(), 1, match.end()
    while depth:
        assert end < len(text)
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return match, start, end - 1


def instrument(text, markers, java):
    for marker, member in markers.items():
        _, start, _ = definition(text, member)
        trace = (f'java.lang.System.err.print("{marker}");' if java else
                 f'(void)fputc(\'{marker}\', stderr);')
        text = text[:start] + "\n    " + trace + text[start:]
    return text if java else "#include <stdio.h>\n" + text


def mutate(text, entry, markers, java, fault):
    match, start, end = definition(text, entry)
    parameters = [re.search(r"\b(\w+)\s*$", value).group(1)
                  for value in match.group(1).split(",")]
    assert len(parameters) == 3
    body = text[start:end]
    call_pattern = r"(?:\b[\w]+\.)*\b" + re.escape(markers["B"]) + r"\((\w+)\)"
    calls = list(re.finditer(call_pattern, body))
    assert len(calls) == 1
    call = calls[0]
    if fault == "eager":
        # Move the actual B call before the first condition; retain the former
        # argument temporary's use so strict C unused-local lint still passes.
        body = body[:call.start()] + call.group(1) + body[call.end():]
        prefix = "" if java else "(void)"
        body = f'\n    {prefix}{markers["B"]}({parameters[1]});\n' + body
    elif fault == "duplicate":
        line = body.rfind("\n", 0, call.start()) + 1
        prefix = "" if java else "(void)"
        body = body[:line] + f"    {prefix}{call.group(0)};\n" + body[line:]
    else:
        assert fault == "reordered"
        ty = "boolean" if java else "_Bool"
        unused = "" if java else f"(void){parameters[2]};"
        body = (f'\n    {unused}\n    {ty} right = {markers["B"]}({parameters[1]});\n'
                f'    {ty} left = {markers["A"]}({parameters[0]});\n    return left && right;\n')
    return text[:start] + body + text[end:]
