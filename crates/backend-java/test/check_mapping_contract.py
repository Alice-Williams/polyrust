"""Compile the actual Java mapping/plan declarations, with isolated base plumbing.

The declarations under test are extracted verbatim from production source.
Only unrelated generic mapping/dialect plumbing is stubbed. A positive control
must compile before any negative result is accepted.
"""
from pathlib import Path
import re
import subprocess
import sys


def declaration(path: Path, name: str) -> str:
    source = path.read_text(encoding="utf-8")
    match = re.search(rf"(?m)^pub trait {re.escape(name)}\b", source)
    assert match is not None, f"missing top-level production trait {name}"
    start = match.start()
    opening = source.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return source[start:end]


def main() -> None:
    compiler, support, plans, scratch = sys.argv[1:]
    contract = declaration(Path(support), "JavaCapabilityMapping")
    plan = declaration(Path(plans), "JavaMappingPlan")
    prefix = r"""
#![allow(dead_code, private_bounds, private_interfaces)]
mod sealed {
    pub trait JavaCapabilityMapping {}
    pub trait JavaMappingPlan {}
}
struct JavaDialect;
struct Diagnostic;
enum JavaRepresentation { Direct }
trait JavaMappingOutput {}
impl JavaMappingOutput for u8 {}
impl JavaMappingOutput for bool {}
trait CapabilityMapping<D> {
    type Input;
    type Output;
    type Error;
}
#[derive(Clone, Copy)]
struct Mapping;
impl sealed::JavaCapabilityMapping for Mapping {}
impl CapabilityMapping<JavaDialect> for Mapping {
    type Input = u8;
    type Output = u8;
    type Error = Vec<Diagnostic>;
}
struct Plan;
impl sealed::JavaMappingPlan for Plan {}
"""
    plan_impl = r"""
impl JavaMappingPlan for Plan {
    type Output = u8;
    fn representation(&self) -> JavaRepresentation { JavaRepresentation::Direct }
    fn verify_output(&self, _: &u8) -> bool { true }
}
"""
    mapping_members = r"""
    type Plan = Plan;
    fn select_plan(&self, _: &u8) -> Result<Plan, Vec<Diagnostic>> { Ok(Plan) }
"""
    cases = [
        ("positive", plan_impl, mapping_members, None),
        ("missing_plan", plan_impl, "", ("E0046", "Plan", "select_plan")),
        ("missing_selector", plan_impl, "type Plan = Plan;", ("E0046", "select_plan")),
        ("missing_verifier", "impl JavaMappingPlan for Plan { type Output = u8; }",
         mapping_members, ("E0046", "representation", "verify_output")),
        ("wrong_output", plan_impl.replace("type Output = u8", "type Output = bool")
         .replace("_: &u8", "_: &bool"), mapping_members, ("E0271",)),
    ]
    for name, implementation, members, expected in cases:
        source = Path(scratch) / f"{name}.rs"
        source.write_text(
            prefix + plan + contract + implementation
            + "impl JavaCapabilityMapping for Mapping {" + members + "}",
            encoding="utf-8",
        )
        result = subprocess.run(
            [compiler, "--edition=2024", "--crate-type=lib", "--emit=metadata",
             str(source), "-o", str(Path(scratch) / f"{name}.rmeta")],
            capture_output=True, text=True, check=False,
        )
        if expected is None:
            assert result.returncode == 0, result.stderr
        else:
            assert result.returncode != 0, f"{name} unexpectedly compiled"
            for fragment in expected:
                assert fragment in result.stderr, (name, fragment, result.stderr)
    print("actual Java mapping contract: positive control and four exact compiler rejections")


if __name__ == "__main__":
    main()
