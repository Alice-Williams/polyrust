//! A source constant is an exact registered public static final scalar value.
use super::{Constant, JavaDialect};
use crate::ast::{
    JavaDeclaredPath, JavaField, JavaIdentifier, JavaModifier, JavaPackage, JavaResolvedName,
    JavaScalarConstantValue, JavaSourceDeclaration, JavaVisibility, ResolvedJavaFileItem,
};
use portable_codegen::{
    GeneratedOrigin, GeneratedSymbolId, RustDeclarationId, RustSourceNode, RustVisibility,
    TargetSymbolRef,
};
use std::collections::BTreeSet;

pub(super) fn verify(
    field: &JavaField,
    item: &ResolvedJavaFileItem,
    package: JavaPackage,
    owner: &JavaIdentifier,
    public: &BTreeSet<RustDeclarationId>,
) -> Result<Constant, String> {
    let generated = field
        .declared
        .ok_or("Java source constant lacks an identity")?;
    let symbol = GeneratedSymbolId::Value(generated);
    let Some(JavaSourceDeclaration::Value(registration)) = item.source_inventory.get(symbol) else {
        return Err("Java source constant lacks retained value registration".into());
    };
    let GeneratedOrigin::RustSource(source) = &registration.origin else {
        return Err("Java source constant lacks source provenance".into());
    };
    let initializer = field
        .initializer
        .as_ref()
        .ok_or("Java source constant lacks an initializer")?;
    let value = JavaScalarConstantValue::from_expression(initializer)
        .ok_or("Java source constant initializer is not an exact scalar constant")?;
    if source.node != RustSourceNode::Declaration
        || source.visibility != RustVisibility::Public
        || !source.externally_reachable
        || !public.contains(&source.declaration)
        || registration.visibility != JavaVisibility::Public
        || registration.name != field.name.as_str()
        || registration.ty != JavaDialect.registered_type(&field.ty)
        || field.modifiers.len() != 3
        || ![
            JavaModifier::Public,
            JavaModifier::Static,
            JavaModifier::Final,
        ]
        .iter()
        .all(|modifier| field.modifiers.contains(modifier))
        || initializer.ty != field.ty
        || field.ty != value.ty()
    {
        return Err("Java source constant registration/type/visibility/literal disagrees".into());
    }
    let Some(JavaResolvedName::DeclaredPath(path)) =
        item.names.get(&TargetSymbolRef::Generated(symbol))
    else {
        return Err("Java source constant lacks a resolved declared path".into());
    };
    if *path
        != (JavaDeclaredPath {
            package,
            owners: vec![owner.clone()],
            member: field.name.clone(),
        })
    {
        return Err("Java source constant resolved owner disagrees".into());
    }
    Ok(Constant {
        generated,
        source: source.clone(),
        path: path.clone(),
        ty: field.ty.clone(),
        value,
    })
}

#[cfg(test)]
#[path = "../../../tests/source_constant_inventory.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/finite_constant_inventory.rs"]
mod finite_tests;

#[cfg(test)]
#[path = "../../../tests/infinite_constant_inventory.rs"]
mod infinite_tests;
