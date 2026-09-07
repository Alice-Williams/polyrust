//! Java lowering: evaluation plans.

use super::call_builders::runtime_call;
use super::{ExprPlan, Lowering};
use crate::ast::{
    JavaExpr, JavaExprKind, JavaIdentifier, JavaLocalFinality, JavaPrimitive, JavaStmt, JavaType,
};
use crate::capabilities::{
    JavaBooleanLogicInput, JavaBooleanLogicPlan, JavaResultPropagationInput,
    JavaResultPropagationPlan,
};
use crate::dialect::JavaRuntimeCallable;
use portable_build::{BooleanLogic, CapabilityMapping};
use portable_core_ir::{CoreBinaryIntrinsic, CoreExprId, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn expr_list(
        &self,
        values: &[CoreExprId],
        callable_return: CoreTypeId,
    ) -> Result<(Vec<JavaStmt>, Vec<JavaExpr>), Vec<Diagnostic>> {
        let mut statements = Vec::new();
        let mut expressions = Vec::with_capacity(values.len());
        for value in values {
            let plan = self.stabilize_plan(self.expr_plan(*value, callable_return)?, "argument");
            statements.extend(plan.statements);
            expressions.push(plan.value);
        }
        Ok((statements, expressions))
    }

    pub(super) fn stabilize_plan(&self, mut plan: ExprPlan, prefix: &str) -> ExprPlan {
        if matches!(
            &plan.value.kind,
            JavaExprKind::Literal(_) | JavaExprKind::Value(_)
        ) {
            return plan;
        }
        let ty = plan.value.ty.clone();
        let (name, value) = self.temporary(prefix, ty.clone());
        plan.statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(plan.value),
        });
        plan.value = value;
        plan
    }

    pub(super) fn temporary(&self, prefix: &str, ty: JavaType) -> (JavaIdentifier, JavaExpr) {
        let index = self.next_temporary.get();
        self.next_temporary.set(index + 1);
        let name = JavaIdentifier::new(format!("__polyrust_{prefix}_{index}"))
            .expect("internal Java temporary identifier is valid");
        let value = JavaExpr::local(ty, name.clone());
        (name, value)
    }

    pub(super) fn success_result(
        &self,
        value: JavaExpr,
        callable_return: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        Ok(runtime_call(
            JavaRuntimeCallable::Ok,
            vec![value],
            self.poly_result_type(callable_return)?,
        ))
    }

    pub(super) fn propagate_call(
        &self,
        statements: Vec<JavaStmt>,
        call: JavaExpr,
        value_type: JavaType,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let (name, _) = self.temporary("callResult", call.ty.clone());
        let JavaResultPropagationPlan { statements, value } = self
            .features
            .mapping_for::<portable_build::ResultPropagation>()
            .lower(
                &mut (),
                JavaResultPropagationInput {
                    prefix: statements,
                    call,
                    result_name: name,
                    value_type,
                    callable_result_type: self.poly_result_type(callable_return)?,
                },
            )?;
        Ok(ExprPlan { statements, value })
    }

    pub(super) fn short_circuit_boolean(
        &self,
        operation: CoreBinaryIntrinsic,
        left: CoreExprId,
        right: CoreExprId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let left = self.stabilize_plan(self.expr_plan(left, callable_return)?, "intrinsicOperand");
        let right =
            self.stabilize_plan(self.expr_plan(right, callable_return)?, "intrinsicOperand");
        let (result_name, _) = self.temporary("booleanResult", boolean.clone());
        let left = JavaBooleanLogicPlan {
            statements: left.statements,
            value: left.value,
        };
        let right = JavaBooleanLogicPlan {
            statements: right.statements,
            value: right.value,
        };
        self.lower_boolean_logic(match operation {
            CoreBinaryIntrinsic::BoolAnd => JavaBooleanLogicInput::And {
                left,
                right,
                result_name,
                result: boolean,
            },
            CoreBinaryIntrinsic::BoolOr => JavaBooleanLogicInput::Or {
                left,
                right,
                result_name,
                result: boolean,
            },
            _ => unreachable!("short-circuit helper only accepts boolean operations"),
        })
    }

    pub(super) fn lower_boolean_logic(
        &self,
        input: JavaBooleanLogicInput,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let JavaBooleanLogicPlan { statements, value } = self
            .features
            .mapping_for::<BooleanLogic>()
            .lower(&mut (), input)?;
        Ok(ExprPlan { statements, value })
    }
}
