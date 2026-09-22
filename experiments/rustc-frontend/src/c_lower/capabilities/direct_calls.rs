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
        let (definition, function) = match target {
            functions::CallTarget::Local(id) => (id.to_def_id(), self.functions.get(&id)),
            functions::CallTarget::Foreign(id) => (id, self.foreign_functions.get(&id)),
        };
        let function = function
            .cloned()
            .ok_or("direct callee signature was not registered")?;
        let hir::ExprKind::Call(_, arguments) = expression.kind else {
            unreachable!()
        };
        let original = self
            .tcx
            .fn_sig(definition)
            .instantiate_identity()
            .skip_binder();
        let signature = function.signature();
        let result_type = self.checked.expr_ty(expression);
        #[cfg(character_source_probe)]
        let result_type = crate::source_origin::types::probe::call_result(self.tcx, result_type);
        if arguments.len() != original.inputs().len()
            || arguments.len() != signature.parameters().len()
            || original.output() != result_type
        {
            return Err("C original source call arity or result type mismatch".into());
        }
        let result_matches = match signature.return_type() {
            CReturnType::Void => original.output().is_unit(),
            CReturnType::Value(result) => self.ty(original.output())? == *result.declared_type(),
        };
        if !result_matches {
            return Err("C source call result representation mismatch".into());
        }
        let mut values = Vec::with_capacity(arguments.len());
        for ((argument, original_type), parameter) in arguments
            .iter()
            .zip(original.inputs())
            .zip(signature.parameters())
        {
            let argument_type = self.checked.expr_ty_adjusted(argument);
            #[cfg(character_source_probe)]
            let argument_type =
                crate::source_origin::types::probe::call_argument(self.tcx, argument_type);
            if argument_type != *original_type {
                return Err("C original source call argument type mismatch".into());
            }
            let expected = self.ty(*original_type)?;
            // Even a pure scalar argument is fixed before the next argument.
            let value = self.expr(argument)?;
            if expected != *parameter.declared_type() || expected != *value.ty() {
                return Err("C source call argument representation mismatch".into());
            }
            values.push(self.materialize(value)?);
        }
        Ok((function, values))
    }

    pub(super) fn materialize(&mut self, value: CValue) -> Result<CValue> {
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
