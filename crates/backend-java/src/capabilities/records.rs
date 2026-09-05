//! Java mapping for the complete `Records` capability.

use portable_build::{CapabilityMapping, Records};
use portable_codegen::GeneratedTypeId;
use portable_diagnostics::Diagnostic;
use portable_ir::v0::Visibility;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaConstructorRef, JavaDeclarationKind, JavaExpr, JavaExprKind, JavaHeritage, JavaMember,
        JavaMemberOrigin, JavaModifier, JavaPrecedence, JavaRecordComponent, JavaType,
        JavaTypeDeclaration, JavaTypeName,
    },
    dialect::JavaDialect,
    lower::{identifier, java_visibility, member_call},
};

#[doc(hidden)]
pub struct JavaRecordDeclarationInput {
    pub(crate) declared: GeneratedTypeId,
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) components: Vec<JavaRecordComponent>,
    pub(crate) heritage: JavaHeritage,
    pub(crate) members: Vec<JavaMember>,
}

#[doc(hidden)]
pub enum JavaRecordsInput {
    Declaration(Box<JavaRecordDeclarationInput>),
    Construction {
        owner: GeneratedTypeId,
        arguments: Vec<JavaExpr>,
        result: JavaType,
    },
    Field {
        receiver: Box<JavaExpr>,
        name: String,
        result: JavaType,
        origin: JavaMemberOrigin,
    },
}

#[doc(hidden)]
pub enum JavaRecordsNode {
    Declaration(JavaTypeDeclaration),
    Expression(JavaExpr),
}

impl sealed::JavaMappingOutput for JavaRecordsNode {}
impl super::support::JavaMappingOutput for JavaRecordsNode {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaRecords;

impl sealed::JavaCapabilityMapping for JavaRecords {}
impl JavaCapabilityMapping for JavaRecords {}

impl CapabilityMapping<JavaDialect> for JavaRecords {
    type Capability = Records;
    type Context = ();
    type Input = JavaRecordsInput;
    type Output = JavaRecordsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaRecordsInput::Declaration(input) => {
                JavaRecordsNode::Declaration(JavaTypeDeclaration {
                    declared: Some(input.declared),
                    kind: JavaDeclarationKind::Record,
                    visibility: java_visibility(input.visibility),
                    modifiers: vec![JavaModifier::Static],
                    name: identifier(&input.name),
                    type_parameters: vec![],
                    record_components: input.components,
                    heritage: input.heritage,
                    permits: vec![],
                    members: input.members,
                })
            }
            JavaRecordsInput::Construction {
                owner,
                arguments,
                result,
            } => {
                let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
                let created = JavaExpr {
                    ty: owner_type.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::New {
                        constructor: JavaConstructorRef::Generated {
                            owner,
                            parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
                        },
                        arguments,
                    },
                };
                let value = if owner_type == result {
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
                JavaRecordsNode::Expression(value)
            }
            JavaRecordsInput::Field {
                receiver,
                name,
                result,
                origin,
            } => JavaRecordsNode::Expression(member_call(*receiver, &name, vec![], result, origin)),
        })
    }
}
