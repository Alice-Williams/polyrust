use super::{ControlInput, LexicalControl, Mapping};
use crate::java_lower::{Place, Reader, Result, TypePlan, Value};
use portable_backend_java::ast::{JavaBlock, JavaExpr, JavaLocalFinality, JavaStmt};
use rustc_ast::BindingMode;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct JavaLexicalControl;

impl Mapping for JavaLexicalControl {
    type Capability = LexicalControl;
    type Context<'tcx> = Reader<'tcx>;
    type Output = JavaBlock;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: ControlInput<'tcx>,
    ) -> Result<JavaBlock> {
        if input.parent != reader.active_scope
            || !reader.prelude.is_empty()
            || reader
                .scopes
                .insert(input.expression.hir_id, input.parent)
                .is_some()
        {
            return Err("inconsistent source lexical scope or undrained evaluation prelude".into());
        }
        let previous_scope = reader.active_scope.replace(input.expression.hir_id);
        let previous_bindings = reader.bindings.clone();
        let result = match input.expression.kind {
            hir::ExprKind::Block(block, None) => {
                Self::block(reader, block, input.expression.hir_id)
            }
            _ => Self::returning(reader, input.expression, input.expression.hir_id),
        };
        reader.active_scope = previous_scope;
        reader.bindings = previous_bindings;
        let statements = match result {
            Ok(statements) => statements,
            Err(error) => {
                // A rejected expression can have a partial local prelude. Do
                // not replace its source diagnostic with an internal drain error.
                reader.prelude.clear();
                return Err(error);
            }
        };
        if !reader.prelude.is_empty() {
            return Err("undrained lexical evaluation prelude".into());
        }
        Ok(JavaBlock::new(statements))
    }
}

impl JavaLexicalControl {
    fn block<'tcx>(
        reader: &mut Reader<'tcx>,
        block: &'tcx hir::Block<'tcx>,
        scope: hir::HirId,
    ) -> Result<Vec<JavaStmt>> {
        let mut statements = Vec::new();
        for statement in block.stmts {
            let hir::StmtKind::Let(local) = statement.kind else {
                return Err("only let statements before a tail result are implemented".into());
            };
            if local.els.is_some() || local.super_.is_some() {
                return Err("let-else and super-let are not implemented".into());
            }
            let hir::PatKind::Binding(BindingMode::NONE, id, _, None) = local.pat.kind else {
                return Err("only plain immutable bindings are implemented".into());
            };
            let init = local
                .init
                .ok_or("uninitialized bindings are not implemented")?;
            let value = reader.initializer(init)?;
            let plan = reader.ty(reader.checked.node_type(local.pat.hir_id))?;
            if value.plan() != &plan {
                return Err("local source representation mismatch".into());
            }
            statements.append(&mut reader.prelude);
            let name = reader.fresh()?;
            let place = Place::resolved(Value::new(
                plan.clone(),
                JavaExpr::local(plan.java_type(), name.clone()),
            )?);
            if reader.bindings.insert(id, place).is_some() {
                return Err("duplicate source local identity".into());
            }
            statements.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: plan.java_type(),
                name,
                value: Some(value.into_expression()),
            });
        }
        statements.extend(Self::returning(
            reader,
            block.expr.ok_or("expected a tail result")?,
            scope,
        )?);
        Ok(statements)
    }

    fn returning<'tcx>(
        reader: &mut Reader<'tcx>,
        expression: &'tcx hir::Expr<'tcx>,
        scope: hir::HirId,
    ) -> Result<Vec<JavaStmt>> {
        let mut statements = Vec::new();
        match expression.kind {
            hir::ExprKind::Block(_, None) => {
                statements.extend(reader.branch(expression, Some(scope))?.statements)
            }
            hir::ExprKind::If(condition, then_value, Some(else_value)) => {
                let condition = reader.expr(condition)?;
                if condition.plan() != &TypePlan::Bool {
                    return Err("branch requires a Boolean condition".into());
                }
                statements.append(&mut reader.prelude);
                let then_block = reader.branch(then_value, Some(scope))?;
                let else_block = reader.branch(else_value, Some(scope))?;
                statements.push(JavaStmt::If {
                    condition: condition.into_expression(),
                    then_block,
                    else_block: Some(else_block),
                });
            }
            _ => {
                let result = reader.expr(expression)?;
                statements.append(&mut reader.prelude);
                statements.push(JavaStmt::Return(Some(result.into_expression())));
            }
        }
        Ok(statements)
    }
}
