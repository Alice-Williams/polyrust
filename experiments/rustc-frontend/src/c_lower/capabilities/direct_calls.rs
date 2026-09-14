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
        let expression = input.0;
        let target = functions::resolve(reader.tcx, reader.checked, expression)?;
        let function = match target {
            functions::CallTarget::Local(id) => reader.functions.get(&id),
            functions::CallTarget::Foreign(id) => reader.foreign_functions.get(&id),
        }
        .cloned()
        .ok_or("direct callee signature was not registered")?;
        let hir::ExprKind::Call(_, arguments) = expression.kind else {
            unreachable!()
        };
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            // Even a pure scalar argument is fixed before the next argument.
            let value = reader.expr(argument)?;
            values.push(reader.materialize(value)?);
        }
        let callable = c(reader.expressions().direct(function))?;
        let call = c(reader.expressions().call_value(callable, values))?;
        reader.materialize(call)
    }
}

impl Reader<'_> {
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
