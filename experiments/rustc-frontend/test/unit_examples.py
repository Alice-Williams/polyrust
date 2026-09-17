"""Inspectable Bazel test artifacts, not committed generated source."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy

def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "unit-results"
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
    for filename in ["unit_leaf.rs", "unit_relay.rs", "unit_root.rs", "reference_unit.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust unit results\n\n"
        "These are actual unmodified generated three-crate packages and handwritten clients.\n"
        "Rust sources are included for inspection. C and Java metadata identify public names,\n"
        "original crate identities, private declarations and scalar versus unit results.\n\n"
        "Proof: //experiments/rustc-frontend:unit_native_test in the Linux dev container.\n"
        "The gate separately compiles every owner with GCC/Zig O0/O2 and Java 21 strict lint,\n"
        "compares scalar continuation against Rust, and instruments separate temporary copies\n"
        "to prove call order and catch dropped, duplicated and reordered effects.\n"
        "The packages here are the uninstrumented originals: no runtime or unit wrapper.\n")
    assert (clients / "consumer.c").is_file() and (clients / "Consumer.java").is_file()
    assert len(list(sources.iterdir())) == 4
