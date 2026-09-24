//! Bounded expressions of the certified source-owner profile.
use super::super::inventory::scalar;
use super::Reader;
use crate::ast::{
    JavaBinaryOperator, JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef, JavaLiteral,
    JavaPrecedence, JavaPrimitive, JavaRecordComponentOrigin, JavaType, JavaUnaryOperator,
    JavaValueRef,
};
use portable_codegen::GeneratedSymbolId;

impl Reader<'_> {
    pub(super) fn expression(&mut self, value: &JavaExpr, depth: usize) -> Result<(), String> {
        self.charge(depth)?;
        if !self.ty(&value.ty) {
            return Err("Java dependency body contains an unadmitted value type".into());
        }
        if self.result_expression(value, depth)? {
            return Ok(());
        }
        if matches!(value.kind, JavaExprKind::Call { .. })
            && value.precedence != JavaPrecedence::Primary
        {
            return Err("Java dependency call requires primary precedence".into());
        }
        if matches!(value.kind, JavaExprKind::Unary { .. })
            && value.precedence != crate::ast::JavaPrecedence::Unary
        {
            return Err("Java dependency unary expression requires unary precedence".into());
        }
        match &value.kind {
            JavaExprKind::Cast {
                target,
                value: operand,
            } if *target == JavaType::primitive(JavaPrimitive::Long)
                && value.ty == *target
                && operand.ty == JavaType::primitive(JavaPrimitive::Int)
                && value.precedence == JavaPrecedence::Unary =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::Literal(
                JavaLiteral::I32(_)
                | JavaLiteral::I64(_)
                | JavaLiteral::Boolean(_)
                | JavaLiteral::F64(_),
            )
            | JavaExprKind::Value(JavaValueRef::Local(_)) => {}
            JavaExprKind::Value(JavaValueRef::KnownField(
                crate::dialect::JavaKnownField::DoublePositiveInfinity
                | crate::dialect::JavaKnownField::DoubleNegativeInfinity,
            )) if value.ty == JavaType::primitive(JavaPrimitive::Double)
                && value.precedence == JavaPrecedence::Primary => {}
            JavaExprKind::Value(JavaValueRef::Dependency(imported))
                if imported.ty() == &value.ty => {}
            JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(id)))
                if self.constants.get(id) == Some(&value.ty) => {}
            JavaExprKind::Unary {
                operator: JavaUnaryOperator::Not,
                operand,
            } if value.ty == JavaType::primitive(JavaPrimitive::Boolean)
                && operand.ty == value.ty =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::Unary {
                operator: JavaUnaryOperator::Negate,
                operand,
            } if value.ty == JavaType::primitive(JavaPrimitive::Double)
                && operand.ty == value.ty =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::Unary {
                operator: JavaUnaryOperator::BitNot | JavaUnaryOperator::Negate,
                operand,
            } if matches!(
                value.ty,
                JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long)
            ) && operand.ty == value.ty =>
            {
                self.expression(operand, depth + 1)?;
            }
            JavaExprKind::Binary {
                operator:
                    JavaBinaryOperator::BitAnd | JavaBinaryOperator::BitOr | JavaBinaryOperator::BitXor,
                left,
                right,
            } if matches!(
                value.ty,
                JavaType::Primitive(
                    JavaPrimitive::Boolean | JavaPrimitive::Int | JavaPrimitive::Long
                )
            ) && left.ty == value.ty
                && right.ty == value.ty =>
            {
                self.expression(left, depth + 1)?;
                self.expression(right, depth + 1)?;
            }
            JavaExprKind::Binary {
                operator:
                    operator @ (JavaBinaryOperator::Add
                    | JavaBinaryOperator::Subtract
                    | JavaBinaryOperator::Multiply),
                left,
                right,
            } if matches!(
                value.ty,
                JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long)
            ) && left.ty == value.ty
                && right.ty == value.ty
                && value.precedence
                    == match operator {
                        JavaBinaryOperator::Multiply => JavaPrecedence::Multiplicative,
                        _ => JavaPrecedence::Additive,
                    } =>
            {
                self.expression(left, depth + 1)?;
                self.expression(right, depth + 1)?;
            }
            JavaExprKind::Binary {
                operator,
                left,
                right,
            } if matches!(
                operator,
                JavaBinaryOperator::Add
                    | JavaBinaryOperator::Subtract
                    | JavaBinaryOperator::Multiply
                    | JavaBinaryOperator::Divide
                    | JavaBinaryOperator::Remainder
            ) && value.ty == JavaType::primitive(JavaPrimitive::Double)
                && left.ty == value.ty
                && right.ty == value.ty
                && value.precedence
                    == match operator {
                        JavaBinaryOperator::Add | JavaBinaryOperator::Subtract => {
                            JavaPrecedence::Additive
                        }
                        _ => JavaPrecedence::Multiplicative,
                    } =>
            {
                self.expression(left, depth + 1)?;
                self.expression(right, depth + 1)?;
            }
            JavaExprKind::Binary {
                operator,
                left,
                right,
            } if matches!(
                operator,
                JavaBinaryOperator::Equal
                    | JavaBinaryOperator::NotEqual
                    | JavaBinaryOperator::Less
                    | JavaBinaryOperator::LessEqual
                    | JavaBinaryOperator::Greater
                    | JavaBinaryOperator::GreaterEqual
            ) && scalar(&left.ty)
                && left.ty == right.ty
                && value.ty == JavaType::primitive(JavaPrimitive::Boolean)
                && value.precedence
                    == match operator {
                        JavaBinaryOperator::Equal | JavaBinaryOperator::NotEqual => {
                            JavaPrecedence::Equality
                        }
                        _ => JavaPrecedence::Relational,
                    } =>
            {
                self.expression(left, depth + 1)?;
                self.expression(right, depth + 1)?;
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } if value.precedence == JavaPrecedence::Conditional => {
                self.expression(condition, depth + 1)?;
                self.expression(when_true, depth + 1)?;
                self.expression(when_false, depth + 1)?;
            }
            JavaExprKind::Call {
                callable,
                receiver: None,
                arguments,
            } => {
                self.call(callable, arguments, depth)?;
            }
            JavaExprKind::New {
                constructor: JavaConstructorRef::Generated { owner, .. },
                arguments,
            } if self.records.contains_key(owner) => {
                for argument in arguments {
                    self.expression(argument, depth + 1)?;
                }
            }
            JavaExprKind::Field {
                receiver,
                field:
                    JavaFieldRef::RustSource {
                        owner,
                        field,
                        name,
                        ty,
                    },
            } => {
                let record = self
                    .records
                    .get(owner)
                    .ok_or("Java dependency field lacks a closed record")?;
                if !record.record_components.iter().any(|component| component.name == *name && component.ty == *ty
                    && matches!(&component.origin, JavaRecordComponentOrigin::RustSource(origin) if origin.origin.declaration == *field)) {
                    return Err("Java dependency field is not an admitted record component".into());
                }
                self.expression(receiver, depth + 1)?;
            }
            _ => return Err("Java dependency body contains an unadmitted expression".into()),
        }
        Ok(())
    }
}
