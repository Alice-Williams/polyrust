//! Java lowering: blocks.

use super::{BlockMode, ExprPlan, Lowering, diagnostic};
use crate::ast::{JavaBlock, JavaLocalFinality, JavaStmt};
use crate::capabilities::{
    JavaLocalBindingsInput, JavaLocalBindingsNode, JavaLoopsInput, JavaLoopsNode,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreBlockId, CoreStatement, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn block(
        &self,
        id: CoreBlockId,
        mode: BlockMode,
        callable_return: CoreTypeId,
    ) -> Result<JavaBlock, Vec<Diagnostic>> {
        let block = self
            .core
            .blocks()
            .get(id)
            .ok_or_else(|| vec![diagnostic("missing CoreIR block")])?;
        let mut statements = Vec::new();
        for statement in &block.statements {
            match statement {
                CoreStatement::Let { local, value, .. } => {
                    let binding = self.core.local(*local).expect("verified local");
                    let plan = self.expr_plan(*value, callable_return)?;
                    statements.extend(plan.statements);
                    let node = self
                        .features
                        .mapping_for::<portable_build::LocalBindings>()
                        .lower(
                            &mut (),
                            JavaLocalBindingsInput::Bind {
                                name: self.names.local(*local).as_str().to_owned(),
                                ty: self.ty(binding.ty)?,
                                value: Box::new(plan.value),
                            },
                        )?;
                    match node {
                        JavaLocalBindingsNode::Statement(statement) => statements.push(*statement),
                        JavaLocalBindingsNode::Expression(_) => {
                            return Err(vec![diagnostic(
                                "Java LocalBindings mapping returned an expression for a binding",
                            )]);
                        }
                    }
                }
                CoreStatement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    let binding_name = self.names.local(*binding).as_str().to_owned();
                    let binding = self.core.local(*binding).expect("verified local");
                    let iterable = self.expr_plan(*iterable, callable_return)?;
                    statements.extend(iterable.statements);
                    let node = self.features.mapping_for::<portable_build::Loops>().lower(
                        &mut (),
                        JavaLoopsInput::ForEach {
                            binding_type: self.ty(binding.ty)?,
                            binding: binding_name,
                            iterable: Box::new(iterable.value),
                            body: self.block(*body, BlockMode::StatementBody, callable_return)?,
                        },
                    )?;
                    match node {
                        JavaLoopsNode::Statement(statement) => statements.push(*statement),
                        JavaLoopsNode::Expression(_) => {
                            return Err(vec![diagnostic(
                                "Java Loops mapping returned an expression for a for-each",
                            )]);
                        }
                    }
                }
                CoreStatement::Return { value, .. } => {
                    let plan = match value {
                        Some(value) => self.expr_plan(*value, callable_return)?,
                        None => ExprPlan::pure(self.lower_unit_value()?),
                    };
                    statements.extend(plan.statements);
                    statements.push(self.lower_function_return(
                        self.success_result(plan.value, callable_return)?,
                    )?);
                }
                CoreStatement::Evaluate { value, .. } => {
                    let plan = self.expr_plan(*value, callable_return)?;
                    self.append_evaluation(&mut statements, plan);
                }
            }
        }
        if let Some(result) = block.result {
            let plan = self.expr_plan(result, callable_return)?;
            match mode {
                BlockMode::ReturnResult => {
                    statements.extend(plan.statements);
                    statements.push(self.lower_function_return(
                        self.success_result(plan.value, callable_return)?,
                    )?);
                }
                BlockMode::AssignResult { ref target } => {
                    statements.extend(plan.statements);
                    statements.push(JavaStmt::Assign {
                        target: target.as_ref().clone(),
                        value: plan.value,
                    });
                }
                BlockMode::StatementBody => self.append_evaluation(&mut statements, plan),
            }
        } else {
            match mode {
                BlockMode::ReturnResult => statements.push(self.lower_function_return(
                    self.success_result(self.lower_unit_value()?, callable_return)?,
                )?),
                BlockMode::AssignResult { target } => statements.push(JavaStmt::Assign {
                    target: *target,
                    value: self.lower_unit_value()?,
                }),
                BlockMode::StatementBody => {}
            }
        }
        Ok(JavaBlock::new(statements))
    }

    fn append_evaluation(&self, statements: &mut Vec<JavaStmt>, mut plan: ExprPlan) {
        statements.append(&mut plan.statements);
        let ty = plan.value.ty.clone();
        let (name, _) = self.temporary("evaluate", ty.clone());
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(plan.value),
        });
    }
}
