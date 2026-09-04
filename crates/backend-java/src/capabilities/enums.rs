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
pub struct JavaEnumBranchInput {
    pub(crate) variant: GeneratedValueId,
    pub(crate) body: JavaBlock,
}

#[doc(hidden)]
pub enum JavaEnumsInput {
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
        enumeration: GeneratedTypeId,
        left: Box<JavaExpr>,
        right: Box<JavaExpr>,
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
    Declaration(Box<JavaTypeDeclaration>),
    Expression(Box<JavaExpr>),
    Statement(Box<JavaStmt>),
}

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
            JavaEnumsInput::Declaration {
                declared,
                visibility,
                name,
                variants,
            } => Ok(JavaEnumsNode::Declaration(Box::new(JavaTypeDeclaration {
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
            }))),
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
                    JavaBinaryOperator::Equal,
                    *left,
                    *right,
                    JavaType::primitive(crate::ast::JavaPrimitive::Boolean),
                ))))
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
mod tests {
    use portable_build::CapabilityMapping;
    use portable_codegen::{
        GeneratedOrigin, GeneratedType, GeneratedValue, SynthesisReason, TargetAstBuilder,
        TargetTypeRef,
    };

    use super::*;
    use crate::ast::{JavaIdentifier, JavaKnownType, JavaLiteral, JavaPrimitive, JavaVisibility};

    fn source(label: &str) -> SourceRef {
        SourceRef::logical(["java-enums-mapping-test", label])
    }

    fn enum_symbols(name: &str) -> (GeneratedTypeId, GeneratedValueId, GeneratedValueId) {
        let mut builder = TargetAstBuilder::new(JavaDialect);
        let enumeration = builder.generated_type(GeneratedType {
            name: name.to_owned(),
            kind: JavaDeclarationKind::Enum,
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source(name),
        });
        let value_type = TargetTypeRef::Generated(enumeration);
        let first = builder.value(GeneratedValue {
            name: "FIRST".to_owned(),
            ty: value_type.clone(),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("first"),
        });
        let second = builder.value(GeneratedValue {
            name: "SECOND".to_owned(),
            ty: value_type,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("second"),
        });
        (enumeration, first, second)
    }

    fn local(enumeration: GeneratedTypeId, name: &str) -> JavaExpr {
        JavaExpr::local(enum_type(enumeration), JavaIdentifier::from_portable(name))
    }

    fn arm(variant: GeneratedValueId, value: i32) -> JavaEnumBranchInput {
        JavaEnumBranchInput {
            variant,
            body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Int),
                JavaLiteral::I32(value),
            )))]),
        }
    }

    #[test]
    fn every_enum_mapping_operation_constructs_typed_java_ast() {
        let (enumeration, first, second) = enum_symbols("Choice");
        let mapping = JavaEnums;

        let declaration = mapping
            .lower(
                &mut (),
                JavaEnumsInput::Declaration {
                    declared: enumeration,
                    visibility: Visibility::Public,
                    name: "Choice".to_owned(),
                    variants: vec![
                        JavaEnumVariantInput {
                            declared: first,
                            name: "FIRST".to_owned(),
                        },
                        JavaEnumVariantInput {
                            declared: second,
                            name: "SECOND".to_owned(),
                        },
                    ],
                },
            )
            .expect("enum declaration maps");
        assert!(matches!(
            declaration,
            JavaEnumsNode::Declaration(value)
                if value.kind == JavaDeclarationKind::Enum && value.members.len() == 2
        ));

        let variant = mapping
            .lower(
                &mut (),
                JavaEnumsInput::Variant {
                    enumeration,
                    variant: first,
                },
            )
            .expect("enum variant maps");
        assert!(matches!(variant, JavaEnumsNode::Expression(_)));

        let equality = mapping
            .lower(
                &mut (),
                JavaEnumsInput::Equality {
                    enumeration,
                    left: Box::new(local(enumeration, "left")),
                    right: Box::new(local(enumeration, "right")),
                },
            )
            .expect("enum equality maps");
        assert!(matches!(equality, JavaEnumsNode::Expression(_)));

        let branch = mapping
            .lower(
                &mut (),
                JavaEnumsInput::Branch {
                    selector: Box::new(local(enumeration, "value")),
                    enumeration,
                    declared_variants: vec![first, second],
                    arms: vec![arm(first, 1), arm(second, 2)],
                },
            )
            .expect("exhaustive enum branch maps");
        assert!(matches!(
            branch,
            JavaEnumsNode::Statement(value)
                if matches!(*value, JavaStmt::Switch { ref arms, .. } if arms.len() == 3)
        ));
    }

    #[test]
    fn enum_mapping_rejects_wrong_types_and_non_exhaustive_branches() {
        let (enumeration, first, second) = enum_symbols("Choice");
        let mapping = JavaEnums;

        let equality = mapping.lower(
            &mut (),
            JavaEnumsInput::Equality {
                enumeration,
                left: Box::new(local(enumeration, "left")),
                right: Box::new(JavaExpr::local(
                    JavaType::known(JavaKnownType::String),
                    JavaIdentifier::from_portable("right"),
                )),
            },
        );
        assert!(equality.is_err());

        let missing = mapping.lower(
            &mut (),
            JavaEnumsInput::Branch {
                selector: Box::new(local(enumeration, "value")),
                enumeration,
                declared_variants: vec![first, second],
                arms: vec![arm(first, 1)],
            },
        );
        assert!(missing.is_err());

        let duplicate = mapping.lower(
            &mut (),
            JavaEnumsInput::Branch {
                selector: Box::new(local(enumeration, "value")),
                enumeration,
                declared_variants: vec![first, second],
                arms: vec![arm(first, 1), arm(first, 2)],
            },
        );
        assert!(duplicate.is_err());
    }
}
