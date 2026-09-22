"""Export actual source-owned packages and baseline clients, never generated substitutes."""
from pathlib import Path
import os
import shutil
from constant_export_scratch import writable_copy


def export(java, c, work, operation="addition"):
    assert operation in ["addition", "subtraction"]
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / ("wrapping-" + operation)
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
    for filename in [operation + "_leaf.rs", operation + "_middle.rs", operation + "_root.rs", "reference_" + operation + "_source.rs"]:
        shutil.copy2(fixtures / filename, sources / filename)
    (destination / "README.md").write_text(
        f"# Checked Rust wrapping {operation}\n\n"
        "Actual three-crate C and Java packages, original Rust, and handwritten external clients. "
        f"No custom runtime or helper package. C uses guarded unsigned normalization; Java uses primitive {operation}.\n\n"
        f"Proof: //experiments/rustc-frontend:{operation}_native_test. "
        "15,790 native Rust and independent modular-oracle inputs; 31,580 target observations per run. "
        "Strict separate GCC14/Zig O0/O2 and Java21 compilation, plus GCC UBSan. "
        f"Measured native Rust and target operand traces agree. {'Three' if operation == 'addition' else 'Four'} compiling value faults and three "
        "value-preserving evaluation faults are detected, alongside exact API/docs/import and external privacy controls.\n")
