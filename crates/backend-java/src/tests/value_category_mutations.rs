//! Enum variants and enum-typed constants have different lexical owners.

use super::Lowering;
use super::lowering_fixture::verify_members;
use crate::ast::{JavaEnumConstant, JavaMember};
use crate::capabilities::java_capabilities;
use crate::preflight::JavaCapabilitySelection;
use portable_build::{ModuleBuilder, Type, Visibility};
use portable_core_ir::CoreDeclaration;

#[test]
fn enum_typed_constant_cannot_become_an_enum_variant() {
    for mutation in 0..3 {
        let mut module = ModuleBuilder::new("value_categories");
        let (enumeration, (variant, ())) =
            module.enumeration("E", Visibility::Public, vec![], |enumeration| {
                enumeration.variant("ONE", vec![], |_| ())
            });
        let constant = module.constant(
            "DEFAULT",
            Visibility::Public,
            vec![],
            Type::named(enumeration),
            |body| body.constant_enum(enumeration, variant, []),
        );
        module.function("get", Visibility::Public, vec![], |function| {
            function.returns(Type::named(enumeration));
            function.body(|body| {
                let value = body.constant(constant);
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let mut lowering = Lowering::new(&core, &selection, java_capabilities());
        lowering.register_types();
        lowering.register_values_and_callables().unwrap();
        let mut members = Vec::new();
        for declaration in &core.module().declarations {
            match *declaration {
                CoreDeclaration::Enum(id) => members.extend(
                    lowering
                        .enum_declarations(id)
                        .unwrap()
                        .into_iter()
                        .map(JavaMember::NestedType),
                ),
                CoreDeclaration::Constant(id) => {
                    members.push(JavaMember::Field(lowering.constant_field(id).unwrap()))
                }
                CoreDeclaration::Function(id) => {
                    members.push(JavaMember::Method(lowering.function_method(id).unwrap()))
                }
                _ => unreachable!(),
            }
        }
        if mutation == 1 {
            let index = members
                .iter()
                .position(|member| matches!(member, JavaMember::Field(_)))
                .unwrap();
            let JavaMember::Field(field) = members.remove(index) else {
                unreachable!()
            };
            let enumeration = members
                .iter_mut()
                .find_map(|member| match member {
                    JavaMember::NestedType(value) => Some(value),
                    _ => None,
                })
                .unwrap();
            enumeration
                .members
                .push(JavaMember::EnumConstant(JavaEnumConstant {
                    declared: field.declared.unwrap(),
                    name: field.name,
                }));
        }
        if mutation == 2 {
            let field = members
                .iter_mut()
                .find_map(|member| match member {
                    JavaMember::Field(field) => Some(field),
                    _ => None,
                })
                .unwrap();
            let initializer = field.initializer.as_mut().unwrap();
            let crate::ast::JavaExprKind::Value(crate::ast::JavaValueRef::EnumVariant {
                variant,
                ..
            }) = initializer.kind
            else {
                panic!("baseline constant must reference its exact enum variant")
            };
            initializer.kind =
                crate::ast::JavaExprKind::Value(crate::ast::JavaValueRef::Generated(
                    portable_codegen::GeneratedSymbolId::Value(variant),
                ));
        }
        let result = verify_members(lowering, members);
        assert_eq!(
            result.is_ok(),
            mutation == 0,
            "mutation {mutation}: {result:?}"
        );
    }
}
