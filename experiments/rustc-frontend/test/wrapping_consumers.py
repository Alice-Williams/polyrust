"""Handwritten fixed-fixture clients and value-preserving trace mutants."""
import re
from short_circuit_mutations import definition
from wrapping_oracle import NAMES


def consumer(calls, java, headers):
    branches = []
    for width in [32, 64]:
        expressions = [f"{calls[name + str(width)]}(value)" for name in NAMES]
        outputs = []
        for index, call in enumerate(expressions):
            separator = " " if index + 1 < len(NAMES) else "\\n"
            if java:
                outputs.append(f'System.out.print({call}); System.out.print("{separator}");')
            else:
                outputs.append(f'printf("%" PRId{width} "{separator}", {call});')
        binding = ("int value=(int)input;" if width == 32 else "long value=input;") if java else f"int{width}_t value=(int{width}_t)input;"
        branches.append(f'if(width=={width}) {{ {binding}\n' + "\n".join(outputs) + "\n}")
    body = " else ".join(branches)
    if java:
        return ('public final class Consumer { public static void main(String[] args) throws java.io.IOException {\n'
                'var reader=new java.io.BufferedReader(new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));\n'
                'String line; while((line=reader.readLine())!=null) { String[] parts=line.split(" ");\n'
                'int width=Integer.parseInt(parts[0]); long input=Long.parseLong(parts[1]);\n'
                + body + '\nSystem.err.println();\n} } }\n')
    return ('#include <inttypes.h>\n#include <stdio.h>\n' + headers
            + 'int main(void) { int width; int64_t input;\n'
            + 'while(scanf("%d %" SCNd64, &width,&input)==2) {\n'
            + body + "\n(void)fputc('\\n',stderr);\n}\nreturn 0;\n}\n")


def mutate(text, entry, receiver, java, fault):
    # This is intentionally a fixed-fixture call recognizer, not a target parser.
    _, start, end = definition(text, entry)
    body = text[start:end]
    pattern = r"(?:\b[\w]+\.)*\b" + re.escape(receiver) + r"\((\w+)\)"
    found = list(re.finditer(pattern, body))
    assert len(found) == 1
    call, = found
    if fault == "drop":
        body = body[:call.start()] + call.group(1) + body[call.end():]
    else:
        assert fault == "duplicate"
        line = body.rfind("\n", 0, call.start()) + 1
        prefix = "" if java else "(void)"
        body = body[:line] + f"    {prefix}{call.group(0)};\n" + body[line:]
    return text[:start] + body + text[end:]
