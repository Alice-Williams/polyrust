"""Strict separate Java21 compilation and independent signed-widening truth."""
from pathlib import Path
import re
import subprocess
import sys

ROOT, JAVAC, JAVA, ORACLE = [Path(arg).resolve() for arg in sys.argv[1:]]
sys.path.insert(0, str(ORACLE))
from widening_oracle import CASES, result, faulty, inputs


def run(command, data=None):
    output = subprocess.run([str(arg) for arg in command], input=data,
                            capture_output=True, text=True, timeout=90)
    assert output.returncode == 0, (command, output.stdout, output.stderr)
    return output


def compile_source(source, classes):
    run([JAVAC, "--release", "21", "-encoding", "UTF-8", "-Xlint:all", "-Werror",
         "-implicit:none", "-sourcepath", "", "-cp", classes, "-d", classes, source])


def mutate(text, variant):
    original = re.findall(r"\breturn\s+([^;]+);", text)
    assert len(original) == 1
    cast = re.fullmatch(r"\(\(long\) (.*)\)", original[0])
    assert cast is not None, original
    operand = cast[1]
    assert operand == "operand" or operand.endswith(".widen(input)"), operand
    if variant == "zero_extend":
        replacement = f"((long)({operand})) & 0xffffffffL"
    elif variant == "narrow":
        replacement = f"(long)(short)({operand})"
    else:
        assert variant == "zero"
        replacement = f"((long)({operand})) & 0L"
    changed, count = re.subn(r"\breturn\s+[^;]+;", "return " + replacement + ";", text)
    assert count == 1 and changed != text
    return changed


CLIENT = """public final class Consumer {
private Consumer() {}
public static void main(String[] args) throws java.io.IOException {
    java.io.BufferedReader reader = new java.io.BufferedReader(
        new java.io.InputStreamReader(System.in, java.nio.charset.StandardCharsets.UTF_8));
    String line;
    while ((line = reader.readLine()) != null) {
        int input = Integer.parseInt(line);
""" + "".join(f"System.out.println(org.polyrust.generated.r{owner:016x}.Generated.widen(input));\n"
              for owner in [812, 813]) + "} } }\n"


def expected(variant):
    return "".join(f"{result(value) if variant == 'valid' else faulty(value, variant)}\n"
                   for value in CASES for _ in range(2))


def main():
    assert run([JAVAC, "-version"]).stdout.startswith("javac 21.")
    originals = []
    for index in range(3):
        sources = list((ROOT / f"owner{index}").rglob("*.java"))
        assert len(sources) == 1
        originals.append((sources[0], sources[0].read_bytes()))
    truth, data = expected("valid"), inputs()
    for variant in ["valid", "zero_extend", "narrow", "zero"]:
        directory, classes = ROOT / variant, ROOT / variant / "classes"
        classes.mkdir(parents=True)
        for index, (original, content) in enumerate(originals):
            text = content.decode()
            assert "Runtime" not in text and "Math." not in text and "import " not in text
            if index == 1 and variant != "valid":
                text = mutate(text, variant)
            source = directory / f"owner{index}" / original.relative_to(ROOT / f"owner{index}")
            source.parent.mkdir(parents=True)
            source.write_text(text)
            compile_source(source, classes)
        client = directory / "Consumer.java"
        client.write_text(CLIENT)
        compile_source(client, classes)
        wanted = expected(variant)
        assert (wanted == truth) == (variant == "valid")
        for options in [[], ["-Xint"]]:
            output = run([JAVA, *options, "-cp", classes, "Consumer"], data)
            assert output.stdout == wanted, (variant, options, "exact signed observations")
            assert output.stderr == "", (variant, options, "runtime diagnostics")
    assert all(path.read_bytes() == content for path, content in originals)
    print(f"{len(CASES)} inputs, both owners; strict separate Java21 compilation; "
          "normal/interpreted JVM; three compiling fault models killed; originals unchanged")


if __name__ == "__main__":
    main()
