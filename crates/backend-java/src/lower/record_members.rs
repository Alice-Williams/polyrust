//! Java lowering: record members.

use super::Lowering;
use super::call_builders::{member_call, runtime_call};
use super::declaration_builders::{identifier, visibility_modifier};
use super::expression_builders::{binary, bool_literal, instance_of, unary};
use crate::ast::{
    JavaAnnotation, JavaBinaryOperator, JavaBlock, JavaConstructor, JavaExpr, JavaExprKind,
    JavaFieldRef, JavaKnownType, JavaMemberOrigin, JavaMethod, JavaMethodDeclaration, JavaModifier,
    JavaParameter, JavaPrecedence, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType,
    JavaTypeName, JavaUnaryOperator, JavaValueRef,
};
use crate::dialect::JavaRuntimeCallable;
use portable_codegen::GeneratedTypeId;
use portable_core_ir::CoreFieldId;
use portable_diagnostics::Diagnostic;
use portable_ir::v0::Visibility;

impl Lowering<'_> {
    pub(super) fn generated_record_constructor(
        &self,
        owner: GeneratedTypeId,
        name: &str,
        visibility: Visibility,
        fields: &[CoreFieldId],
    ) -> Result<JavaConstructor, Vec<Diagnostic>> {
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let mut parameters = Vec::new();
        let mut statements = Vec::new();
        for field_id in fields {
            let field = self.core.field(*field_id).expect("verified field");
            let ty = self.ty(field.ty)?;
            let field_name = self.names.field(*field_id).clone();
            let input = JavaExpr::local(ty.clone(), field_name.clone());
            let normalized = self.normalize_boundary_value(field.ty, input)?;
            statements.extend(normalized.statements);
            parameters.push(JavaParameter {
                ty: ty.clone(),
                name: field_name.clone(),
                final_parameter: true,
            });
            statements.push(JavaStmt::Assign {
                target: JavaExpr {
                    ty: ty.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Field {
                        receiver: Box::new(JavaExpr {
                            ty: owner_type.clone(),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Value(JavaValueRef::This),
                        }),
                        field: JavaFieldRef::Generated {
                            owner,
                            field: *field_id,
                            name: field_name,
                            ty,
                        },
                    },
                },
                value: normalized.value,
            });
        }
        Ok(JavaConstructor {
            modifiers: vec![visibility_modifier(visibility)],
            name: identifier(name),
            parameters,
            body: JavaBlock::new(statements),
        })
    }

    pub(super) fn value_equality_method(
        &self,
        owner: GeneratedTypeId,
        fields: &[CoreFieldId],
        callable: JavaRuntimeCallable,
        member: JavaRuntimeMember,
    ) -> Result<JavaMethod, Vec<Diagnostic>> {
        let object = JavaType::known(JavaKnownType::Object);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let owner_type = JavaType::Reference(JavaTypeName::Generated(owner));
        let this = JavaExpr {
            ty: owner_type.clone(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Value(JavaValueRef::This),
        };
        let other = JavaExpr::local(owner_type.clone(), identifier("otherValue"));
        let mut equal = bool_literal(true);
        for field in fields {
            let metadata = self.core.field(*field).expect("verified field");
            let field_type = self.ty(metadata.ty)?;
            equal = binary(
                JavaBinaryOperator::LogicalAnd,
                equal,
                runtime_call(
                    callable,
                    vec![
                        member_call(
                            this.clone(),
                            self.names.field(*field).as_str(),
                            vec![],
                            field_type.clone(),
                            JavaMemberOrigin::GeneratedField(*field),
                        ),
                        member_call(
                            other.clone(),
                            self.names.field(*field).as_str(),
                            vec![],
                            field_type,
                            JavaMemberOrigin::GeneratedField(*field),
                        ),
                    ],
                    boolean.clone(),
                ),
                boolean.clone(),
            );
        }
        Ok(JavaMethod {
            declared: JavaMethodDeclaration::Structural,
            annotations: vec![JavaAnnotation::Override],
            modifiers: vec![JavaModifier::Public],
            type_parameters: vec![],
            return_type: boolean.clone(),
            name: identifier(member.name()),
            parameters: vec![JavaParameter {
                ty: object.clone(),
                name: identifier("other"),
                final_parameter: true,
            }],
            body: Some(JavaBlock::new(vec![
                JavaStmt::If {
                    condition: unary(
                        JavaUnaryOperator::Not,
                        instance_of(
                            JavaExpr::local(object, identifier("other")),
                            owner_type,
                            Some(identifier("otherValue")),
                        ),
                        boolean.clone(),
                    ),
                    then_block: JavaBlock::new(vec![JavaStmt::Return(Some(bool_literal(false)))]),
                    else_block: None,
                },
                JavaStmt::Return(Some(equal)),
            ])),
        })
    }
}
