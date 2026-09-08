//! Java lowering: intrinsic plans.

use super::{ExprPlan, JavaIntrinsicExpr, Lowering, diagnostic, source};
use crate::ast::JavaExpr;
use crate::capabilities::{JavaBooleanLogicInput, JavaBooleanLogicPlan};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreConstantExpr, CoreExprId, CoreIntrinsicExpr, CoreTypeId,
    CoreUnaryIntrinsic,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode};

impl Lowering<'_> {
    pub(super) fn intrinsic_plan(
        &self,
        value: &CoreIntrinsicExpr<CoreExprId>,
        result: CoreTypeId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        if let CoreIntrinsicExpr::Unary {
            operation: CoreUnaryIntrinsic::BoolNot,
            operand,
        } = value
        {
            let operand = self.stabilize_plan(
                self.expr_plan(*operand, callable_return)?,
                "intrinsicOperand",
            );
            return self.lower_boolean_logic(JavaBooleanLogicInput::Not {
                operand: JavaBooleanLogicPlan {
                    statements: operand.statements,
                    value: operand.value,
                },
                result: self.ty(result)?,
            });
        }
        if let CoreIntrinsicExpr::Binary {
            operation: operation @ (CoreBinaryIntrinsic::BoolAnd | CoreBinaryIntrinsic::BoolOr),
            left,
            right,
        } = value
        {
            return self.short_circuit_boolean(*operation, *left, *right, callable_return);
        }

        let mut statements = Vec::new();
        let mapped = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => {
                let operand = self.stabilize_plan(
                    self.expr_plan(*operand, callable_return)?,
                    "intrinsicOperand",
                );
                statements.extend(operand.statements);
                CoreIntrinsicExpr::Unary {
                    operation: *operation,
                    operand: operand.value,
                }
            }
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => {
                let left = self
                    .stabilize_plan(self.expr_plan(*left, callable_return)?, "intrinsicOperand");
                statements.extend(left.statements);
                let right = self
                    .stabilize_plan(self.expr_plan(*right, callable_return)?, "intrinsicOperand");
                statements.extend(right.statements);
                CoreIntrinsicExpr::Binary {
                    operation: *operation,
                    left: left.value,
                    right: right.value,
                }
            }
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => {
                let first = self
                    .stabilize_plan(self.expr_plan(*first, callable_return)?, "intrinsicOperand");
                statements.extend(first.statements);
                let second = self.stabilize_plan(
                    self.expr_plan(*second, callable_return)?,
                    "intrinsicOperand",
                );
                statements.extend(second.statements);
                let third = self
                    .stabilize_plan(self.expr_plan(*third, callable_return)?, "intrinsicOperand");
                statements.extend(third.statements);
                CoreIntrinsicExpr::Ternary {
                    operation: *operation,
                    first: first.value,
                    second: second.value,
                    third: third.value,
                }
            }
            CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            } => {
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                statements.extend(argument_statements);
                CoreIntrinsicExpr::Variadic {
                    operation: *operation,
                    arguments,
                }
            }
        };
        match self.intrinsic_java(mapped, self.ty(result)?)? {
            JavaIntrinsicExpr::Infallible(value) => Ok(ExprPlan { statements, value }),
            JavaIntrinsicExpr::Fallible { call, value_type } => {
                self.propagate_call(statements, call, value_type, callable_return)
            }
        }
    }

    pub(super) fn constant_intrinsic(
        &self,
        value: &CoreIntrinsicExpr<CoreConstantExpr>,
        result: CoreTypeId,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        let mapped = match value {
            CoreIntrinsicExpr::Unary { operation, operand } => CoreIntrinsicExpr::Unary {
                operation: *operation,
                operand: self.constant_untyped(operand)?.0,
            },
            CoreIntrinsicExpr::Binary {
                operation,
                left,
                right,
            } => CoreIntrinsicExpr::Binary {
                operation: *operation,
                left: self.constant_untyped(left)?.0,
                right: self.constant_untyped(right)?.0,
            },
            CoreIntrinsicExpr::Ternary {
                operation,
                first,
                second,
                third,
            } => CoreIntrinsicExpr::Ternary {
                operation: *operation,
                first: self.constant_untyped(first)?.0,
                second: self.constant_untyped(second)?.0,
                third: self.constant_untyped(third)?.0,
            },
            CoreIntrinsicExpr::Variadic {
                operation,
                arguments,
            } => CoreIntrinsicExpr::Variadic {
                operation: *operation,
                arguments: arguments
                    .iter()
                    .map(|value| self.constant_untyped(value).map(|value| value.0))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        };
        let java_result = self.ty(result)?;
        let boolean_input = match &mapped {
            CoreIntrinsicExpr::Unary {
                operation: CoreUnaryIntrinsic::BoolNot,
                operand,
            } => Some(JavaBooleanLogicInput::Not {
                operand: JavaBooleanLogicPlan {
                    statements: vec![],
                    value: operand.clone(),
                },
                result: java_result.clone(),
            }),
            CoreIntrinsicExpr::Binary {
                operation: CoreBinaryIntrinsic::BoolAnd | CoreBinaryIntrinsic::BoolOr,
                left,
                right,
            } => {
                let (result_name, _) = self.temporary("constantBoolean", java_result.clone());
                let left = JavaBooleanLogicPlan {
                    statements: vec![],
                    value: left.clone(),
                };
                let right = JavaBooleanLogicPlan {
                    statements: vec![],
                    value: right.clone(),
                };
                Some(match &mapped {
                    CoreIntrinsicExpr::Binary {
                        operation: CoreBinaryIntrinsic::BoolAnd,
                        ..
                    } => JavaBooleanLogicInput::And {
                        left,
                        right,
                        result_name,
                        result: java_result.clone(),
                    },
                    CoreIntrinsicExpr::Binary {
                        operation: CoreBinaryIntrinsic::BoolOr,
                        ..
                    } => JavaBooleanLogicInput::Or {
                        left,
                        right,
                        result_name,
                        result: java_result.clone(),
                    },
                    _ => unreachable!("closed boolean constant operation"),
                })
            }
            _ => None,
        };
        if let Some(input) = boolean_input {
            let plan = self.lower_boolean_logic(input)?;
            return if plan.statements.is_empty() {
                Ok(plan.value)
            } else {
                Err(vec![diagnostic(
                    "Java constant boolean mapping unexpectedly required statements",
                )])
            };
        }
        match self.intrinsic_java(mapped, java_result)? {
            JavaIntrinsicExpr::Infallible(value) => Ok(value),
            JavaIntrinsicExpr::Fallible { .. } => Err(vec![Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Java constants cannot contain a fallible intrinsic",
                source("constant-fallible-intrinsic"),
            )]),
        }
    }
}
