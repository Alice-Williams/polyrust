//! Owned or certified foreign calls, in Rust source evaluation order.
use super::{CallInput, DirectCalls, Mapping};
use crate::c_lower::{Reader, Result, c, functions};
use portable_backend_c::ast::*;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct CDirectCalls;
impl Mapping for CDirectCalls {
    type Capability = DirectCalls;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: CallInput<'tcx>) -> Result<CValue> {
        let (function, values) = reader.prepare_direct_call(input.0)?;
        let callable = c(reader.expressions().direct(function))?;
        let call = c(reader.expressions().call_value(callable, values))?;
        reader.materialize(call)
    }
}

impl<'tcx> Reader<'tcx> {
    pub(super) fn prepare_direct_call(
        &mut self,
        expression: &'tcx hir::Expr<'tcx>,
    ) -> Result<(CFunctionRef, Vec<CValue>)> {
        let target = functions::resolve(self.tcx, self.checked, expression)?;
        let function = match target {
            functions::CallTarget::Local(id) => self.functions.get(&id),
            functions::CallTarget::Foreign(id) => self.foreign_functions.get(&id),
        }
        .cloned()
        .ok_or("direct callee signature was not registered")?;
        let hir::ExprKind::Call(_, arguments) = expression.kind else {
            unreachable!()
        };
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            // Even a pure scalar argument is fixed before the next argument.
            let value = self.expr(argument)?;
            values.push(self.materialize(value)?);
        }
        Ok((function, values))
    }

    fn materialize(&mut self, value: CValue) -> Result<CValue> {
        let scope = self
            .active_scope
            .clone()
            .ok_or("call requires an active lexical scope")?;
        let key = CDeclarationKey {
            origin: CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary),
            name: c(CIdentifier::new(&format!("eval{}", self.next_temporary)))?,
        };
        self.next_temporary += 1;
        let local = c(self
            .registry
            .register_local(&scope, key, value.ty().clone()))?;
        let initializer = c(self.expressions().expression_initializer(value))?;
        let declaration = c(self.statements()?.declare(local.clone(), Some(initializer)))?;
        self.prelude.push(declaration);
        let place = c(self.expressions().local(local))?;
        c(self.expressions().read(place))
    }
}
