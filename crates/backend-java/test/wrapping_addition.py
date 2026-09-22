"""Strict separate Java21 compilation, modular truth and executable fault controls."""
from pathlib import Path
import re
import subprocess
import sys

root, javac, java, oracle = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(oracle))
from wrapping_add_oracle import CASES, faulty, inputs, result


def run(command, data=None):
    output = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def compile_source(source, classes):
    run([javac, "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes, source])


def mutate(text, variant):
    pattern = r"(public static (int|long) addition[01]\(final (?:int|long) left, final (?:int|long) right\)\s*\{)\s*return ([^;]+);\s*\}"
    count = 0

    def replace(match):
        nonlocal count
        count += 1
        ty = match.group(2)
        # Assert the real structural renderer produced the intended dataflow.
        assert re.sub(r"[()\s]", "", match.group(3)) == "left+right", match.group(0)
        if variant == "plain":
            return match.group(0)
        expression = {"carryless": "left ^ right", "subtract": "left - right",
                      "narrow": f"({'short' if ty == 'int' else 'int'})(left + right)",
                      "wrong_operand": "left + left"}.get(variant)
        if variant == "saturating":
            boxed = "Integer" if ty == "int" else "Long"
            expression = ("java.math.BigInteger.valueOf(left).add(java.math.BigInteger.valueOf(right))"
                          f".max(java.math.BigInteger.valueOf({boxed}.MIN_VALUE))"
                          f".min(java.math.BigInteger.valueOf({boxed}.MAX_VALUE)).{ty}Value()")
        assert expression is not None, variant
        return match.group(1) + f" return {expression}; }}"

    changed = re.sub(pattern, replace, text)
    assert count == 2, (count, text)
    return changed


CLIENT = """public final class Consumer {
private Consumer() {}
public static void main(String[] args) throws java.io.IOException {
    java.io.BufferedReader reader = new java.io.BufferedReader(
        new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        String[] parts = line.split(" ");
        if (parts[0].equals("32")) {
            int left = Integer.parseInt(parts[1]);
            int right = Integer.parseInt(parts[2]);
""" + "".join(f"System.out.println(org.polyrust.generated.r{owner:016x}.Generated.addition0(left, right));\n"
              for owner in [801, 802]) + """} else {
            long left = Long.parseLong(parts[1]);
            long right = Long.parseLong(parts[2]);
""" + "".join(f"System.out.println(org.polyrust.generated.r{owner:016x}.Generated.addition1(left, right));\n"
              for owner in [801, 802]) + "} } } }\n"


def main():
    assert run([javac, "-version"]).stdout.startswith("javac 21.")
    truth = "".join(f"{result(left, right, width)}\n" for width, left, right in CASES for _ in range(2))
    for variant in ["plain", "saturating", "carryless", "subtract", "narrow", "wrong_operand"]:
        directory = root / variant
        classes = directory / "classes"
        classes.mkdir(parents=True)
        for index in range(2):
            originals = list((root / f"owner{index}").rglob("*.java"))
            assert len(originals) == 1
            original = originals[0]
            text = original.read_text()
            assert "Runtime" not in text and "Math." not in text
            if index == 0:
                text = mutate(text, variant)
            path = directory / f"owner{index}" / original.relative_to(root / f"owner{index}")
            path.parent.mkdir(parents=True)
            path.write_text(text)
            compile_source(path, classes)
        client = directory / "Consumer.java"
        client.write_text(CLIENT)
        compile_source(client, classes)
        wanted = truth
        if variant != "plain":
            wanted = "".join(f"{result(left, left, width) if variant == 'wrong_operand' else faulty(left, right, width, variant)}\n"
                             for width, left, right in CASES for _ in range(2))
            assert wanted != truth, variant
        output = run([java, "-cp", classes, "Consumer"], inputs())
        assert output.stdout == wanted and output.stderr == "", (variant, "exact modular results")
    print(f"{len(CASES) * 2} Java21 producer/consumer results per run; five compiled value-fault controls")


if __name__ == "__main__":
    main()
