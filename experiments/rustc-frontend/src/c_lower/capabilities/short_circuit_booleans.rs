//! Branch-local evaluation, using typed ordinary C locals and statements.
use super::{LazyBooleanInput, LazyBooleanOperator, Mapping, ShortCircuitBooleans};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::*;

#[derive(Clone, Copy)]
pub(crate) struct CShortCircuitBooleans;

impl Mapping for CShortCircuitBooleans {
    type Capability = ShortCircuitBooleans;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: LazyBooleanInput<'tcx>,
    ) -> Result<CValue> {
        #[cfg(lazy_ast_probe)]
        let start = reader.prelude.len();
        let parent = reader
            .active_scope
            .clone()
            .ok_or("lazy Boolean requires a source scope")?;
        let left = reader.expr(input.left())?;
        let key = temporary_key(reader, "lazy")?;
        let local = c(reader.registry.register_local(
            &parent,
            key,
            CObjectType::scalar(CScalarType::Bool),
        ))?;
        let initial = c(reader.expressions().expression_initializer(left))?;
        reader.prelude.push(c(reader
            .statements()?
            .declare(local.clone(), Some(initial)))?);
        let place = c(reader.expressions().local(local))?;
        let result = c(reader.expressions().read(place.clone()))?;
        let condition = match input.operator() {
            LazyBooleanOperator::And => result.clone(),
            LazyBooleanOperator::Or => {
                let not = c(reader
                    .expressions()
                    .unary(CUnaryOperator::LogicalNot, result.clone()))?;
                c(reader
                    .expressions()
                    .numeric_conversion(CScalarType::Bool, not))?
            }
        };
        let taken_key = temporary_key(reader, "lazy_taken")?;
        let taken = c(reader
            .registry
            .register_scope(&reader.function, Some(&parent), taken_key))?;
        let skipped_key = temporary_key(reader, "lazy_skipped")?;
        let skipped =
            c(reader
                .registry
                .register_scope(&reader.function, Some(&parent), skipped_key))?;

        let outer = std::mem::take(&mut reader.prelude);
        let previous_scope = reader.active_scope.replace(taken.clone());
        let right = reader.expr(input.right());
        let mut statements = std::mem::replace(&mut reader.prelude, outer);
        reader.active_scope = previous_scope;
        statements.push(c(reader.statements()?.assign(place, right?))?);
        let taken = c(reader.statements()?.block(taken, statements))?;
        let skipped = c(reader.statements()?.block(skipped, Vec::new()))?;
        reader.prelude.push(c(reader
            .statements()?
            .if_statement(condition, taken, skipped))?);
        #[cfg(lazy_ast_probe)]
        super::lazy_ast::check(reader, &input, start, &result);
        Ok(result)
    }
}

fn temporary_key(reader: &mut Reader<'_>, prefix: &str) -> Result<CDeclarationKey> {
    let index = reader.next_temporary;
    reader.next_temporary = index
        .checked_add(1)
        .ok_or("C evaluation temporary budget exceeded")?;
    Ok(CDeclarationKey {
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary),
        name: c(CIdentifier::new(&format!("{prefix}{index}")))?,
    })
}
