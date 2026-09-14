use super::{ComparisonInput, Mapping, ScalarComparisons};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{
    JavaBinaryOperator as Op, JavaExpr, JavaExprKind, JavaLiteral, JavaPrecedence,
};
use rustc_hir as hir;
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct JavaScalarComparisons;

impl Mapping for JavaScalarComparisons {
    type Capability = ScalarComparisons;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: ComparisonInput<'tcx>,
    ) -> Result<Value> {
        let expression = input.0;
        let hir::ExprKind::Binary(operator, left, right) = expression.kind else {
            return Err("comparison requires a binary source expression".into());
        };
        if reader
            .checked
            .type_dependent_def_id(expression.hir_id)
            .is_some()
        {
            return Err("overloaded operators are not implemented".into());
        }
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("comparison compiler adjustment is not implemented".into());
        }
        for operand in [left, right] {
            if !matches!(
                reader.checked.expr_ty_adjusted(operand).kind(),
                ty::Int(ty::IntTy::I32) | ty::Bool
            ) {
                return Err("only scalar comparisons are implemented".into());
            }
        }
        let (operator, precedence, ordering) = match operator.node {
            hir::BinOpKind::Eq => (Op::Equal, JavaPrecedence::Equality, false),
            hir::BinOpKind::Ne => (Op::NotEqual, JavaPrecedence::Equality, false),
            hir::BinOpKind::Lt => (Op::Less, JavaPrecedence::Relational, true),
            hir::BinOpKind::Le => (Op::LessEqual, JavaPrecedence::Relational, true),
            hir::BinOpKind::Gt => (Op::Greater, JavaPrecedence::Relational, true),
            hir::BinOpKind::Ge => (Op::GreaterEqual, JavaPrecedence::Relational, true),
            _ => return Err("only comparison binary operators are implemented".into()),
        };
        let left = reader.expr(left)?;
        let left = reader.materialize(left)?;
        let right = reader.expr(right)?;
        let right = reader.materialize(right)?;
        if left.plan() != right.plan() {
            return Err("comparison operand representations differ".into());
        }
        let convert = ordering && left.plan() == &TypePlan::Bool;
        let left = operand(left, convert);
        let right = operand(right, convert);
        let result = Value::new(
            TypePlan::Bool,
            JavaExpr {
                ty: TypePlan::Bool.java_type(),
                precedence,
                kind: JavaExprKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            },
        )?;
        #[cfg(java_ast_probe)]
        super::super::expression_assertions::comparison(reader, expression, &result);
        Ok(result)
    }
}

fn operand(value: Value, convert: bool) -> JavaExpr {
    if !convert {
        return value.into_expression();
    }
    JavaExpr {
        ty: TypePlan::I32.java_type(),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(value.into_expression()),
            when_true: Box::new(JavaExpr::literal(
                TypePlan::I32.java_type(),
                JavaLiteral::I32(1),
            )),
            when_false: Box::new(JavaExpr::literal(
                TypePlan::I32.java_type(),
                JavaLiteral::I32(0),
            )),
        },
    }
}
