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
        let target = functions::resolve(reader.tcx, reader.checked, input.0)?;
        let (callable, signature) = match target.as_local() {
            Some(id) => {
                let function = reader
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
                let function = reader
                    .imported
                    .get(&target)
                    .ok_or("foreign callee certificate was not registered")?;
                (
                    JavaCallableRef::Dependency(function.clone()),
                    function.signature().clone(),
                )
            }
        };
        let hir::ExprKind::Call(_, arguments) = input.0.kind else {
            unreachable!()
        };
        if arguments.len() != signature.parameters.len() {
            return Err("source call arity mismatch".into());
        }
        let mut values = Vec::new();
        for (argument, ty) in arguments.iter().zip(&signature.parameters) {
            let value = reader.expr(argument)?;
            if value.plan() != &TypePlan::scalar(ty)? {
                return Err("source call argument representation mismatch".into());
            }
            values.push(reader.materialize(value)?.into_expression());
        }
        let plan = TypePlan::scalar(&signature.result)?;
        let expression = JavaExpr {
            ty: plan.java_type(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable,
                receiver: None,
                arguments: values,
            },
        };
        #[cfg(java_ast_probe)]
        super::super::expression_assertions::call(reader, input.0, &expression);
        reader.materialize(Value::new(plan, expression)?)
    }
}
