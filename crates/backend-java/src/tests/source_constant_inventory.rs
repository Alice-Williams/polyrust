//! Exercise the narrow constant gate directly, without an earlier verifier hiding errors.
use super::*;
use crate::{
    ast::{JavaExprKind, JavaFileItem, JavaLiteral, JavaPrimitive, JavaType},
    tests::source_constant_fixture as c,
};
use portable_codegen::GeneratedSymbolId;

#[test]
fn direct_constant_projection_requires_exact_type_literal_and_declared_owner() {
    let api = c::api(false);
    let original = &api.package().ast().files()[0].items()[0];
    let JavaFileItem::Type { declaration, .. } = &original.item else {
        panic!()
    };
    let crate::ast::JavaMember::Field(field) = &declaration.members[1] else {
        panic!()
    };
    let public = api.constants().map(|c| c.declaration()).collect();
    verify(
        field,
        original,
        api.package_identity().namespace(),
        &declaration.name,
        &public,
    )
    .unwrap();
    for fault in 0..8 {
        let mut item = original.clone();
        let mut field = field.clone();
        let symbol = TargetSymbolRef::Generated(GeneratedSymbolId::Value(field.declared.unwrap()));
        match fault {
            0 => {
                item.names.remove(&symbol);
            }
            1 => {
                item.names
                    .insert(symbol, JavaResolvedName::Local(field.name.clone()));
            }
            2..=4 => {
                let Some(JavaResolvedName::DeclaredPath(path)) = item.names.get_mut(&symbol) else {
                    panic!()
                };
                match fault {
                    2 => path.package = JavaPackage::RustCrate(999),
                    3 => path.owners = vec![JavaIdentifier::new("Wrong").unwrap()],
                    4 => path.member = JavaIdentifier::new("wrong").unwrap(),
                    _ => unreachable!(),
                }
            }
            5 => field.initializer.as_mut().unwrap().ty = JavaType::primitive(JavaPrimitive::Int),
            6 => {
                field.initializer.as_mut().unwrap().kind =
                    JavaExprKind::Literal(JavaLiteral::I32(1))
            }
            7 => field.modifiers.push(JavaModifier::Final),
            _ => unreachable!(),
        }
        assert!(
            verify(
                &field,
                &item,
                api.package_identity().namespace(),
                &declaration.name,
                &public
            )
            .is_err(),
            "direct fault {fault}"
        );
    }
}
