"""Publish actual, uninstrumented generated packages as Bazel test artifacts."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "wrapping-negation"
    destination.mkdir()
    writable_copy(java, destination / "java")
    writable_copy(c, destination / "c")
    clients = destination / "clients"
    clients.mkdir()
    for parent, filename in [("java-plain", "Consumer.java"), ("c-plain", "consumer.c")]:
        shutil.copy2(work / parent / filename, clients / filename)
    sources = destination / "rust-source"
    sources.mkdir()
    fixtures = Path(__file__).resolve().parent.parent / "fixtures"
    for filename in ["wrapping_leaf.rs", "wrapping_root.rs", "reference_wrapping.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust wrapping negation\n\n"
        "Actual unmodified two-crate generated C and Java packages, with handwritten clients "
        "and original Rust inputs. No static runtime or support package is generated.\n\n"
        "Proof: //experiments/rustc-frontend:wrapping_native_test in the Linux dev container. "
        "Each owner is compiled separately with GCC/Zig O0/O2, GCC UBSan, and Java 21 strict lint. "
        "Boundary/random results agree with original Rust and an independent modular integer oracle. "
        "Temporary instrumented copies separately prove exactly-once receivers, including "
        "value-preserving dropped/duplicated-call mutants.\n")
