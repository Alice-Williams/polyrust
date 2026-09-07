//! Java AST: object members.

use super::declaration_model::{
    JavaDeclarationKind, JavaMethod, JavaModifier, JavaTypeDeclaration,
};
use super::types::{JavaKnownType, JavaPrimitive, JavaType};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

pub(super) fn java_record_component_name_is_reserved(name: &str) -> bool {
    matches!(
        name,
        "clone"
            | "finalize"
            | "getClass"
            | "hashCode"
            | "notify"
            | "notifyAll"
            | "toString"
            | "wait"
    )
}

pub(super) fn verify_record_accessor(
    declaration: &JavaTypeDeclaration,
    method: &JavaMethod,
) -> Vec<AstViolation> {
    if declaration.kind != JavaDeclarationKind::Record || !method.parameters.is_empty() {
        return vec![];
    }
    let Some(component) = declaration
        .record_components
        .iter()
        .find(|component| component.name == method.name)
    else {
        return vec![];
    };
    let valid = method.modifiers.contains(&JavaModifier::Public)
        && !method.modifiers.contains(&JavaModifier::Private)
        && !method.modifiers.contains(&JavaModifier::Static)
        && !method.modifiers.contains(&JavaModifier::Abstract)
        && method.type_parameters.is_empty()
        && method.return_type == component.ty
        && method.body.is_some();
    if valid {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "explicit Java record accessor must be public, concrete, non-static, non-generic, and return its exact component type",
        )]
    }
}

pub(super) fn verify_inherited_object_method(method: &JavaMethod) -> Vec<AstViolation> {
    let name = method.name.as_str();
    let parameters = method
        .parameters
        .iter()
        .map(|parameter| &parameter.ty)
        .collect::<Vec<_>>();
    let public_instance = method.modifiers.contains(&JavaModifier::Public)
        && !method.modifiers.contains(&JavaModifier::Private)
        && !method.modifiers.contains(&JavaModifier::Static);
    let valid_override = match (name, parameters.as_slice()) {
        ("equals", [parameter]) if **parameter == JavaType::known(JavaKnownType::Object) => {
            public_instance && method.return_type == JavaType::primitive(JavaPrimitive::Boolean)
        }
        ("hashCode", []) => {
            public_instance && method.return_type == JavaType::primitive(JavaPrimitive::Int)
        }
        ("toString", []) => {
            public_instance && method.return_type == JavaType::known(JavaKnownType::String)
        }
        _ => true,
    };
    let final_object_member = matches!(
        (name, parameters.as_slice()),
        ("getClass", [])
            | ("notify", [])
            | ("notifyAll", [])
            | ("wait", [])
            | ("wait", [JavaType::Primitive(JavaPrimitive::Long)])
            | (
                "wait",
                [
                    JavaType::Primitive(JavaPrimitive::Long),
                    JavaType::Primitive(JavaPrimitive::Int)
                ]
            )
            | ("clone", [])
            | ("finalize", [])
    );
    if valid_override && !final_object_member {
        vec![]
    } else {
        vec![AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java structural method illegally collides with an inherited Object method",
        )]
    }
}
