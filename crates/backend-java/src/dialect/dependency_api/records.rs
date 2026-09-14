//! Admit immutable local records with exact, effect-free canonical constructors.
use super::inventory::scalar;
use super::{bodies::Budget, exports::Agreement};
use crate::ast::{
    JavaDeclarationKind, JavaExpr, JavaExprKind, JavaFieldRef, JavaHeritage, JavaMember,
    JavaModifier, JavaPrecedence, JavaRecordComponentOrigin, JavaSourceDeclaration,
    JavaSourceInventory, JavaStmt, JavaType, JavaTypeDeclaration, JavaTypeName, JavaValueRef,
    JavaVisibility,
};
use portable_codegen::{GeneratedOrigin, GeneratedSymbolId, GeneratedTypeId};

pub(super) fn verify<'a>(
    record: &'a JavaTypeDeclaration,
    inventory: &JavaSourceInventory,
    agreement: &mut Agreement<'a>,
    budget: &mut Budget,
) -> Result<GeneratedTypeId, String> {
    let id = record
        .declared
        .ok_or("Java dependency record has no nominal identity")?;
    let Some(JavaSourceDeclaration::Type(registration)) =
        inventory.get(GeneratedSymbolId::Type(id))
    else {
        return Err("Java dependency record has no source registration".into());
    };
    let GeneratedOrigin::RustSource(origin) = &registration.origin else {
        return Err("Java dependency record has no source origin".into());
    };
    if registration.kind != JavaDeclarationKind::Record
        || registration.visibility != JavaVisibility::Private
        || registration.name != record.name.as_str()
        || origin.externally_reachable
        || record.kind != JavaDeclarationKind::Record
        || record.visibility != JavaVisibility::Private
        || !record.modifiers.is_empty()
        || !record.type_parameters.is_empty()
        || record.heritage != JavaHeritage::None
        || !record.permits.is_empty()
        || record.record_components.is_empty()
    {
        return Err("Java dependency record must be a private immutable scalar nominal".into());
    }
    let [JavaMember::Constructor(constructor)] = record.members.as_slice() else {
        return Err("Java dependency record requires only its canonical constructor".into());
    };
    budget.charge(0)?; // Canonical constructor body block.
    if constructor.modifiers != [JavaModifier::Private]
        || constructor.name != record.name
        || constructor.parameters.len() != record.record_components.len()
        || constructor.body.statements.len() != record.record_components.len()
    {
        return Err("Java dependency record constructor shape disagrees".into());
    }
    for ((component, parameter), statement) in record
        .record_components
        .iter()
        .zip(&constructor.parameters)
        .zip(&constructor.body.statements)
    {
        // Canonical assignment statement, field expression, this receiver,
        // and local-value expression. Charge before inspecting/allocating them.
        for depth in [0, 1, 2, 1] {
            budget.charge(depth)?;
        }
        let JavaRecordComponentOrigin::RustSource(field) = &component.origin else {
            return Err("Java dependency record field lacks source identity".into());
        };
        if !scalar(&component.ty)
            || field.owner != origin.declaration
            || component.ty != parameter.ty
            || component.name != parameter.name
            || !parameter.final_parameter
        {
            return Err("Java dependency record component/constructor inventory disagrees".into());
        }
        agreement.check(&field.origin.crate_exports)?;
        let target = JavaExpr {
            ty: component.ty.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Field {
                receiver: Box::new(JavaExpr {
                    ty: JavaType::Reference(JavaTypeName::Generated(id)),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::This),
                }),
                field: JavaFieldRef::RustSource {
                    owner: id,
                    field: field.origin.declaration,
                    name: component.name.clone(),
                    ty: component.ty.clone(),
                },
            },
        };
        let expected = JavaStmt::Assign {
            target,
            value: JavaExpr::local(component.ty.clone(), component.name.clone()),
        };
        if statement != &expected {
            return Err(
                "Java dependency record constructor is not an exact field initialization".into(),
            );
        }
    }
    Ok(id)
}
