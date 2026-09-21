"""Export actual packages, source and external baseline clients, without mutation."""
import os
from pathlib import Path
import shutil
from constant_export_scratch import writable_copy


def export(java, c, source, work, count):
    destination = Path(os.environ["TEST_UNDECLARED_OUTPUTS_DIR"]) / "negative-zero"
    destination.mkdir()
    writable_copy(java, destination / "java")
    writable_copy(c, destination / "c")
    shutil.copy2(source, destination / "lib.rs")
    clients = destination / "clients"
    clients.mkdir()
    for parent, filename in [("java-plain", "Consumer.java"), ("c-plain", "consumer.c")]:
        shutil.copy2(work / parent / filename, clients / filename)
    (destination / "README.md").write_text(
        "# Runtime-free negative-zero composition\n\n"
        "Actual Rust source and generated C/Java packages. No runtime or math-library dependency.\n\n"
        f"{count:,} raw-bit cases per run agree across native Rust, vendored Apache-2.0 stdlib "
        "0.2.3, an independent bit oracle, GCC14/Zig O0/O2 and Java21 strict lint. "
        "Measured per-case Rust/target traces prove lazy reciprocal evaluation. "
        "Compiling value/trace faults and external private-access controls are included.\n\n"
        "Proof target: //experiments/rustc-frontend:negative_zero_native_test.\n")
