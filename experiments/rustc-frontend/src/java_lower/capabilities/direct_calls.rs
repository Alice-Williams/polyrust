use super::{CallInput, DirectCalls, Mapping};
use crate::java_lower::{Reader, Result, TypePlan, Value, functions};
use portable_backend_java::ast::{JavaCallableRef, JavaExpr, JavaExprKind, JavaPrecedence};
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct JavaDirectCalls;

impl Mapping for JavaDirectCalls {
    type Capability = DirectCalls;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: CallInput<'tcx>) -> Result<Value> {
        let expression = reader.prepare_direct_call(input.0)?;
        let plan = TypePlan::scalar(&expression.ty)?;
        #[cfg(java_ast_probe)]
        super::super::expression_assertions::call(reader, input.0, &expression);
        reader.materialize(Value::new(plan, expression)?)
    }
}

impl<'tcx> Reader<'tcx> {
    pub(super) fn prepare_direct_call(
        &mut self,
        source: &'tcx hir::Expr<'tcx>,
    ) -> Result<JavaExpr> {
        let target = functions::resolve(self.tcx, self.checked, source)?;
        let (callable, signature) = match target.as_local() {
            Some(id) => {
                let function = self
                    .functions
                    .get(&id)
                    .ok_or("direct callee signature was not registered")?;
                (
                    JavaCallableRef::Generated {
                        symbol: function.id,
                        signature: function.signature.clone(),
                    },
                    function.signature.clone(),
                )
            }
            None => {
                let function = self
                    .imported
                    .get(&target)
                    .ok_or("foreign callee certificate was not registered")?;
                (
                    JavaCallableRef::Dependency(function.clone()),
                    function.signature().clone(),
                )
            }
        };
        let hir::ExprKind::Call(_, arguments) = source.kind else {
            unreachable!()
        };
        if arguments.len() != signature.parameters.len() {
            return Err("source call arity mismatch".into());
        }
        let mut values = Vec::new();
        for (argument, ty) in arguments.iter().zip(&signature.parameters) {
            let value = self.expr(argument)?;
            if value.plan() != &TypePlan::scalar(ty)? {
                return Err("source call argument representation mismatch".into());
            }
            values.push(self.materialize(value)?.into_expression());
        }
        Ok(JavaExpr {
            ty: signature.result,
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable,
                receiver: None,
                arguments: values,
            },
        })
    }
}
