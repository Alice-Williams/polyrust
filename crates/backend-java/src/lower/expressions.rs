//! Java lowering: expressions.

use super::{BlockMode, ExprPlan, Lowering, diagnostic};
use crate::ast::{
    JavaCallableRef, JavaInterfaceWitness, JavaLocalFinality, JavaMemberOrigin,
    JavaMethodSignature, JavaStmt, JavaType, JavaTypeName,
};
use crate::capabilities::{
    JavaConcreteInterfaceCallInput, JavaConditionalValueInput, JavaConditionalsInput,
    JavaConditionalsNode, JavaConstantsInput, JavaConstantsNode, JavaEnumsInput,
    JavaFunctionsInput, JavaInterfaceCallInput, JavaInterfacesInput, JavaListInput,
    JavaLocalBindingsInput, JavaLocalBindingsNode, JavaLoopsInput, JavaLoopsNode, JavaOptionInput,
    JavaPatternMatchingInput, JavaPatternMatchingNode, JavaRecordsInput, JavaResultInput,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreExprId, CoreExprKind, CoreLocalKind, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn expr_plan(
        &self,
        id: CoreExprId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let expression = self
            .core
            .expressions()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR expression")])?;
        let ty = self.ty(expression.ty)?;
        match &expression.kind {
            CoreExprKind::Literal(value) => Ok(ExprPlan::pure(self.value(value, expression.ty)?)),
            CoreExprKind::Local(id) => {
                let local_value = self.core.local(*id).expect("verified local");
                let name = self.names.local(*id).as_str().to_owned();
                let value = match local_value.kind {
                    CoreLocalKind::Parameter => {
                        self.lower_function_expr(JavaFunctionsInput::ParameterRead { ty, name })?
                    }
                    CoreLocalKind::Let => match self
                        .features
                        .mapping_for::<portable_build::LocalBindings>()
                        .lower(&mut (), JavaLocalBindingsInput::Read { name, ty })?
                    {
                        JavaLocalBindingsNode::Expression(value) => *value,
                        JavaLocalBindingsNode::Statement(_) => {
                            return Err(vec![diagnostic(
                                "Java LocalBindings mapping returned a statement for a read",
                            )]);
                        }
                    },
                    CoreLocalKind::ForEach => {
                        match self.features.mapping_for::<portable_build::Loops>().lower(
                            &mut (),
                            JavaLoopsInput::BindingRead {
                                binding_type: ty,
                                binding: name,
                            },
                        )? {
                            JavaLoopsNode::Expression(value) => *value,
                            JavaLoopsNode::Statement(_) => {
                                return Err(vec![diagnostic(
                                    "Java Loops mapping returned a statement for a binding read",
                                )]);
                            }
                        }
                    }
                    CoreLocalKind::Pattern => match self
                        .features
                        .mapping_for::<portable_build::PatternMatching>()
                        .lower(
                            &mut (),
                            JavaPatternMatchingInput::BindingRead {
                                binding_type: ty,
                                binding: name,
                            },
                        )? {
                        JavaPatternMatchingNode::Expression(value) => *value,
                        JavaPatternMatchingNode::Pattern(_) | JavaPatternMatchingNode::Match(_) => {
                            return Err(vec![diagnostic(
                                "Java PatternMatching mapping returned a non-expression for a binding read",
                            )]);
                        }
                    },
                };
                Ok(ExprPlan::pure(value))
            }
            CoreExprKind::Constant(id) => match self
                .features
                .mapping_for::<portable_build::Constants>()
                .lower(
                    &mut (),
                    JavaConstantsInput::Reference {
                        symbol: self.constants[id],
                        result: ty,
                    },
                )? {
                JavaConstantsNode::Expression(value) => Ok(ExprPlan::pure(value)),
                JavaConstantsNode::Declaration(_) => Err(vec![diagnostic(
                    "Java Constants mapping returned a declaration for a reference",
                )]),
            },
            CoreExprKind::SelfValue(id) => Ok(ExprPlan::pure(self.lower_interface_expr(
                JavaInterfacesInput::SelfValue {
                    record: self.records[id],
                },
            )?)),
            CoreExprKind::ConstructRecord { record, fields } => {
                self.construct_generated_plan(self.records[record], fields, ty, callable_return)
            }
            CoreExprKind::ConstructEnum {
                enumeration,
                variant,
                fields,
            } if self.enum_is_payload_free(*enumeration) => {
                if !fields.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum construction cannot contain fields",
                    )]);
                }
                Ok(ExprPlan::pure(
                    self.enum_variant_expr(*enumeration, *variant)?,
                ))
            }
            CoreExprKind::ConstructEnum {
                variant, fields, ..
            } => {
                let ids = fields.iter().map(|field| field.value).collect::<Vec<_>>();
                let (statements, arguments) = self.expr_list(&ids, callable_return)?;
                let value = self.lower_enum_expr(JavaEnumsInput::PayloadConstruction {
                    variant: self.variants[variant],
                    arguments,
                    result: ty,
                })?;
                Ok(ExprPlan { statements, value })
            }
            CoreExprKind::ConstructSome(value) => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.value_expression(
                    self.features
                        .mapping_for::<portable_build::OptionValues>()
                        .lower(
                            &mut (),
                            JavaOptionInput::Some {
                                value: Box::new(plan.value),
                                result: ty,
                            },
                        )?,
                    "OptionValues construction",
                )?;
                Ok(plan)
            }
            CoreExprKind::ConstructNone { .. } => {
                let value = self.value_expression(
                    self.features
                        .mapping_for::<portable_build::OptionValues>()
                        .lower(&mut (), JavaOptionInput::None { result: ty })?,
                    "OptionValues construction",
                )?;
                Ok(ExprPlan::pure(value))
            }
            CoreExprKind::ConstructOk { value, .. } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Ok {
                                value: plan.value,
                                result: ty,
                            },
                        )?,
                    "ResultValues construction",
                )?;
                Ok(plan)
            }
            CoreExprKind::ConstructErr { value, .. } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ResultValues>()
                        .lower(
                            &mut (),
                            JavaResultInput::Err {
                                value: plan.value,
                                result: ty,
                            },
                        )?,
                    "ResultValues construction",
                )?;
                Ok(plan)
            }
            CoreExprKind::ConstructList { elements, .. } => {
                let (statements, values) = self.expr_list(elements, callable_return)?;
                let value = self.value_expression(
                    self.features
                        .mapping_for::<portable_build::ListValues>()
                        .lower(
                            &mut (),
                            JavaListInput::Value {
                                elements: values,
                                result: ty,
                            },
                        )?,
                    "ListValues construction",
                )?;
                Ok(ExprPlan { statements, value })
            }
            CoreExprKind::CoerceInterface {
                implementation,
                value,
            } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                let conformance = self
                    .core
                    .implementation(*implementation)
                    .expect("verified conformance");
                let witness = JavaInterfaceWitness::from_checked(
                    self.core,
                    *implementation,
                    self.records[&conformance.record],
                    self.interfaces[&conformance.interface],
                );
                plan.value = self.lower_interface_expr(JavaInterfacesInput::Coerce {
                    implementation: witness,
                    value: Box::new(plan.value),
                    result: ty,
                })?;
                Ok(plan)
            }
            CoreExprKind::Field { value, field } => {
                let mut plan = self.expr_plan(*value, callable_return)?;
                plan.value = self.lower_record_expr(JavaRecordsInput::Field {
                    receiver: Box::new(plan.value),
                    name: self.names.field(*field).as_str().to_owned(),
                    result: ty,
                    origin: JavaMemberOrigin::GeneratedField(*field),
                })?;
                Ok(plan)
            }
            CoreExprKind::Call {
                function,
                arguments,
            } => {
                let function_value = self.core.function(*function).expect("verified function");
                let (statements, arguments) = self.expr_list(arguments, callable_return)?;
                let result = self.poly_result_type(function_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: None,
                    parameters: function_value
                        .parameters
                        .iter()
                        .map(|parameter| self.ty(parameter.ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = self.lower_function_expr(JavaFunctionsInput::Call {
                    result,
                    callable: Box::new(JavaCallableRef::Generated {
                        symbol: self.functions[function],
                        signature,
                    }),
                    arguments,
                })?;
                self.propagate_call(statements, call, ty, callable_return)
            }
            CoreExprKind::StaticMethodCall {
                method,
                receiver,
                arguments,
                ..
            } => {
                let method_value = self
                    .core
                    .implementation_method(*method)
                    .expect("verified method");
                let mut receiver =
                    self.stabilize_plan(self.expr_plan(*receiver, callable_return)?, "receiver");
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                receiver.statements.extend(argument_statements);
                let result = self.poly_result_type(method_value.return_type)?;
                let call = self.lower_interface_expr(JavaInterfacesInput::ConcreteCall(
                    Box::new(JavaConcreteInterfaceCallInput {
                        receiver: receiver.value,
                        interface_method_name: self
                            .names
                            .method(method_value.interface_method)
                            .as_str()
                            .to_owned(),
                        arguments,
                        result,
                        method: *method,
                    }),
                ))?;
                self.propagate_call(receiver.statements, call, ty, callable_return)
            }
            CoreExprKind::InterfaceCall {
                method,
                receiver,
                arguments,
                ..
            } => {
                let method_value = self
                    .core
                    .interface_method(*method)
                    .expect("verified interface method");
                let mut receiver =
                    self.stabilize_plan(self.expr_plan(*receiver, callable_return)?, "receiver");
                let (argument_statements, arguments) =
                    self.expr_list(arguments, callable_return)?;
                receiver.statements.extend(argument_statements);
                let result = self.poly_result_type(method_value.return_type)?;
                let signature = JavaMethodSignature {
                    receiver: Some(JavaType::Reference(JavaTypeName::Generated(
                        self.interfaces[&method_value.interface],
                    ))),
                    parameters: method_value
                        .parameters
                        .iter()
                        .map(|parameter| self.ty(parameter.ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    result: result.clone(),
                    checked_exceptions: vec![],
                    nullable_result: false,
                    pure: true,
                };
                let call = self.lower_interface_expr(JavaInterfacesInput::InterfaceCall(
                    Box::new(JavaInterfaceCallInput {
                        receiver: receiver.value,
                        arguments,
                        result,
                        symbol: self.interface_methods[method],
                        signature,
                    }),
                ))?;
                self.propagate_call(receiver.statements, call, ty, callable_return)
            }
            CoreExprKind::Intrinsic(value) => {
                self.intrinsic_plan(value, expression.ty, callable_return)
            }
            CoreExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition = self.expr_plan(*condition, callable_return)?;
                let (name, result_local) = self.temporary("ifResult", ty.clone());
                match self
                    .features
                    .mapping_for::<portable_build::Conditionals>()
                    .lower(
                        &mut (),
                        JavaConditionalsInput::Value(Box::new(JavaConditionalValueInput {
                            prefix: condition.statements,
                            condition: condition.value,
                            result_name: name,
                            result_type: ty,
                            then_block: self.block(
                                *then_block,
                                BlockMode::AssignResult {
                                    target: Box::new(result_local.clone()),
                                },
                                callable_return,
                            )?,
                            else_block: self.block(
                                *else_block,
                                BlockMode::AssignResult {
                                    target: Box::new(result_local),
                                },
                                callable_return,
                            )?,
                        })),
                    )? {
                    JavaConditionalsNode::Value { statements, value } => Ok(ExprPlan {
                        statements,
                        value: *value,
                    }),
                }
            }
            CoreExprKind::Match { value, arms } => {
                self.match_plan(*value, arms, expression.ty, callable_return)
            }
            CoreExprKind::Block(block) => {
                let (name, result_local) = self.temporary("blockResult", ty.clone());
                let mut statements = vec![JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty,
                    name,
                    value: None,
                }];
                statements.extend(
                    self.block(
                        *block,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?
                    .statements,
                );
                Ok(ExprPlan {
                    statements,
                    value: result_local,
                })
            }
        }
    }
}
