use super::*;
use crate::dialect::JavaRuntimeCallable;
use crate::{
    ast::*,
    capabilities as c,
    lower::{bool_literal, runtime_call},
};
use c::boolean_logic::{JavaBooleanLogicInput as I, JavaBooleanLogicPlan};
use c::support::plans::JavaRepresentation as R;

fn operand(prefix: bool, nested_runtime: bool) -> JavaBooleanLogicPlan {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    JavaBooleanLogicPlan {
        statements: if prefix {
            vec![JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: boolean.clone(),
                name: JavaIdentifier::from_portable("prefix"),
                value: Some(bool_literal(true)),
            }]
        } else {
            vec![]
        },
        value: if nested_runtime {
            runtime_call(
                JavaRuntimeCallable::SemanticEqual,
                vec![bool_literal(true), bool_literal(false)],
                boolean,
            )
        } else {
            bool_literal(true)
        },
    }
}
#[test]
fn boolean_plans_distinguish_owned_roots_from_opaque_operands() {
    for prefix in [false, true] {
        let (plan, mut output) = checked(
            c::JavaBooleanLogic,
            I::Not {
                operand: operand(prefix, true),
                result: JavaType::primitive(JavaPrimitive::Boolean),
            },
        );
        assert_eq!(plan.representation(), R::Direct);
        let JavaExprKind::Unary { operand, .. } = &mut output.value.kind else {
            panic!("native not");
        };
        **operand = bool_literal(false);
        assert!(
            plan.verify_output(&output),
            "input operands are opaque holes"
        );
        output.value = bool_literal(true);
        assert!(
            !plan.verify_output(&output),
            "the owned unary root is mandatory"
        );
    }
    for is_or in [false, true] {
        for left_prefix in [false, true] {
            for right_prefix in [false, true] {
                let left = operand(left_prefix, true);
                let right = operand(right_prefix, false);
                let result_name = JavaIdentifier::from_portable("result");
                let result = JavaType::primitive(JavaPrimitive::Boolean);
                let input = if is_or {
                    I::Or {
                        left,
                        right,
                        result_name,
                        result,
                    }
                } else {
                    I::And {
                        left,
                        right,
                        result_name,
                        result,
                    }
                };
                let (plan, mut output) = checked(c::JavaBooleanLogic, input);
                assert_eq!(
                    plan.representation(),
                    if right_prefix {
                        R::StructuredControl
                    } else {
                        R::Direct
                    }
                );
                if right_prefix {
                    let Some(JavaStmt::If { else_block, .. }) = output.statements.last_mut() else {
                        panic!("structured short circuit");
                    };
                    *else_block = None;
                } else {
                    let JavaExprKind::Binary { operator, .. } = &mut output.value.kind else {
                        panic!("native short circuit");
                    };
                    *operator = JavaBinaryOperator::Equal;
                }
                assert!(!plan.verify_output(&output));
            }
        }
    }
}
