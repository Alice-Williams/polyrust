//! Mutate the real mapping output before the ordinary target-AST verification gate.

use super::{Lowering, identifier, source};
use crate::ast::{
    JavaConstructor, JavaDeclarationKind, JavaEnumConstant, JavaKnownType, JavaMember,
    JavaMethodDeclaration, JavaModifier, JavaType, JavaTypeName, JavaVisibility,
};
use crate::capabilities::{JavaModuleInput, java_capabilities};
use crate::preflight::JavaCapabilitySelection;
use portable_build::{CapabilityMapping, ModuleBuilder, Modules, Parameter, Type, Visibility};
use portable_codegen::{
    FileGroupRole, GeneratedOrigin, GeneratedSymbolId, GeneratedValue, SynthesisReason,
    TargetFileGroup, TargetFileMember, verify_target_ast,
};
use portable_core_ir::CoreDeclaration;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Valid,
    Constant,
    Public,
    Constructor,
    Factory,
    MissingPermit,
    WrongPermit,
    MissingMethod,
    WrongMethod,
    WrongSignature,
    WrongBody,
    ExtraMethod,
    WrongName,
    DuplicateIdentity,
    JointMethodRemoval,
    DuplicateParameters,
}

#[test]
fn uninhabited_interface_privilege_rejects_every_shape_mutation() {
    for mutation in [
        Mutation::Valid,
        Mutation::Constant,
        Mutation::Public,
        Mutation::Constructor,
        Mutation::Factory,
        Mutation::MissingPermit,
        Mutation::WrongPermit,
        Mutation::MissingMethod,
        Mutation::WrongMethod,
        Mutation::WrongSignature,
        Mutation::WrongBody,
        Mutation::ExtraMethod,
        Mutation::WrongName,
        Mutation::DuplicateIdentity,
        Mutation::JointMethodRemoval,
        Mutation::DuplicateParameters,
    ] {
        let mut module = ModuleBuilder::new("empty_interface_mutations");
        for name in ["First", "Second"] {
            module.interface(name, Visibility::Public, vec![], |interface| {
                interface.method(
                    "transform",
                    vec![],
                    vec![
                        Parameter::new("input", Type::list(Type::string())),
                        Parameter::new("other", Type::list(Type::string())),
                    ],
                    Some(Type::list(Type::string())),
                )
            });
        }
        let checked = module.finish().unwrap();
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let mut lowering = Lowering::new(&core, &selection, java_capabilities());
        lowering.register_types();
        lowering.register_interface_sealing().unwrap();
        lowering.register_values_and_callables().unwrap();
        let mut declarations = core
            .module()
            .declarations
            .iter()
            .flat_map(|declaration| {
                let CoreDeclaration::Interface(id) = *declaration else {
                    panic!("interface fixture")
                };
                lowering.interface_declarations(id).unwrap()
            })
            .collect::<Vec<_>>();
        let synthetic_index = declarations
            .iter()
            .position(|value| matches!(value.kind, JavaDeclarationKind::UninhabitedEnum(_)))
            .unwrap();
        let synthetic_id = declarations[synthetic_index].declared.unwrap();
        let JavaDeclarationKind::UninhabitedEnum(interface_id) = declarations[synthetic_index].kind
        else {
            unreachable!()
        };
        let interface_index = declarations
            .iter()
            .position(|value| value.declared == Some(interface_id))
            .unwrap();
        let foreign_method = declarations
            .iter()
            .filter(|value| {
                value.kind == JavaDeclarationKind::SealedInterface
                    && value.declared != Some(interface_id)
            })
            .flat_map(|value| &value.members)
            .find_map(|member| {
                if let JavaMember::Method(method) = member {
                    Some(method.declared)
                } else {
                    None
                }
            })
            .unwrap();
        let mut extra_members = Vec::new();
        match mutation {
            Mutation::Valid => {}
            Mutation::DuplicateParameters => {
                let JavaMember::Method(method) = &mut declarations[interface_index].members[0]
                else {
                    unreachable!()
                };
                method.parameters[1].name = method.parameters[0].name.clone();
            }
            Mutation::JointMethodRemoval => {
                declarations[interface_index].members.clear();
                declarations[synthetic_index].members.clear();
            }
            Mutation::WrongName => {
                declarations[synthetic_index].name = identifier("WrongName");
            }
            Mutation::DuplicateIdentity => {
                let mut duplicate = declarations[synthetic_index].clone();
                duplicate.name = identifier("DifferentSpellingSameIdentity");
                declarations.push(duplicate);
            }
            Mutation::Constant => {
                let value = lowering.builder.value(GeneratedValue {
                    visibility: crate::ast::JavaVisibility::Public,
                    name: "INSTANCE".to_owned(),
                    ty: portable_codegen::TargetTypeRef::Generated(synthetic_id),
                    origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
                    source: source("forged-enum-value"),
                });
                lowering.declared.push(GeneratedSymbolId::Value(value));
                declarations[synthetic_index]
                    .members
                    .push(JavaMember::EnumConstant(JavaEnumConstant {
                        declared: value,
                        name: identifier("INSTANCE"),
                    }));
            }
            Mutation::Public => declarations[synthetic_index].visibility = JavaVisibility::Public,
            Mutation::Constructor => {
                let name = declarations[synthetic_index].name.clone();
                declarations[synthetic_index]
                    .members
                    .push(JavaMember::Constructor(JavaConstructor {
                        modifiers: vec![JavaModifier::Private],
                        name,
                        parameters: vec![],
                        body: crate::ast::uninhabited::unreachable_body(),
                    }));
            }
            Mutation::Factory => {
                let JavaMember::Method(mut method) =
                    declarations[synthetic_index].members[0].clone()
                else {
                    unreachable!()
                };
                method.declared = JavaMethodDeclaration::Structural;
                method.annotations.clear();
                method.modifiers = vec![JavaModifier::Public, JavaModifier::Static];
                method.name = identifier("factory");
                method.parameters.clear();
                method.return_type = JavaType::Reference(JavaTypeName::Generated(synthetic_id));
                extra_members.push(JavaMember::Method(method));
            }
            Mutation::MissingPermit => declarations[interface_index].permits.clear(),
            Mutation::WrongPermit => {
                declarations[interface_index].permits =
                    vec![JavaType::Reference(JavaTypeName::Generated(interface_id))]
            }
            Mutation::MissingMethod => declarations[synthetic_index].members.clear(),
            Mutation::WrongMethod
            | Mutation::WrongSignature
            | Mutation::WrongBody
            | Mutation::ExtraMethod => {
                let JavaMember::Method(method) = &mut declarations[synthetic_index].members[0]
                else {
                    unreachable!()
                };
                match mutation {
                    Mutation::WrongMethod => {
                        let JavaMethodDeclaration::Interface(id) = foreign_method else {
                            unreachable!()
                        };
                        method.declared = JavaMethodDeclaration::UninhabitedImplementation(id);
                    }
                    Mutation::WrongSignature => {
                        method.parameters[0].ty = JavaType::generic(
                            JavaKnownType::List,
                            vec![JavaType::known(JavaKnownType::Integer)],
                        )
                    }
                    Mutation::WrongBody => method.body = Some(crate::ast::JavaBlock::new(vec![])),
                    Mutation::ExtraMethod => {
                        let extra = JavaMember::Method(method.clone());
                        declarations[synthetic_index].members.push(extra);
                    }
                    _ => unreachable!(),
                }
            }
        }
        let members = declarations
            .into_iter()
            .map(JavaMember::NestedType)
            .chain(extra_members)
            .collect();
        let file = lowering
            .features
            .mapping_for::<Modules>()
            .lower(
                &mut (),
                JavaModuleInput {
                    conformances: crate::ast::JavaConformanceInventory::from_checked(&core),
                    entry: lowering.entry.unwrap(),
                    declared: lowering.declared.clone(),
                    members,
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
                source("mutation-group"),
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
