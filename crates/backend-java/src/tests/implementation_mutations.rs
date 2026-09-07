//! Unused method declarations retain their checked implementation identity.

use super::{Lowering, identifier, source};
use crate::ast::{
    JavaDeclarationKind, JavaMember, JavaMethodDeclaration, JavaModifier, JavaVisibility,
};
use crate::capabilities::{JavaModuleInput, java_capabilities};
use crate::preflight::JavaCapabilitySelection;
use portable_build::{CapabilityMapping, ModuleBuilder, Modules, Type, Value, Visibility};
use portable_codegen::{FileGroupRole, TargetFileGroup, TargetFileMember, verify_target_ast};
use portable_core_ir::CoreDeclaration;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Valid,
    MethodIdentity,
    InterfaceIdentity,
    Witness,
    DuplicateRecordIdentity,
    OpenInterface,
    RenamedInterface,
    InterfaceVisibility,
    HidePublicFunction,
    ExposePrivateFunction,
    GenericRecord,
    GenericFunction,
    GenericInterfaceMethod,
}

#[test]
fn unused_implementation_identity_is_checked_without_any_call_site() {
    for mutation in [
        Mutation::Valid,
        Mutation::MethodIdentity,
        Mutation::InterfaceIdentity,
        Mutation::Witness,
        Mutation::DuplicateRecordIdentity,
        Mutation::OpenInterface,
        Mutation::RenamedInterface,
        Mutation::InterfaceVisibility,
        Mutation::HidePublicFunction,
        Mutation::ExposePrivateFunction,
        Mutation::GenericRecord,
        Mutation::GenericFunction,
        Mutation::GenericInterfaceMethod,
    ] {
        let mut module = ModuleBuilder::new("unused_implementations");
        let (record, ()) = module.record("Datum", Visibility::Public, vec![], |_| ());
        for (index, name) in ["First", "Second"].into_iter().enumerate() {
            let (interface, method) = module.interface(name, Visibility::Public, vec![], |value| {
                value.method("read", vec![], vec![], Some(Type::i32()))
            });
            module.implementation(
                format!("Datum{name}"),
                Visibility::Package,
                vec![],
                interface,
                record,
                |implementation| {
                    implementation.method("read", method, vec![], |method| {
                        method.returns(Type::i32());
                        method.body(|body| {
                            let value = body.literal(Value::i32(index as i32));
                            body.block([], Some(value))
                        });
                    })
                },
            );
        }
        for (name, visibility) in [
            ("public_function", Visibility::Public),
            ("private_function", Visibility::Package),
        ] {
            module.function(name, visibility, vec![], |function| {
                function.returns(Type::i32());
                function.body(|body| {
                    let value = body.literal(Value::i32(7));
                    body.block([], Some(value))
                });
            });
        }
        let checked = module.finish().unwrap();
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let mut lowering = Lowering::new(&core, &selection, java_capabilities());
        lowering.register_types();
        lowering.register_values_and_callables().unwrap();
        let mut declarations = Vec::new();
        let mut functions = Vec::new();
        for declaration in &core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    declarations.push(lowering.record_declaration(id).unwrap());
                }
                CoreDeclaration::Interface(id) => {
                    declarations.extend(lowering.interface_declarations(id).unwrap());
                }
                CoreDeclaration::Implementation(_) => {}
                CoreDeclaration::Function(id) => {
                    functions.push(lowering.function_method(id).unwrap())
                }
                _ => unreachable!(),
            }
        }
        let implemented = declarations[0]
            .members
            .iter()
            .filter_map(|member| match member {
                JavaMember::Method(method)
                    if matches!(
                        method.declared,
                        JavaMethodDeclaration::Implementation { .. }
                    ) =>
                {
                    Some(method.declared)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(implemented.len(), 2);
        let JavaMethodDeclaration::Implementation {
            method: foreign_method,
            interface: foreign_interface,
            witness: foreign_witness,
        } = implemented[1]
        else {
            unreachable!()
        };
        let method = declarations[0]
            .members
            .iter_mut()
            .find_map(|member| match member {
                JavaMember::Method(method) if method.declared == implemented[0] => Some(method),
                _ => None,
            })
            .unwrap();
        let JavaMethodDeclaration::Implementation {
            method: identity,
            interface,
            witness,
        } = &mut method.declared
        else {
            unreachable!()
        };
        match mutation {
            Mutation::Valid => {}
            Mutation::MethodIdentity => *identity = foreign_method,
            Mutation::InterfaceIdentity => *interface = foreign_interface,
            Mutation::Witness => *witness = foreign_witness,
            Mutation::GenericRecord => declarations[0].type_parameters.push(identifier("T")),
            Mutation::GenericFunction => functions[0].type_parameters.push(identifier("T")),
            Mutation::GenericInterfaceMethod => {
                let JavaMember::Method(method) = &mut declarations[1].members[0] else {
                    unreachable!()
                };
                method.type_parameters.push(identifier("T"));
            }
            Mutation::HidePublicFunction => {
                let function = functions
                    .iter_mut()
                    .find(|value| value.modifiers.contains(&JavaModifier::Public))
                    .unwrap();
                function.modifiers = vec![JavaModifier::Private, JavaModifier::Static];
            }
            Mutation::ExposePrivateFunction => {
                let function = functions
                    .iter_mut()
                    .find(|value| value.modifiers.contains(&JavaModifier::Private))
                    .unwrap();
                function.modifiers = vec![JavaModifier::Public, JavaModifier::Static];
            }
            Mutation::OpenInterface => {
                declarations[1].kind = JavaDeclarationKind::Interface;
                declarations[1].permits.clear();
            }
            Mutation::RenamedInterface => declarations[1].name = identifier("DifferentName"),
            Mutation::InterfaceVisibility => declarations[1].visibility = JavaVisibility::Private,
            Mutation::DuplicateRecordIdentity => {
                let mut duplicate = declarations[0].clone();
                duplicate.name = identifier("SameRecordDifferentAstName");
                declarations.push(duplicate);
            }
        }
        let file = lowering
            .features
            .mapping_for::<Modules>()
            .lower(
                &mut (),
                JavaModuleInput {
                    conformances: crate::ast::JavaConformanceInventory::from_checked(&core),
                    entry: lowering.entry.unwrap(),
                    declared: lowering.declared.clone(),
                    members: declarations
                        .into_iter()
                        .map(JavaMember::NestedType)
                        .chain(functions.into_iter().map(JavaMember::Method))
                        .collect(),
                },
            )
            .unwrap();
        let file = lowering.builder.file(file);
        let runtime = lowering.runtime_file().unwrap();
        for (role, file) in [
            (FileGroupRole::PublicApi, file),
            (FileGroupRole::Runtime, runtime),
        ] {
            lowering.builder.group(TargetFileGroup::new(
                role,
                vec![TargetFileMember::Source(file)],
                source("identity-mutations"),
            ));
        }
        let result = verify_target_ast(&lowering.builder.build());
        assert_eq!(
            result.is_ok(),
            matches!(mutation, Mutation::Valid),
            "{mutation:?}: {result:?}"
        );
    }
}
