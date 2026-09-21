"""Exact Java21 remainder and independently expected producer-call traces."""
from pathlib import Path
import re
import subprocess
import sys

root, javac, java, oracle = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(oracle))
from arithmetic_oracle import observed
from remainder_oracle import result
from remainder_faults import changed
from remainder_cases import PAIRS
from short_circuit_mutations import instrument, definition


def run(command, inputs=None):
    output = subprocess.run([str(arg) for arg in command], input=inputs,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def mutate(text, variant):
    _, start, end = definition(text, "remainder0")
    body = text[start:end]
    declarations = []
    for side in ["left", "right"]:
        pattern = r"final double\s+(\w+)\s*=\s*((?:\w+\.)*" + side + r"Identity\((\w+)\));"
        matches = list(re.finditer(pattern, body))
        assert len(matches) == 1, (variant, side, body)
        declarations.append(matches[0])
    a, b = declarations
    left, right = a.group(1), b.group(1)
    if variant == "dropped":
        body = body[:a.start()] + f"final double {left} = {a.group(3)};" + body[a.end():]
    elif variant == "duplicated":
        body = "\n" + a.group(2) + ";\n" + body
    elif variant == "reversed":
        assert a.end() < b.start()
        body = body[:a.start()] + b.group(0) + body[a.end():b.start()] + a.group(0) + body[b.end():]
    else:
        old = re.search(r"return\s+(.*);", body)
        assert old is not None
        replacement = {
            "nearest": f"java.lang.Math.IEEEremainder({left}, {right})",
            "swapped": f"{right} % {left}",
            "zero_sign": f"({old.group(1)}) == 0.0 ? 0.0 : ({old.group(1)})",
        }[variant]
        body = body[:old.start()] + "return " + replacement + ";" + body[old.end():]
    return text[:start] + body + text[end:]


CONSUMER = """public final class Consumer {
private Consumer() {}
private static void observe(double value) {
    if (Double.isNaN(value)) { System.out.println("nan"); }
    else { System.out.printf(java.util.Locale.ROOT, "%016x%n", Double.doubleToRawLongBits(value)); }
}
public static void main(String[] args) throws java.io.IOException {
    java.io.BufferedReader reader = new java.io.BufferedReader(
        new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        String[] parts = line.split(" ");
        double left = Double.longBitsToDouble(Long.parseUnsignedLong(parts[0], 16));
        double right = Double.longBitsToDouble(Long.parseUnsignedLong(parts[1], 16));
""" + "".join(f"observe(org.polyrust.generated.r{owner:016x}.Generated.remainder0(left, right));\n"
              for owner in [602, 603]) + "} } }\n"


def compile_source(source, classes):
    run([javac, "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes, source])


def main():
    assert run([javac, "-version"]).stdout.startswith("javac 21.")
    truth = "\n".join(observed(result(a, b)) for a, b in PAIRS for _ in range(2)) + "\n"
    inputs = "".join(f"{a:016x} {b:016x}\n" for a, b in PAIRS)
    for variant in ["plain", "traced", "nearest", "swapped", "zero_sign", "dropped", "duplicated", "reversed"]:
        directory = root / variant
        classes = directory / "classes"
        classes.mkdir(parents=True)
        for index in range(3):
            originals = list((root / f"owner{index}").rglob("*.java"))
            assert len(originals) == 1
            original = originals[0]
            text = original.read_text()
            assert "Math." not in text and "Runtime" not in text
            if variant != "plain" and index == 0:
                text = instrument(text, {"A": "leftIdentity", "B": "rightIdentity"}, True)
            if variant not in ("plain", "traced") and index == 1:
                text = mutate(text, variant)
            path = directory / f"owner{index}" / original.relative_to(root / f"owner{index}")
            path.parent.mkdir(parents=True)
            path.write_text(text)
            compile_source(path, classes)
        client = directory / "Consumer.java"
        client.write_text(CONSUMER)
        compile_source(client, classes)
        wanted = truth
        if variant in ("nearest", "swapped", "zero_sign"):
            wanted = "\n".join(observed(changed(a, b, variant)) for a, b in PAIRS for _ in range(2)) + "\n"
            assert wanted != truth, variant
        trace = "" if variant == "plain" else {"dropped": "B", "duplicated": "AAB", "reversed": "BA"}.get(variant, "AB") * 2 * len(PAIRS)
        if variant in ("dropped", "duplicated", "reversed"):
            assert wanted == truth and trace != "AB" * 2 * len(PAIRS)
        output = run([java, "-cp", classes, "Consumer"], inputs)
        assert output.stdout == wanted, (variant, "exact remainder values")
        assert output.stderr == trace, (variant, "original operand evaluation")
    print(f"{len(PAIRS) * 2} exact Java21 results/run; separate producer/importer/client compilation; "
          "three value faults and three value-preserving trace faults")


if __name__ == "__main__":
    main()
