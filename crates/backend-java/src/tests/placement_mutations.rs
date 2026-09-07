//! Moving declarations must not invalidate unchanged outer-scope references.

use super::lowering_fixture::verify_members_with;
use super::{Lowering, identifier};
use crate::ast::{JavaMember, JavaMethodDeclaration};
use crate::capabilities::java_capabilities;
use crate::preflight::JavaCapabilitySelection;
use portable_build::{ModuleBuilder, Parameter, Type, Value, Visibility};
use portable_core_ir::CoreDeclaration;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Valid,
    Function,
    Constant,
    Nominal,
    GenericEntry,
    ParameterQualifier,
    LocalQualifier,
    FieldQualifier,
}

#[test]
fn generated_symbols_retain_the_lexical_owner_used_by_unchanged_references() {
    for mutation in [
        Mutation::Valid,
        Mutation::Function,
        Mutation::Constant,
        Mutation::Nominal,
        Mutation::GenericEntry,
        Mutation::ParameterQualifier,
        Mutation::LocalQualifier,
        Mutation::FieldQualifier,
    ] {
        let mut module = ModuleBuilder::new("placement");
        let (record, ()) = module.record("Datum", Visibility::Public, vec![], |_| ());
        module.record("Container", Visibility::Public, vec![], |_| ());
        let constant = module.constant("VALUE", Visibility::Public, vec![], Type::i32(), |body| {
            body.constant_literal(Value::i32(7))
        });
        let function = module.function("f", Visibility::Public, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let value = body.constant(constant);
                body.block([], Some(value))
            });
        });
        module.function("g", Visibility::Public, vec![], |declaration| {
            declaration.parameter(Parameter::new("input", Type::i32()));
            declaration.returns(Type::i32());
            declaration.body(|body| {
                let value = body.call(function, []);
                body.block([], Some(value))
            });
        });
        module.function("datum", Visibility::Public, vec![], |declaration| {
            declaration.returns(Type::named(record));
            declaration.body(|body| {
                let value = body.record(record, []);
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
            members.push(match *declaration {
                CoreDeclaration::Record(id) => {
                    JavaMember::NestedType(lowering.record_declaration(id).unwrap())
                }
                CoreDeclaration::Function(id) => {
                    JavaMember::Method(lowering.function_method(id).unwrap())
                }
                CoreDeclaration::Constant(id) => {
                    JavaMember::Field(lowering.constant_field(id).unwrap())
                }
                _ => unreachable!(),
            });
        }
        let position = members.iter().position(|member| match (mutation, member) {
            (Mutation::Function, JavaMember::Method(method)) => {
                matches!(method.declared, JavaMethodDeclaration::Callable(_))
                    && method.name.as_str() == "f"
            }
            (Mutation::Constant, JavaMember::Field(_)) => true,
            (Mutation::Nominal, JavaMember::NestedType(value)) => value.name.as_str() == "Datum",
            _ => false,
        });
        if let Some(position) = position {
            let moved = members.remove(position);
            let container = members
                .iter_mut()
                .find_map(|member| match member {
                    JavaMember::NestedType(value) if value.name.as_str() == "Container" => {
                        Some(value)
                    }
                    _ => None,
                })
                .unwrap();
            container.members.push(moved);
        }
        let result = verify_members_with(lowering, members, |entry| {
            if matches!(mutation, Mutation::FieldQualifier) {
                entry.members.push(JavaMember::Field(crate::ast::JavaField {
                    declared: None,
                    modifiers: vec![
                        crate::ast::JavaModifier::Private,
                        crate::ast::JavaModifier::Static,
                        crate::ast::JavaModifier::Final,
                    ],
                    ty: crate::ast::JavaType::primitive(crate::ast::JavaPrimitive::Int),
                    name: identifier("org"),
                    initializer: Some(super::i32_literal(0)),
                }));
            }
            if matches!(
                mutation,
                Mutation::ParameterQualifier | Mutation::LocalQualifier
            ) {
                let method = entry
                    .members
                    .iter_mut()
                    .find_map(|member| match member {
                        JavaMember::Method(method) if method.name.as_str() == "g" => Some(method),
                        _ => None,
                    })
                    .unwrap();
                if matches!(mutation, Mutation::ParameterQualifier) {
                    method.parameters[0].name = identifier("org");
                } else {
                    method.body.as_mut().unwrap().statements.insert(
                        0,
                        crate::ast::JavaStmt::Local {
                            finality: crate::ast::JavaLocalFinality::Final,
                            ty: crate::ast::JavaType::primitive(crate::ast::JavaPrimitive::Int),
                            name: identifier("org"),
                            value: Some(super::i32_literal(0)),
                        },
                    );
                }
            }
            if matches!(mutation, Mutation::GenericEntry) {
                entry.type_parameters.push(identifier("T"));
            }
        });
        assert_eq!(
            result.is_ok(),
            matches!(mutation, Mutation::Valid),
            "{mutation:?}: {result:?}"
        );
    }
}
