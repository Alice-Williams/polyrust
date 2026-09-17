"""Independent trace truth; instrumentation changes only private test copies."""
import re

VALUES = [-2147483648, -1, 0, 1, 2147483647]

def expected():
    truth, traces = [], []
    for value in VALUES:
        for flag in [False, True]:
            end = "E|" if flag else "U|"
            relay = f"A:{value}|B:{value}|O:{value}:{~value}:{int(flag)}|" + end + f"P:{int(flag)}|"
            if flag:
                relay += "E|U|"
            execute = relay + f"O:{value}:{value}:1|E|" + end
            tail = f"A:{value}|B:{value}|O:{value}:{~value}:1|E|P:1|E|U|"
            truth.append(str(value))
            traces.append(execute + tail + end)
    return "\n".join(truth) + "\n", "\n".join(traces) + "\n"

def instrument(text, calls, java):
    for name, marker in [("first", "A"), ("second", "B"), ("observe", "O"), ("predicate", "P"), ("empty", "E"), ("explicit", "U")]:
        symbol = calls[name][0].split(".")[-1] if java else calls[name][1]
        pattern = re.compile(r"\b" + re.escape(symbol) + r"\(([^)]*)\)\s*\{")
        matches = list(pattern.finditer(text))
        assert len(matches) == 1, (symbol, text)
        match = matches[0]
        arguments = [part.strip().split()[-1] for part in match[1].split(",")] if name not in ("empty", "explicit") else []
        if java:
            parts = [f'"{marker}"']
            for index, argument in enumerate(arguments):
                value = f"({argument} ? 1 : 0)" if (index == 2 or name == "predicate") else argument
                parts += ['":"', value]
            statement = "System.err.print(" + " + ".join(parts + ['"|"']) + ");"
        else:
            formats = marker + "".join(":%d" if (index == 2 or name == "predicate") else ':%" PRId32 "' for index in range(len(arguments))) + "|"
            values = [f"(int){arg}" if (index == 2 or name == "predicate") else arg for index, arg in enumerate(arguments)]
            statement = '(void)fprintf(stderr, "' + formats + '"' + (", " + ", ".join(values) if values else "") + ");"
        text = text[:match.end()] + "\n" + statement + text[match.end():]
    if not java:
        text = "#include <inttypes.h>\n#include <stdio.h>\n" + text
    return text
