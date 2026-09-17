"""Export actual unmodified compiler-generated artifacts, source and consumers."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "floating-truncation"
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
    for filename in ["truncation_leaf.rs", "truncation_middle.rs", "truncation_root.rs", "reference_truncation.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        "# Checked Rust binary64 truncation\n\n"
        "Actual unmodified three-crate generated C and Java packages, handwritten clients "
        "and original Rust inputs. No generated static runtime or helper package.\n\n"
        "Proof target: //experiments/rustc-frontend:truncation_native_test in the Linux dev container. "
        "Owners and consumers compile separately with GCC/Zig O0/O2 and Java21 strict lint. "
        "Exact binary64 truncation matches Rust and an independent integer-bit oracle. "
        "Test-only copies detect lost-zero-sign, wrong-floor, wrong-ceil, dropped/duplicate receivers and replaced/duplicate ordinary trunc calls.\n\n"
        "Scope: standard inherent f64::trunc method and associated forms. This example adds "
        "no arithmetic, float constants, casts or additional inspection mappings. Earlier "
        "documented capabilities are unchanged. "
        "NaN payload/sign preservation is not promised.\n\n"
        "C system-library requirements are derived from each manifest; use its closed "
        "system_libraries inventory when linking (m maps to -lm).\n")
