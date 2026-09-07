//! Exact verification of the target-only, zero-value interface implementor.

use super::field_metadata::find_type_declaration;
use super::method_contracts::interface_implementation_matches;
use super::{
    JavaAnnotation, JavaBlock, JavaConstructorRef, JavaDeclarationKind, JavaExpr, JavaExprKind,
    JavaFileItem, JavaHeritage, JavaKnownType, JavaLiteral, JavaMember, JavaMethod,
    JavaMethodDeclaration, JavaModifier, JavaPrecedence, JavaStmt, JavaType, JavaTypeDeclaration,
    JavaTypeName, JavaVisibility,
};
use crate::dialect::{JavaDialect, JavaKnownConstructor};
use portable_codegen::{
    AstViolation, GeneratedOrigin, GeneratedSymbolId, GeneratedTypeId, SynthesisReason,
    TargetAstContext, TargetSymbolRef,
};
use portable_core_ir::CoreDeclaration;
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeSet;

pub(crate) fn unreachable_body() -> JavaBlock {
    let owner = JavaType::known(JavaKnownType::AssertionError);
    let string = JavaType::known(JavaKnownType::String);
    JavaBlock::new(vec![JavaStmt::Throw(JavaExpr {
        ty: owner.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor: JavaKnownConstructor::AssertionErrorString,
                owner,
                parameters: vec![string.clone()],
            },
            arguments: vec![JavaExpr::literal(
                string,
                JavaLiteral::String("uninhabited interface".to_owned()),
            )],
        },
    })])
}

pub(super) fn method_matches(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
    context: &TargetAstContext<'_, JavaDialect>,
) -> bool {
    let JavaDeclarationKind::UninhabitedEnum(owner) = declaration.kind else {
        return false;
    };
    let JavaMethodDeclaration::UninhabitedImplementation(id) = method.declared else {
        return false;
    };
    context.interface_method(id).is_some_and(|registered| {
        registered.owner == owner
            && method.annotations == [JavaAnnotation::Override]
            && method.modifiers == [JavaModifier::Public]
            && method.type_parameters.is_empty()
            && method
                .parameters
                .iter()
                .all(|parameter| parameter.final_parameter)
            && method.body.as_ref() == Some(&unreachable_body())
            && interface_implementation_matches(
                declaration,
                method,
                &registered.signature,
                &registered.name,
                owner,
            )
    })
}

pub(super) fn verify(
    declaration: &JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
    top_level: bool,
) -> Vec<AstViolation> {
    let JavaDeclarationKind::UninhabitedEnum(interface) = declaration.kind else {
        return vec![];
    };
    let registered = declaration
        .declared
        .and_then(|id| context.generated_type(id));
    let origin_matches = registered.is_some_and(|value| {
        value.kind == declaration.kind
            && value.name == declaration.name.as_str()
            && value.visibility == JavaVisibility::Private
            && value.origin == GeneratedOrigin::Synthesized(SynthesisReason::UninhabitedInterface)
    });
    let interface_type = JavaType::Reference(JavaTypeName::Generated(interface));
    let registered_interface = context.generated_type(interface).is_some_and(|value| {
        value.kind == JavaDeclarationKind::SealedInterface
            && matches!(
                value.origin,
                GeneratedOrigin::CoreDeclaration(CoreDeclaration::Interface(_))
            )
    });
    let permitted = find_type_declaration(&interface_type, context).is_some_and(|owner| {
        declaration
            .declared
            .is_some_and(|id| owner.permits == [JavaType::Reference(JavaTypeName::Generated(id))])
    });
    let members_valid = declaration.members.iter().all(|member| {
        matches!(member, JavaMember::Method(method) if method_matches(declaration, method, context))
    });
    let shape_valid = !top_level
        && declaration.visibility == JavaVisibility::Private
        && declaration.modifiers.is_empty()
        && declaration.type_parameters.is_empty()
        && declaration.record_components.is_empty()
        && declaration.permits.is_empty()
        && declaration.heritage == JavaHeritage::Interfaces(vec![interface_type]);
    let unexposed = declaration.declared.is_some_and(|id| {
        !context.files().any(|file| {
            file.items().iter().any(|item| {
            matches!(item, JavaFileItem::Type { declaration, .. } if exposes_type(declaration, id))
        })
        })
    });
    if origin_matches
        && registered_interface
        && permitted
        && members_valid
        && shape_valid
        && unexposed
    {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "uninhabited Java interface implementor must be an exact private zero-value enum with no constructors or factories",
        )]
    }
}

fn type_mentions(ty: &JavaType, id: GeneratedTypeId) -> bool {
    let mut symbols = BTreeSet::new();
    ty.symbols(&mut symbols);
    symbols.contains(&TargetSymbolRef::Generated(GeneratedSymbolId::Type(id)))
}

fn exposes_type(declaration: &JavaTypeDeclaration, id: GeneratedTypeId) -> bool {
    declaration
        .record_components
        .iter()
        .any(|value| type_mentions(&value.ty, id))
        || declaration.members.iter().any(|member| match member {
            JavaMember::Field(value) => type_mentions(&value.ty, id),
            JavaMember::CompileFailField(value) => type_mentions(&value.expected_type, id),
            JavaMember::Method(value) => {
                type_mentions(&value.return_type, id)
                    || value
                        .parameters
                        .iter()
                        .any(|value| type_mentions(&value.ty, id))
            }
            JavaMember::Constructor(value) => value
                .parameters
                .iter()
                .any(|value| type_mentions(&value.ty, id)),
            JavaMember::NestedType(value) => exposes_type(value, id),
            JavaMember::EnumConstant(_) => false,
        })
}
