use super::Reader;
use crate::ast::{
    JavaBinaryOperator, JavaBlock, JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind,
    JavaFieldRef, JavaLiteral, JavaStmt, JavaUnaryOperator, JavaValueRef,
};
use portable_codegen::{GeneratedSymbolId, TargetSymbolRef};

impl Reader<'_> {
    pub(super) fn block(&mut self, value: &JavaBlock, depth: usize) -> Result<(), String> {
        self.budget.node(depth)?;
        for statement in &value.statements {
            self.budget.node(depth)?;
            match statement {
                JavaStmt::Local {
                    ty,
                    name,
                    value: Some(value),
                    ..
                } => {
                    self.ty(ty)?;
                    self.spelling(name.as_str())?;
                    self.expression(value, depth + 1)?;
                }
                JavaStmt::Assign { target, value } => {
                    self.expression(target, depth + 1)?;
                    self.expression(value, depth + 1)?;
                }
                JavaStmt::Return(Some(value)) => self.expression(value, depth + 1)?,
                JavaStmt::If {
                    condition,
                    then_block,
                    else_block: Some(else_block),
                } => {
                    self.expression(condition, depth + 1)?;
                    self.block(then_block, depth + 1)?;
                    self.block(else_block, depth + 1)?;
                }
                _ => return Err("source reservation encountered an unsupported statement".into()),
            }
        }
        Ok(())
    }
    pub(super) fn expression(&mut self, value: &JavaExpr, depth: usize) -> Result<(), String> {
        self.budget.node(depth)?;
        match &value.kind {
            JavaExprKind::Literal(
                JavaLiteral::I32(_) | JavaLiteral::I64(_) | JavaLiteral::Boolean(_),
            )
            | JavaExprKind::Value(JavaValueRef::This) => Ok(()),
            JavaExprKind::Value(JavaValueRef::Dependency(imported)) => {
                self.symbol(TargetSymbolRef::DependencyValue(imported.clone()))
            }
            JavaExprKind::Value(JavaValueRef::Local(name)) => self.spelling(name.as_str()),
            JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(id))) => {
                self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Value(*id)))
            }
            JavaExprKind::Unary {
                operator: JavaUnaryOperator::Not | JavaUnaryOperator::BitNot,
                operand,
            } => {
                // The node charge covers punctuation; recurse so operand
                // names and nesting still contribute to the reservation.
                self.expression(operand, depth + 1)
            }
            JavaExprKind::Binary {
                operator:
                    JavaBinaryOperator::Equal
                    | JavaBinaryOperator::NotEqual
                    | JavaBinaryOperator::Less
                    | JavaBinaryOperator::LessEqual
                    | JavaBinaryOperator::Greater
                    | JavaBinaryOperator::GreaterEqual
                    | JavaBinaryOperator::BitAnd
                    | JavaBinaryOperator::BitOr
                    | JavaBinaryOperator::BitXor,
                left,
                right,
            } => {
                self.expression(left, depth + 1)?;
                self.expression(right, depth + 1)
            }
            JavaExprKind::Conditional {
                condition,
                when_true,
                when_false,
            } => {
                self.expression(condition, depth + 1)?;
                self.expression(when_true, depth + 1)?;
                self.expression(when_false, depth + 1)
            }
            JavaExprKind::Call {
                callable,
                receiver: None,
                arguments,
            } => {
                let symbol = match callable {
                    JavaCallableRef::Generated { symbol, .. } => {
                        TargetSymbolRef::Generated(GeneratedSymbolId::Callable(*symbol))
                    }
                    JavaCallableRef::Dependency(value) => {
                        TargetSymbolRef::DependencyCallable(value.clone())
                    }
                    _ => return Err("source reservation encountered an unsupported call".into()),
                };
                self.symbol(symbol)?;
                self.arguments(arguments, depth)
            }
            JavaExprKind::New {
                constructor: JavaConstructorRef::Generated { owner, .. },
                arguments,
            } => {
                self.symbol(TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)))?;
                self.arguments(arguments, depth)
            }
            JavaExprKind::Field {
                receiver,
                field: JavaFieldRef::RustSource { name, .. },
            } => {
                self.spelling(name.as_str())?;
                self.expression(receiver, depth + 1)
            }
            _ => Err("source reservation encountered an unsupported expression".into()),
        }
    }
    fn arguments(&mut self, values: &[JavaExpr], depth: usize) -> Result<(), String> {
        for value in values {
            self.expression(value, depth + 1)?;
        }
        Ok(())
    }
}
