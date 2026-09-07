//! Java lowering: patterns.

use super::{BlockMode, ExprPlan, Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaLocalFinality, JavaStmt, JavaType, JavaTypeName};
use crate::capabilities::{
    JavaEnumBranchInput, JavaEnumsInput, JavaEnumsNode, JavaLoweredPattern, JavaMatchArmInput,
    JavaMatchInput, JavaPatternFieldBindingInput, JavaPatternInput, JavaPatternMatchPlan,
    JavaPatternMatchingInput, JavaPatternMatchingNode,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreEnumId, CoreExprId, CoreMatchArm, CorePattern, CoreType, CoreTypeId};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn match_plan(
        &self,
        value: CoreExprId,
        arms: &[CoreMatchArm],
        result: CoreTypeId,
        callable_return: CoreTypeId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let matched_expression = self
            .core
            .expressions()
            .get(value)
            .expect("verified match expression");
        if let Some(CoreType::Enum(enumeration)) = self.core.types().get(matched_expression.ty)
            && self.enum_is_payload_free(*enumeration)
            && arms
                .iter()
                .all(|arm| matches!(arm.pattern, CorePattern::EnumVariant { .. }))
        {
            return self.payload_free_enum_match_plan(
                value,
                arms,
                result,
                callable_return,
                *enumeration,
            );
        }
        let matched = self.expr_plan(value, callable_return)?;
        let matched_type = matched.value.ty.clone();
        let result_type = self.ty(result)?;
        let (matched_name, matched_local) = self.temporary("matchValue", matched_type.clone());
        let (result_name, result_local) = self.temporary("matchResult", result_type.clone());
        let arms = arms
            .iter()
            .map(|arm| {
                Ok(JavaMatchArmInput {
                    pattern: self.pattern(&arm.pattern, matched_local.clone())?,
                    body: self.block(
                        arm.body,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        match self
            .features
            .mapping_for::<portable_build::PatternMatching>()
            .lower(
                &mut (),
                JavaPatternMatchingInput::Match(Box::new(JavaMatchInput {
                    prefix: matched.statements,
                    matched: matched.value,
                    matched_name,
                    result_name,
                    result_type,
                    arms,
                })),
            )? {
            JavaPatternMatchingNode::Match(plan) => {
                let JavaPatternMatchPlan { statements, value } = *plan;
                Ok(ExprPlan { statements, value })
            }
            JavaPatternMatchingNode::Pattern(_) | JavaPatternMatchingNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java PatternMatching mapping returned a pattern for a match",
                )])
            }
        }
    }

    fn payload_free_enum_match_plan(
        &self,
        value: CoreExprId,
        arms: &[CoreMatchArm],
        result: CoreTypeId,
        callable_return: CoreTypeId,
        enumeration: CoreEnumId,
    ) -> Result<ExprPlan, Vec<Diagnostic>> {
        let matched = self.expr_plan(value, callable_return)?;
        let result_type = self.ty(result)?;
        let (matched_name, matched_local) = self.temporary("matchValue", matched.value.ty.clone());
        let (result_name, result_local) = self.temporary("matchResult", result_type.clone());
        let mut statements = matched.statements;
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: matched.value.ty.clone(),
            name: matched_name,
            value: Some(matched.value),
        });
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: result_type,
            name: result_name,
            value: None,
        });
        let lowered_arms = arms
            .iter()
            .map(|arm| {
                let CorePattern::EnumVariant {
                    variant, bindings, ..
                } = &arm.pattern
                else {
                    unreachable!("payload-free enum match precondition")
                };
                if !bindings.is_empty() {
                    return Err(vec![diagnostic(
                        "payload-free Java enum patterns cannot bind fields",
                    )]);
                }
                Ok(JavaEnumBranchInput {
                    variant: self.enum_values[variant],
                    body: self.block(
                        arm.body,
                        BlockMode::AssignResult {
                            target: Box::new(result_local.clone()),
                        },
                        callable_return,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
        let enumeration_value = self.core.enumeration(enumeration).expect("verified enum");
        let branch = self.features.mapping_for::<portable_build::Enums>().lower(
            &mut (),
            JavaEnumsInput::Branch {
                selector: Box::new(matched_local),
                enumeration: self.enums[&enumeration],
                declared_variants: enumeration_value
                    .variants
                    .iter()
                    .map(|variant| self.enum_values[variant])
                    .collect(),
                arms: lowered_arms,
            },
        )?;
        match branch {
            JavaEnumsNode::Statement(branch) => statements.push(*branch),
            JavaEnumsNode::Type(_)
            | JavaEnumsNode::Declaration(_)
            | JavaEnumsNode::Expression(_) => {
                return Err(vec![diagnostic(
                    "Java Enums mapping returned a value for exhaustive branching",
                )]);
            }
        }
        Ok(ExprPlan {
            statements,
            value: result_local,
        })
    }

    pub(super) fn pattern(
        &self,
        pattern: &CorePattern,
        matched: JavaExpr,
    ) -> Result<JavaLoweredPattern, Vec<Diagnostic>> {
        let input = match pattern {
            CorePattern::Wildcard { .. } => JavaPatternInput::Wildcard,
            CorePattern::Bool { value, .. } => JavaPatternInput::Bool {
                matched: Box::new(matched),
                value: *value,
            },
            CorePattern::EnumVariant {
                variant, bindings, ..
            } => {
                let variant_type =
                    JavaType::Reference(JavaTypeName::Generated(self.variants[variant]));
                let bindings = bindings
                    .iter()
                    .map(|binding| {
                        let local_value = self
                            .core
                            .local(binding.binding)
                            .expect("verified pattern local");
                        let field = self.core.field(binding.field).expect("verified field");
                        Ok(JavaPatternFieldBindingInput {
                            binding_name: self.names.local(binding.binding).as_str().to_owned(),
                            binding_type: self.ty(local_value.ty)?,
                            field_name: self.names.field(binding.field).as_str().to_owned(),
                            field_type: self.ty(field.ty)?,
                            field: binding.field,
                        })
                    })
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?;
                let (variant_name, _) = self.temporary("matchedVariant", variant_type.clone());
                JavaPatternInput::EnumVariant {
                    matched: Box::new(matched),
                    variant_type,
                    variant_name: variant_name.as_str().to_owned(),
                    bindings,
                }
            }
            CorePattern::None { .. } => JavaPatternInput::None {
                matched: Box::new(matched),
            },
            CorePattern::Some { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Some {
                    matched: Box::new(matched),
                    binding_name: self.names.local(*binding).as_str().to_owned(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
            CorePattern::Ok { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Ok {
                    matched: Box::new(matched),
                    binding_name: self.names.local(*binding).as_str().to_owned(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
            CorePattern::Err { binding, .. } => {
                let local_value = self.core.local(*binding).expect("verified pattern local");
                JavaPatternInput::Err {
                    matched: Box::new(matched),
                    binding_name: self.names.local(*binding).as_str().to_owned(),
                    binding_type: self.ty(local_value.ty)?,
                }
            }
        };
        match self
            .features
            .mapping_for::<portable_build::PatternMatching>()
            .lower(&mut (), JavaPatternMatchingInput::Pattern(Box::new(input)))?
        {
            JavaPatternMatchingNode::Pattern(pattern) => Ok(*pattern),
            JavaPatternMatchingNode::Match(_) | JavaPatternMatchingNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java PatternMatching mapping returned a match for a pattern",
                )])
            }
        }
    }
}
