//! Java mapping for the complete payload-free `Enums` capability.

use std::collections::BTreeSet;

use portable_build::{CapabilityMapping, Enums};
use portable_codegen::{GeneratedTypeId, GeneratedValueId};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef};
use portable_ir::v0::Visibility;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaBinaryOperator, JavaBlock, JavaDeclarationKind, JavaEnumConstant, JavaExpr,
        JavaExprKind, JavaHeritage, JavaMember, JavaPattern, JavaPrecedence, JavaStmt,
        JavaSwitchArm, JavaType, JavaTypeDeclaration, JavaTypeName, JavaValueRef,
    },
    dialect::JavaDialect,
    lower::{binary, identifier, java_visibility, string_literal},
};

#[doc(hidden)]
pub struct JavaEnumVariantInput {
    pub(crate) declared: GeneratedValueId,
    pub(crate) name: String,
}

#[doc(hidden)]
pub struct JavaEnumPayloadVariantInput {
    pub(crate) declared: GeneratedTypeId,
    pub(crate) name: String,
    pub(crate) components: Vec<crate::ast::JavaRecordComponent>,
    pub(crate) members: Vec<JavaMember>,
}

#[doc(hidden)]
pub struct JavaEnumBranchInput {
    pub(crate) variant: GeneratedValueId,
    pub(crate) body: JavaBlock,
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaEnumEqualityOperator {
    Equal,
    NotEqual,
}

#[doc(hidden)]
pub enum JavaEnumsInput {
    PayloadDeclaration {
        declared: GeneratedTypeId,
        visibility: Visibility,
        name: String,
        variants: Vec<JavaEnumPayloadVariantInput>,
    },
    Type {
        enumeration: GeneratedTypeId,
    },
    Declaration {
        declared: GeneratedTypeId,
        visibility: Visibility,
        name: String,
        variants: Vec<JavaEnumVariantInput>,
    },
    Variant {
        enumeration: GeneratedTypeId,
        variant: GeneratedValueId,
    },
    Equality {
        operator: JavaEnumEqualityOperator,
        enumeration: GeneratedTypeId,
        left: Box<JavaExpr>,
        right: Box<JavaExpr>,
    },
    PayloadConstruction {
        variant: GeneratedTypeId,
        arguments: Vec<JavaExpr>,
        result: JavaType,
    },
    Branch {
        selector: Box<JavaExpr>,
        enumeration: GeneratedTypeId,
        declared_variants: Vec<GeneratedValueId>,
        arms: Vec<JavaEnumBranchInput>,
    },
}

#[doc(hidden)]
pub enum JavaEnumsNode {
    Type(JavaType),
    Declaration(Vec<JavaTypeDeclaration>),
    Expression(Box<JavaExpr>),
    Statement(Box<JavaStmt>),
}

impl sealed::JavaMappingOutput for JavaEnumsNode {}
impl super::support::JavaMappingOutput for JavaEnumsNode {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaEnums;

impl sealed::JavaCapabilityMapping for JavaEnums {}
impl JavaCapabilityMapping for JavaEnums {}

impl CapabilityMapping<JavaDialect> for JavaEnums {
    type Capability = Enums;
    type Context = ();
    type Input = JavaEnumsInput;
    type Output = JavaEnumsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        match input {
            JavaEnumsInput::PayloadDeclaration {
                declared,
                visibility,
                name,
                variants,
            } => {
                let visibility = java_visibility(visibility);
                let mut declarations = vec![JavaTypeDeclaration {
                    declared: Some(declared),
                    kind: JavaDeclarationKind::SealedInterface,
                    visibility,
                    modifiers: vec![crate::ast::JavaModifier::Static],
                    name: identifier(&name),
                    type_parameters: vec![],
                    record_components: vec![],
                    heritage: JavaHeritage::None,
                    permits: variants
                        .iter()
                        .map(|variant| enum_type(variant.declared))
                        .collect(),
                    members: vec![],
                }];
                declarations.extend(variants.into_iter().map(|variant| JavaTypeDeclaration {
                    declared: Some(variant.declared),
                    kind: JavaDeclarationKind::Record,
                    visibility,
                    modifiers: vec![crate::ast::JavaModifier::Static],
                    name: identifier(&variant.name),
                    type_parameters: vec![],
                    record_components: variant.components,
                    heritage: JavaHeritage::Interfaces(vec![
                        enum_type(declared),
                        JavaType::known(crate::ast::JavaKnownType::RuntimeSemanticValue),
                    ]),
                    permits: vec![],
                    members: variant.members,
                }));
                Ok(JavaEnumsNode::Declaration(declarations))
            }
            JavaEnumsInput::Type { enumeration } => Ok(JavaEnumsNode::Type(enum_type(enumeration))),
            JavaEnumsInput::Declaration {
                declared,
                visibility,
                name,
                variants,
            } => Ok(JavaEnumsNode::Declaration(vec![JavaTypeDeclaration {
                declared: Some(declared),
                kind: JavaDeclarationKind::Enum,
                visibility: java_visibility(visibility),
                modifiers: vec![],
                name: identifier(&name),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: variants
                    .into_iter()
                    .map(|variant| {
                        JavaMember::EnumConstant(JavaEnumConstant {
                            declared: variant.declared,
                            name: identifier(&variant.name),
                        })
                    })
                    .collect(),
            }])),
            JavaEnumsInput::Variant {
                enumeration,
                variant,
            } => Ok(JavaEnumsNode::Expression(Box::new(JavaExpr {
                ty: enum_type(enumeration),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::EnumVariant {
                    enumeration,
                    variant,
                }),
            }))),
            JavaEnumsInput::Equality {
                operator,
                enumeration,
                left,
                right,
            } => {
                if left.ty != enum_type(enumeration) || right.ty != enum_type(enumeration) {
                    return Err(enum_diagnostic(
                        "Java enum equality requires two values of the same generated enum",
                    ));
                }
                Ok(JavaEnumsNode::Expression(Box::new(binary(
                    match operator {
                        JavaEnumEqualityOperator::Equal => JavaBinaryOperator::Equal,
                        JavaEnumEqualityOperator::NotEqual => JavaBinaryOperator::NotEqual,
                    },
                    *left,
                    *right,
                    JavaType::primitive(crate::ast::JavaPrimitive::Boolean),
                ))))
            }
            JavaEnumsInput::PayloadConstruction {
                variant,
                arguments,
                result,
            } => {
                let variant_type = JavaType::Reference(JavaTypeName::Generated(variant));
                let created = JavaExpr {
                    ty: variant_type.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::New {
                        constructor: crate::ast::JavaConstructorRef::Generated {
                            owner: variant,
                            parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                        },
                        arguments,
                    },
                };
                let value = if variant_type == result {
                    created
                } else {
                    JavaExpr {
                        ty: result.clone(),
                        precedence: JavaPrecedence::Unary,
                        kind: JavaExprKind::Cast {
                            target: result,
                            value: Box::new(created),
                        },
                    }
                };
                Ok(JavaEnumsNode::Expression(Box::new(value)))
            }
            JavaEnumsInput::Branch {
                selector,
                enumeration,
                declared_variants,
                arms,
            } => {
                if selector.ty != enum_type(enumeration) {
                    return Err(enum_diagnostic(
                        "Java enum branch selector has the wrong generated enum type",
                    ));
                }
                let declared = declared_variants.iter().copied().collect::<BTreeSet<_>>();
                let covered = arms.iter().map(|arm| arm.variant).collect::<BTreeSet<_>>();
                if declared.is_empty()
                    || declared.len() != declared_variants.len()
                    || covered.len() != arms.len()
                    || declared != covered
                {
                    return Err(enum_diagnostic(
                        "Java enum branch must cover every declared variant exactly once",
                    ));
                }
                let mut lowered = arms
                    .into_iter()
                    .map(|arm| JavaSwitchArm {
                        pattern: JavaPattern::EnumVariant {
                            enumeration,
                            variant: arm.variant,
                        },
                        body: arm.body,
                    })
                    .collect::<Vec<_>>();
                lowered.push(JavaSwitchArm {
                    pattern: JavaPattern::Default,
                    body: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(
                        "unreachable exhaustive enum branch",
                    ))]),
                });
                Ok(JavaEnumsNode::Statement(Box::new(JavaStmt::Switch {
                    value: *selector,
                    arms: lowered,
                })))
            }
        }
    }
}

fn enum_type(enumeration: GeneratedTypeId) -> JavaType {
    JavaType::Reference(JavaTypeName::Generated(enumeration))
}

fn enum_diagnostic(message: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        DiagnosticCode::InvalidStructure,
        message,
        SourceRef::logical(["java-enums"]),
    )]
}

#[cfg(test)]
#[path = "../tests/enum_mapping.rs"]
mod tests;
