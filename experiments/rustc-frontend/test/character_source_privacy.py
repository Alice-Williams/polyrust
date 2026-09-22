"""Compile real external consumers: public scalar API works, private helpers do not."""
from constant_export_native import java_path
from finite_constant_source_consumers import run


def verify(c_dir, j_dir, evidence, tools):
    _, owners, c, _, bindings, cf, jf = evidence
    leaf = owners["leaf"]
    public = bindings[leaf, "value", "identity"]
    private, = [row["id"] for row in c[leaf]["source_types"]["functions"]
                if row["parameters"] == ["i32"] and row["result"] == "i32"]
    assert not jf[private]["externally_reachable"]
    for identity, accepted in [(public, True), (private, False)]:
        probe = c_dir / "privacy.c"
        probe.write_text('#include "' + c[leaf]["header"] + '"\nint main(void) { (void)'
                         + cf[identity]["symbol"] + '(0); return 0; }\n')
        run(["gcc-14", "-std=c17", "-Wall", "-Wextra", "-Wpedantic", "-Werror",
             "-I", c_dir, "-fsyntax-only", probe], accepted=accepted)
        probe = j_dir / "PrivacyProbe.java"
        probe.write_text("public final class PrivacyProbe { public static void main(String[] args) { "
                         + java_path(jf[identity]["target"]["path"]) + "(0); } }\n")
        run([tools / "javac", "--release", "21", "-Xlint:all", "-Werror",
             "-implicit:none", "-sourcepath", "", "-cp", j_dir / "classes",
             "-d", j_dir / "classes", probe], accepted=accepted)
