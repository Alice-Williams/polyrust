//! Lazy evaluation as ordinary typed Java branches, never eager RHS preludes.
use super::{LazyBooleanInput, LazyBooleanOperator, Mapping, ShortCircuitBooleans};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use portable_backend_java::ast::*;

#[derive(Clone, Copy)]
pub(crate) struct JavaShortCircuitBooleans;

impl Mapping for JavaShortCircuitBooleans {
    type Capability = ShortCircuitBooleans;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: LazyBooleanInput<'tcx>,
    ) -> Result<Value> {
        #[cfg(lazy_ast_probe)]
        let start = reader.prelude.len();
        let parent = reader
            .active_scope
            .ok_or("lazy Boolean requires a source scope")?;
        let left = reader.expr(input.left())?;
        if left.plan() != &TypePlan::Bool {
            return Err("lazy Boolean left representation differs".into());
        }
        let name = reader.fresh()?;
        let result = JavaExpr::local(TypePlan::Bool.java_type(), name.clone());
        reader.prelude.push(JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: TypePlan::Bool.java_type(),
            name,
            value: Some(left.into_expression()),
        });
        let condition = match input.operator() {
            LazyBooleanOperator::And => result.clone(),
            LazyBooleanOperator::Or => JavaExpr {
                ty: TypePlan::Bool.java_type(),
                precedence: JavaPrecedence::Unary,
                kind: JavaExprKind::Unary {
                    operator: JavaUnaryOperator::Not,
                    operand: Box::new(result.clone()),
                },
            },
        };
        let scope = input.right().hir_id;
        if reader.scopes.insert(scope, Some(parent)).is_some() {
            return Err("lazy Boolean scope is already registered".into());
        }
        let outer = std::mem::take(&mut reader.prelude);
        let previous_scope = reader.active_scope.replace(scope);
        let right = reader.expr(input.right());
        let mut statements = std::mem::replace(&mut reader.prelude, outer);
        reader.active_scope = previous_scope;
        let right = right?;
        if right.plan() != &TypePlan::Bool {
            return Err("lazy Boolean right representation differs".into());
        }
        statements.push(JavaStmt::Assign {
            target: result.clone(),
            value: right.into_expression(),
        });
        reader.prelude.push(JavaStmt::If {
            condition,
            then_block: JavaBlock::new(statements),
            else_block: Some(JavaBlock::new(Vec::new())),
        });
        #[cfg(lazy_ast_probe)]
        super::lazy_ast::check(reader, &input, start, &result);
        Value::new(TypePlan::Bool, result)
    }
}
