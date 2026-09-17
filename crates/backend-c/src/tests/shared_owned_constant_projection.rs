//! Original typed sources authenticate every projected global and primary placement.
use super::{
    CDialect, CFileGrammar, CVisibility,
    bindings::CBindings,
    owned_constant_fixture::{Shape, fixture},
    project_c_package,
};
use crate::ast::CScalarType;
use portable_codegen::{
    TargetAstBuilder, TargetAstPackage, TargetFile, TargetTypeRef, verify_unresolved_package,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
enum Mutation {
    None,
    Name,
    Type,
    Visibility,
    MissingBinding,
    SwappedBindings,
    MissingPrimary,
    SourcePrimary,
    CoupledBindings,
}

fn swap(bindings: &mut CBindings) {
    let entries: Vec<_> = bindings
        .values
        .iter()
        .take(2)
        .map(|(key, id)| (key.clone(), *id))
        .collect();
    let [(left, a), (right, b)] = entries.as_slice() else {
        panic!("two globals")
    };
    bindings.values.insert(left.clone(), *b);
    bindings.values.insert(right.clone(), *a);
    bindings.reverse_values.insert(*b, left.clone());
    bindings.reverse_values.insert(*a, right.clone());
}

fn changed(package: &TargetAstPackage<CDialect>, mutation: Mutation) -> TargetAstPackage<CDialect> {
    let mut builder = TargetAstBuilder::new(CDialect);
    for ty in package.generated_types() {
        builder.generated_type(ty.clone());
    }
    for callable in package.callables() {
        builder.callable(callable.clone());
    }
    for (index, value) in package.values().enumerate() {
        let mut value = value.clone();
        if index == 0 {
            match mutation {
                Mutation::Name => value.name = "different_constant".into(),
                Mutation::Type => {
                    value.ty = TargetTypeRef::Primitive(crate::dialect::CPrimitiveType::Scalar(
                        CScalarType::Bool,
                    ))
                }
                Mutation::Visibility => value.visibility = CVisibility::Private,
                _ => {}
            }
        }
        builder.value(value);
    }
    for file in package.files() {
        let header = matches!(file.source_kind(), CFileGrammar::Header(_));
        let mut units = file.items().to_vec();
        let data = Arc::make_mut(&mut units[0].data);
        match mutation {
            Mutation::MissingBinding => {
                let (key, id) = data.bindings.values.first_key_value().unwrap();
                let (key, id) = (key.clone(), *id);
                data.bindings.values.remove(&key);
                data.bindings.reverse_values.remove(&id);
            }
            Mutation::SwappedBindings | Mutation::CoupledBindings => swap(&mut data.bindings),
            Mutation::MissingPrimary if header => data.declarations.clear(),
            Mutation::SourcePrimary if !header => data.declarations.extend(
                data.bindings
                    .values
                    .values()
                    .map(|id| portable_codegen::GeneratedSymbolId::Value(*id)),
            ),
            _ => {}
        }
        if matches!(mutation, Mutation::CoupledBindings) {
            swap(&mut Arc::make_mut(&mut units[0].projection).bindings);
        }
        builder.file(TargetFile::new(
            file.path().clone(),
            file.role(),
            file.module().clone(),
            *file.placement(),
            units,
            file.source_kind().clone(),
            file.source().clone(),
        ));
    }
    for group in package.groups() {
        builder.group(group.clone());
    }
    builder.build()
}

#[test]
fn owned_global_projection_cannot_change_metadata_binding_or_primary_authority() {
    let fixture = fixture(Shape::ConstantsOnly);
    let package = project_c_package(fixture.registry, fixture.files).unwrap();
    verify_unresolved_package(&CDialect, changed(&package, Mutation::None)).unwrap();
    for mutation in [
        Mutation::Name,
        Mutation::Type,
        Mutation::Visibility,
        Mutation::MissingBinding,
        Mutation::SwappedBindings,
        Mutation::MissingPrimary,
        Mutation::SourcePrimary,
        Mutation::CoupledBindings,
    ] {
        assert!(
            verify_unresolved_package(&CDialect, changed(&package, mutation)).is_err(),
            "{mutation:?}"
        );
    }
}
