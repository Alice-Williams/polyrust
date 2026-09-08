//! Native execution proof for structured Boolean RHS prerequisites.
use super::totality_oracle::CompiledPackage;
use super::*;
#[test]
fn short_circuit_skips_fallible_right_prerequisites() {
    let mut module = ModuleBuilder::new("java_short_circuit_plans");
    for (name, operation, left) in [
        ("and_false", Operation::BoolAnd, false),
        ("or_true", Operation::BoolOr, true),
    ] {
        module.function(name, Visibility::Public, vec![], |function| {
            function.returns(Type::bool());
            function.body(|body| {
                let left = body.literal(Value::bool(left));
                let one = body.literal(Value::i32(1));
                let zero = body.literal(Value::i32(0));
                let division = body.intrinsic(Operation::IntDivChecked, [one, zero]);
                let comparison_zero = body.literal(Value::i32(0));
                let right = body.intrinsic(Operation::Equal, [division, comparison_zero]);
                let result = body.intrinsic(operation, [left, right]);
                body.block([], Some(result))
            });
        });
    }
    let checked = module.finish().expect("short-circuit fixture");
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("generate");
    CompiledPackage::new(&manifest, "checked-short-circuit").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] arguments) {
        if (!Generated.and_false().ok() || Generated.and_false().value()
            || !Generated.or_true().ok() || !Generated.or_true().value()) {
            throw new AssertionError("fallible RHS was evaluated");
        }
    }
}
"#,
    );
}
