//! Exact closed inventory and canonical storage; no name-based type recognition.
use super::{JavaDialect, JavaScalarResultTypes};
use crate::ast::{
    JavaConstructor, JavaDeclarationKind, JavaExpr, JavaExprKind, JavaFieldRef, JavaFileItem,
    JavaHeritage, JavaIdentifier, JavaMember, JavaModifier, JavaParameter, JavaPrecedence,
    JavaPrimitive, JavaRecordComponentOrigin, JavaSourceDeclaration, JavaStmt,
    JavaSynthesizedField, JavaSynthesizedFieldRole, JavaType, JavaTypeDeclaration, JavaTypeName,
    JavaValueRef, JavaVisibility,
};
use portable_codegen::{GeneratedOrigin, GeneratedSymbolId, RenderReadyPackage, SynthesisReason};

pub(super) fn family(
    package: &RenderReadyPackage<JavaDialect>,
    types: JavaScalarResultTypes,
) -> Result<JavaIdentifier, String> {
    checked_family(package, types, None)
}

pub(in crate::dialect) fn checked_family(
    package: &RenderReadyPackage<JavaDialect>,
    types: JavaScalarResultTypes,
    error_kinds: Option<super::super::error_result::JavaErrorKindValues>,
) -> Result<JavaIdentifier, String> {
    for item in package.ast().files().iter().flat_map(|file| file.items()) {
        let JavaFileItem::Type { declaration, .. } = &item.item else {
            continue;
        };
        if let Some(name) = check_item(declaration, &item.source_inventory, types, error_kinds)? {
            return Ok(name);
        }
    }
    Err(
        "Java scalar-result family must retain three exact declarations in one top-level nest"
            .into(),
    )
}

pub(in crate::dialect) fn check_item(
    declaration: &JavaTypeDeclaration,
    inventory: &crate::ast::JavaSourceInventory,
    types: JavaScalarResultTypes,
    error_kinds: Option<super::super::error_result::JavaErrorKindValues>,
) -> Result<Option<JavaIdentifier>, String> {
    if types.interface == types.success
        || types.interface == types.error
        || types.success == types.error
    {
        return Err("Java scalar-result declarations must have three distinct identities".into());
    }
    let nested = |id| {
        declaration.members.iter().find_map(|member| match member {
            JavaMember::NestedType(child) if child.declared == Some(id) => Some(child),
            _ => None,
        })
    };
    let (Some(interface), Some(success), Some(error)) = (
        nested(types.interface),
        nested(types.success),
        nested(types.error),
    ) else {
        return Ok(None);
    };
    for id in [types.interface, types.success, types.error] {
        let Some(JavaSourceDeclaration::Type(registration)) =
            inventory.get(GeneratedSymbolId::Type(id))
        else {
            return Err("Java scalar-result type lacks its original local registration".into());
        };
        if registration.origin != GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter) {
            return Err("Java scalar-result types require synthesized adapter origins".into());
        }
    }
    let interface_type = reference(types.interface);
    let permitted = [reference(types.success), reference(types.error)];
    if interface.kind != JavaDeclarationKind::SealedInterface
        || !interface.modifiers.is_empty()
        || !interface.type_parameters.is_empty()
        || !interface.record_components.is_empty()
        || !interface.members.is_empty()
        || interface.heritage != JavaHeritage::None
        || interface.permits.len() != 2
        || !permitted.iter().all(|ty| interface.permits.contains(ty))
    {
        return Err(
            "Java scalar-result interface must seal exactly its two empty-contract variants".into(),
        );
    }
    let success_constructor = variant(success, &interface_type, interface.visibility)?;
    let [component] = success.record_components.as_slice() else {
        return Err("Java scalar-result success requires exactly one primitive-int payload".into());
    };
    let field = JavaSynthesizedField {
        owner: types.success,
        role: JavaSynthesizedFieldRole::ScalarResultPayload,
    };
    if component.origin != JavaRecordComponentOrigin::Synthesized(field) || component.ty != int() {
        return Err("Java scalar-result variant payload inventory disagrees".into());
    }
    canonical_success(success_constructor, field, &component.name)?;
    if let Some(kinds) = error_kinds {
        if interface.visibility != JavaVisibility::Public
            || declaration.visibility != JavaVisibility::Public
        {
            return Err(
                "Java error-kind family must have public enclosing and selected types".into(),
            );
        }
        super::super::error_result::check::error_variant(error, &interface_type, kinds, inventory)?;
    } else {
        let error_constructor = variant(error, &interface_type, interface.visibility)?;
        if !error.record_components.is_empty()
            || !error_constructor.parameters.is_empty()
            || !error_constructor.body.statements.is_empty()
        {
            return Err("Java scalar-result variant payload inventory disagrees".into());
        }
    }
    Ok(Some(component.name.clone()))
}

fn reference(owner: portable_codegen::GeneratedTypeId) -> JavaType {
    JavaType::Reference(JavaTypeName::Generated(owner))
}
fn int() -> JavaType {
    JavaType::primitive(JavaPrimitive::Int)
}

fn variant<'a>(
    record: &'a JavaTypeDeclaration,
    interface: &JavaType,
    visibility: JavaVisibility,
) -> Result<&'a JavaConstructor, String> {
    if record.kind != JavaDeclarationKind::Record
        || record.visibility != visibility
        || !record.modifiers.is_empty()
        || !record.type_parameters.is_empty()
        || !record.permits.is_empty()
        || record.heritage != JavaHeritage::Interfaces(vec![interface.clone()])
    {
        return Err(
            "Java scalar-result variant must be an exact immutable implementing record".into(),
        );
    }
    let [JavaMember::Constructor(constructor)] = record.members.as_slice() else {
        return Err("Java scalar-result variant permits only its canonical constructor, not custom accessors or members".into());
    };
    let modifier = match visibility {
        JavaVisibility::Public => JavaModifier::Public,
        JavaVisibility::Private => JavaModifier::Private,
        JavaVisibility::Package => {
            return Err("Java scalar-result nested variants need explicit visibility".into());
        }
    };
    if constructor.modifiers != [modifier] || constructor.name != record.name {
        return Err("Java scalar-result constructor visibility or identity disagrees".into());
    }
    Ok(constructor)
}

fn canonical_success(
    constructor: &JavaConstructor,
    field: JavaSynthesizedField,
    name: &JavaIdentifier,
) -> Result<(), String> {
    let expected_parameter = JavaParameter {
        ty: int(),
        name: name.clone(),
        final_parameter: true,
    };
    let expected_assignment = JavaStmt::Assign {
        target: JavaExpr {
            ty: int(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Field {
                receiver: Box::new(JavaExpr {
                    ty: reference(field.owner),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::This),
                }),
                field: JavaFieldRef::Synthesized {
                    field,
                    name: name.clone(),
                    ty: int(),
                },
            },
        },
        value: JavaExpr::local(int(), name.clone()),
    };
    if constructor.parameters != [expected_parameter]
        || constructor.body.statements != [expected_assignment]
    {
        return Err(
            "Java scalar-result success constructor must assign its exact final input once".into(),
        );
    }
    Ok(())
}
