//! Unused constants must match their authoritative declaration registration.

use super::lowering_fixture::verify_members;
use super::{Lowering, identifier};
use crate::ast::{JavaExpr, JavaLiteral, JavaMember, JavaModifier, JavaPrimitive, JavaType};
use crate::capabilities::java_capabilities;
use crate::preflight::JavaCapabilitySelection;
use portable_build::{ModuleBuilder, Type, Value, Visibility};

#[derive(Clone, Copy, Debug)]
enum Mutation {
    Valid,
    Name,
    Type,
    Static,
    Final,
    Initializer,
    HidePublic,
    ExposePrivate,
}

#[test]
fn unused_static_values_keep_registered_type_name_shape_and_visibility() {
    for mutation in [
        Mutation::Valid,
        Mutation::Name,
        Mutation::Type,
        Mutation::Static,
        Mutation::Final,
        Mutation::Initializer,
        Mutation::HidePublic,
        Mutation::ExposePrivate,
    ] {
        let mut module = ModuleBuilder::new("constant_mutations");
        for (name, visibility) in [
            ("PUBLIC", Visibility::Public),
            ("PRIVATE", Visibility::Package),
        ] {
            module.constant(name, visibility, vec![], Type::i32(), |body| {
                body.constant_literal(Value::i32(7))
            });
        }
        let checked = module.finish().unwrap();
        let core = portable_core_ir::lower_checked(&checked).unwrap();
        let selection = JavaCapabilitySelection::for_test(&core);
        let mut lowering = Lowering::new(&core, &selection, java_capabilities());
        lowering.register_types();
        lowering.register_values_and_callables().unwrap();
        let mut fields = lowering
            .ordered_constant_ids()
            .unwrap()
            .into_iter()
            .map(|id| lowering.constant_field(id).unwrap())
            .collect::<Vec<_>>();
        let public = fields
            .iter()
            .position(|field| field.modifiers.contains(&JavaModifier::Public))
            .unwrap();
        let private = fields
            .iter()
            .position(|field| field.modifiers.contains(&JavaModifier::Private))
            .unwrap();
        let field = &mut fields[public];
        match mutation {
            Mutation::Valid => {}
            Mutation::Name => field.name = identifier("WRONG"),
            Mutation::Type => {
                field.ty = JavaType::primitive(JavaPrimitive::Boolean);
                field.initializer = Some(JavaExpr::literal(
                    field.ty.clone(),
                    JavaLiteral::Boolean(true),
                ));
            }
            Mutation::Static => field
                .modifiers
                .retain(|value| *value != JavaModifier::Static),
            Mutation::Final => field
                .modifiers
                .retain(|value| *value != JavaModifier::Final),
            Mutation::Initializer => field.initializer = None,
            Mutation::HidePublic => {
                field.modifiers = vec![
                    JavaModifier::Private,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ]
            }
            Mutation::ExposePrivate => {
                fields[private].modifiers = vec![
                    JavaModifier::Public,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ]
            }
        }
        let result = verify_members(
            lowering,
            fields.into_iter().map(JavaMember::Field).collect(),
        );
        assert_eq!(
            result.is_ok(),
            matches!(mutation, Mutation::Valid),
            "{mutation:?}: {result:?}"
        );
    }
}
