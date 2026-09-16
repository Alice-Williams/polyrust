"""Selected-entry is an explicit value-only projection, not a public API package."""
import re


def check(work, source, reference, c_adapter, c_probe, java_adapter, java_probe, jdk, zig, run):
    work.mkdir()
    truth = "0\n0\n62\n"
    assert run([reference]).stdout == truth
    c_source = work / "model.c"
    java_source = work / "Generated.java"
    for adapter, probe, destination in [(c_adapter, c_probe, c_source),
                                         (java_adapter, java_probe, java_source)]:
        run([adapter, source, destination])
        expected = destination.read_bytes()
        observed = run([probe, source, destination]).stdout
        assert destination.read_bytes() == expected
        assert "PUBLIC_CONSTANT_DECL" not in observed
        assert "PUBLIC_CONSTANT_READ" not in observed
    c_text, java_text = c_source.read_text(), java_source.read_text()
    assert "constant_" not in c_text and "extern const" not in c_text
    assert "public static final" not in java_text
    assert "62" in c_text and "62" in java_text

    # The selected-entry facade has a stable score member, but the crate's package
    # identifier is compiler-derived. Read only the package declaration here.
    packages = re.findall(r"^package ([a-zA-Z0-9_.]+);$", java_text, re.MULTILINE)
    assert len(packages) == 1
    consumer = work / "Consumer.java"
    consumer.write_text("public class Consumer { public static void main(String[] args) {\n"
                        "for (int value : new int[]{-1,0,1}) { System.out.println(" +
                        packages[0] + ".Generated.score(value)); } } }\n")
    classes = work / "classes"
    classes.mkdir()
    flags = ["--release", "21", "-Xlint:all", "-Werror", "-implicit:none",
             "-sourcepath", "", "-cp", classes, "-d", classes]
    run([jdk / "javac", *flags, java_source])
    run([jdk / "javac", *flags, consumer])
    assert run([jdk / "java", "-cp", classes, "Consumer"]).stdout == truth

    consumer = work / "consumer.c"
    consumer.write_text('#include <stdint.h>\n#include <stdio.h>\n'
                        'int32_t poly_score(int32_t value);\n'
                        'int main(void) { for (int32_t value = -1; value <= 1; ++value) {\n'
                        'printf("%d\\n", (int)poly_score(value)); } return 0; }\n')
    flags = ["-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-Wstrict-prototypes", "-Wmissing-prototypes"]
    for compiler in ["gcc-14", zig]:
        for opt in ["0", "2"]:
            objects = []
            for i, unit in enumerate([c_source, consumer]):
                obj = work / f"entry{i}.o"
                run([compiler, *flags, "-O" + opt, "-c", unit, "-o", obj])
                objects.append(obj)
            binary = work / "consumer"
            run([compiler, *objects, "-o", binary])
            assert run([binary]).stdout == truth
