//! Legacy checked payload enums also require distinct nested generated binders.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{ModuleBuilder, Operation, Parameter, Type, Visibility};
use portable_codegen::{Backend, BackendOptions};

#[test]
fn nested_payload_enum_matches_allocate_nonoverlapping_generated_locals() {
    let mut module = ModuleBuilder::new("nested_payload_matches");
    let (choice, (variant, field)) =
        module.enumeration("Choice", Visibility::Public, vec![], |enumeration| {
            enumeration.variant("Boxed", vec![], |variant| {
                variant.field("value", Type::i32(), vec![])
            })
        });
    module.function("nested", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("left", Type::named(choice)));
        function.parameter(Parameter::new("right", Type::named(choice)));
        function.returns(Type::i32());
        function.body(|body| {
            let outer_pattern = body.enum_pattern(choice, variant, [(field, "x".to_owned())]);
            let inner_pattern = body.enum_pattern(choice, variant, [(field, "y".to_owned())]);
            let x = body.local("x");
            let y = body.local("y");
            let sum = body.intrinsic(Operation::IntAddWrapping, [x, y]);
            let inner_body = body.block([], Some(sum));
            let inner_arm = body.match_arm(inner_pattern, inner_body);
            let right = body.local("right");
            let inner = body.match_value(right, [inner_arm]);
            let outer_body = body.block([], Some(inner));
            let outer_arm = body.match_arm(outer_pattern, outer_body);
            let left = body.local("left");
            let outer = body.match_value(left, [outer_arm]);
            body.block([], Some(outer))
        });
    });
    let checked = module.finish().unwrap();
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    CompiledPackage::new(&manifest, "nested-payload-patterns").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (Generated.nested(new Generated.ChoiceBoxed(7), new Generated.ChoiceBoxed(11)).value() != 18) {
            throw new AssertionError("nested pattern binding identity changed");
        }
    }
}
"#,
    );
}
