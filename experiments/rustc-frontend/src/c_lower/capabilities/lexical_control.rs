//! Scope-owned lets, nested tail blocks, if/else and scalar returns.
use super::{
    ControlCompletion, ControlInput, LexicalControl, LocalConstantInput, LocalConstants, Mapping,
    Supports,
};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::*;
use portable_codegen::RustSourceNode;
use rustc_ast::BindingMode;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct CLexicalControl;
impl Mapping for CLexicalControl {
    type Capability = LexicalControl;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CBlock;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: ControlInput<'tcx>) -> Result<CBlock> {
        let value = input.expression;
        let parent = input
            .parent
            .map(|id| {
                reader
                    .control_scopes
                    .get(&id)
                    .cloned()
                    .ok_or("source parent scope is not registered")
            })
            .transpose()?;
        if reader.control_scopes.contains_key(&value.hir_id) {
            return Err("source control scope is already registered".into());
        }
        let name = format!("scope{}", reader.next_scope);
        reader.next_scope += 1;
        let key = reader.key(
            value.hir_id,
            RustSourceNode::LexicalScope(value.hir_id.local_id.as_u32()),
            &name,
        )?;
        let scope = c(reader
            .registry
            .register_scope(&reader.function, parent.as_ref(), key))?;
        reader.control_scopes.insert(value.hir_id, scope.clone());
        if !reader.prelude.is_empty() {
            return Err("evaluation prelude crossed a lexical boundary".into());
        }
        let previous_scope = reader.active_scope.replace(scope.clone());
        let mut statements = match value.kind {
            hir::ExprKind::Block(block, None) => {
                Self::block(reader, block, &scope, value.hir_id, input.completion)?
            }
            _ => Self::returning(reader, value, value.hir_id, input.completion)?,
        };
        if parent.is_none() {
            // Explicit typed normalization, never a renderer-invented statement.
            let mut uses = Vec::new();
            for parameter in &reader.parameters {
                let place = c(reader.expressions().parameter(parameter.clone()))?;
                let read = c(reader.expressions().read(place))?;
                uses.push(c(reader.statements()?.discard(read))?);
            }
            uses.extend(statements);
            statements = uses;
        }
        if !reader.prelude.is_empty() {
            return Err("undrained lexical evaluation prelude".into());
        }
        reader.active_scope = previous_scope;
        #[cfg(unit_ast_probe)]
        super::unit_effects::ast::completion(reader, &input, &statements);
        c(reader.statements()?.block(scope, statements))
    }
}

impl CLexicalControl {
    fn block<'tcx>(
        reader: &mut Reader<'tcx>,
        block: &'tcx hir::Block<'tcx>,
        scope: &CScopeRef,
        source_scope: hir::HirId,
        completion: ControlCompletion,
    ) -> Result<Vec<CStatement>> {
        let mut result = Vec::new();
        for statement in block.stmts {
            if let hir::StmtKind::Expr(value) | hir::StmtKind::Semi(value) = statement.kind {
                result.extend(reader.unit(value, source_scope)?);
                continue;
            }
            if matches!(statement.kind, hir::StmtKind::Item(_)) {
                let input = LocalConstantInput::read(reader.tcx, statement)?;
                Supports::<LocalConstants>::mapping(&reader.mappings).lower(reader, input)?;
                continue;
            }
            let hir::StmtKind::Let(local) = statement.kind else {
                return Err("only let statements before a tail result are implemented".into());
            };
            if local.els.is_some() || local.super_.is_some() {
                return Err("let-else and super-let are not implemented".into());
            }
            let init = local
                .init
                .ok_or("uninitialized bindings are not implemented")?;
            let initializer = reader.initializer(init)?;
            result.append(&mut reader.prelude);
            let hir::PatKind::Binding(BindingMode::NONE, id, _, None) = local.pat.kind else {
                return Err("only plain immutable bindings are implemented".into());
            };
            let ty = reader.ty(reader.checked.node_type(local.pat.hir_id))?;
            let name = format!("v{}", reader.next_binding);
            reader.next_binding += 1;
            let key = reader.key(id, RustSourceNode::Binding(id.local_id.as_u32()), &name)?;
            let local = c(reader.registry.register_local(scope, key, ty))?;
            let place = c(reader.expressions().local(local.clone()))?;
            reader.bindings.insert(id, place.clone());
            result.push(c(reader.statements()?.declare(local, Some(initializer)))?);
            // An address discard avoids unused-local warnings without observing
            // the initialized value, copying aggregates or changing lifetime.
            let address = c(reader.expressions().address_of(place))?;
            result.push(c(reader.statements()?.discard(address))?);
        }
        if let Some(tail) = block.expr {
            result.extend(Self::returning(reader, tail, source_scope, completion)?);
        } else if completion == ControlCompletion::Return {
            result.push(c(reader.statements()?.return_statement(None))?);
        }
        Ok(result)
    }

    fn returning<'tcx>(
        reader: &mut Reader<'tcx>,
        value: &'tcx hir::Expr<'tcx>,
        source_scope: hir::HirId,
        completion: ControlCompletion,
    ) -> Result<Vec<CStatement>> {
        if matches!(reader.checked.expr_ty(value).kind(), rustc_middle::ty::Tuple(fields) if fields.is_empty())
        {
            let mut statements = reader.unit(value, source_scope)?;
            if completion == ControlCompletion::Return {
                statements.push(c(reader.statements()?.return_statement(None))?);
            }
            return Ok(statements);
        }
        if completion == ControlCompletion::Effect {
            return Err("effect-only block requires a unit result".into());
        }
        let mut prelude = Vec::new();
        let statement = match value.kind {
            hir::ExprKind::Block(_, None) => {
                let block = reader.branch(value, Some(source_scope))?;
                c(reader.statements()?.nested_block(block))?
            }
            hir::ExprKind::If(condition, then_value, Some(else_value)) => {
                let condition = reader.expr(condition)?;
                prelude.append(&mut reader.prelude);
                let then_block = reader.branch(then_value, Some(source_scope))?;
                let else_block = reader.branch(else_value, Some(source_scope))?;
                c(reader
                    .statements()?
                    .if_statement(condition, then_block, else_block))?
            }
            _ => {
                let result = reader.expr(value)?;
                prelude.append(&mut reader.prelude);
                c(reader.statements()?.return_statement(Some(result)))?
            }
        };
        prelude.push(statement);
        Ok(prelude)
    }
}
