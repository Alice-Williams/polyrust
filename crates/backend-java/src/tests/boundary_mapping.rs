//! Boundary-only ownership: no portable tagged constructors can mask bypasses.

use crate::ast::JavaExpr;
use crate::capabilities::{
    java_capabilities, java_mapping_operation_counts, reset_java_mapping_invocations,
};
use crate::lower::{Lowering, identifier};
use crate::preflight::JavaCapabilitySelection;
use portable_build::{ModuleBuilder, Parameter, Type};
use portable_core_ir::{CoreExprKind, CoreProgram, CoreType};
use portable_ir::v0::Visibility;

fn fixture() -> CoreProgram {
    let mut module = ModuleBuilder::new("boundary_ownership");
    for (name, ty) in [
        ("option_identity", Type::option(Type::i32())),
        ("result_identity", Type::result(Type::i32(), Type::string())),
    ] {
        module.function(name, Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", ty.clone()));
            function.returns(ty);
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        });
    }
    let core = portable_core_ir::lower_checked(&module.finish().expect("fixture checks"))
        .expect("fixture lowers");
    for (_, expression) in core.expressions().iter() {
        assert!(!matches!(
            &expression.kind,
            CoreExprKind::ConstructSome(_)
                | CoreExprKind::ConstructNone { .. }
                | CoreExprKind::ConstructOk { .. }
                | CoreExprKind::ConstructErr { .. }
        ));
    }
    core
}

fn assert_tagged_mapping_operations() {
    let counts = java_mapping_operation_counts();
    for capability in [
        std::any::type_name::<portable_build::OptionValues>(),
        std::any::type_name::<portable_build::ResultValues>(),
    ] {
        assert_eq!(
            counts.get(capability),
            Some(&3),
            "type and both constructors must be mapped for {capability}"
        );
    }
}

#[test]
fn public_factories_invoke_tagged_value_mappings_without_core_constructors() {
    let core = fixture();
    let selection = JavaCapabilitySelection::for_test(&core);
    let lowering = Lowering::new(&core, &selection, java_capabilities());
    reset_java_mapping_invocations();
    assert_eq!(
        lowering
            .public_tagged_value_factories()
            .expect("factories lower")
            .len(),
        4
    );
    assert_tagged_mapping_operations();
}

#[test]
fn normalization_independently_invokes_tagged_value_mappings() {
    let core = fixture();
    let selection = JavaCapabilitySelection::for_test(&core);
    let lowering = Lowering::new(&core, &selection, java_capabilities());
    reset_java_mapping_invocations();
    for (id, ty) in core.types().iter() {
        if matches!(ty, CoreType::Option(_) | CoreType::Result { .. }) {
            let input =
                JavaExpr::local(lowering.ty(id).expect("boundary type"), identifier("value"));
            lowering
                .normalize_boundary_value(id, input)
                .expect("boundary normalizes");
        }
    }
    assert_tagged_mapping_operations();
}
