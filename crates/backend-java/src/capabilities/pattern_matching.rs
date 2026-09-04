//! Java mapping for the complete `PatternMatching` capability.

use portable_build::{CapabilityMapping, PatternMatching};
use portable_core_ir::CoreFieldId;
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaBinaryOperator, JavaBlock, JavaExpr, JavaExprKind, JavaIdentifier, JavaLocalFinality,
        JavaMemberOrigin, JavaPrecedence, JavaPrimitive, JavaStmt, JavaType, JavaUnaryOperator,
    },
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::{binary, bool_literal, identifier, member_call, runtime_call, string_literal, unary},
};

#[doc(hidden)]
pub struct JavaPatternFieldBindingInput {
    pub(crate) binding_name: String,
    pub(crate) binding_type: JavaType,
    pub(crate) field_name: String,
    pub(crate) field_type: JavaType,
    pub(crate) field: CoreFieldId,
}

#[doc(hidden)]
pub enum JavaPatternInput {
    Wildcard,
    Bool {
        matched: Box<JavaExpr>,
        value: bool,
    },
    EnumVariant {
        matched: Box<JavaExpr>,
        variant_type: JavaType,
        variant_name: String,
        bindings: Vec<JavaPatternFieldBindingInput>,
    },
    None {
        matched: Box<JavaExpr>,
    },
    Some {
        matched: Box<JavaExpr>,
        binding_name: String,
        binding_type: JavaType,
    },
    Ok {
        matched: Box<JavaExpr>,
        binding_name: String,
        binding_type: JavaType,
    },
    Err {
        matched: Box<JavaExpr>,
        binding_name: String,
        binding_type: JavaType,
    },
}

#[doc(hidden)]
pub struct JavaLoweredPattern {
    pub(crate) condition: JavaExpr,
    pub(crate) bindings: Vec<JavaStmt>,
}

#[doc(hidden)]
pub struct JavaMatchArmInput {
    pub(crate) pattern: JavaLoweredPattern,
    pub(crate) body: JavaBlock,
}

#[doc(hidden)]
pub struct JavaMatchInput {
    pub(crate) prefix: Vec<JavaStmt>,
    pub(crate) matched: JavaExpr,
    pub(crate) matched_name: JavaIdentifier,
    pub(crate) result_name: JavaIdentifier,
    pub(crate) result_type: JavaType,
    pub(crate) arms: Vec<JavaMatchArmInput>,
}

#[doc(hidden)]
pub struct JavaPatternMatchPlan {
    pub(crate) statements: Vec<JavaStmt>,
    pub(crate) value: JavaExpr,
}

#[doc(hidden)]
pub enum JavaPatternMatchingInput {
    Pattern(Box<JavaPatternInput>),
    Match(Box<JavaMatchInput>),
}

#[doc(hidden)]
pub enum JavaPatternMatchingNode {
    Pattern(Box<JavaLoweredPattern>),
    Match(Box<JavaPatternMatchPlan>),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaPatternMatching;

impl sealed::JavaCapabilityMapping for JavaPatternMatching {}
impl JavaCapabilityMapping for JavaPatternMatching {}

impl CapabilityMapping<JavaDialect> for JavaPatternMatching {
    type Capability = PatternMatching;
    type Context = ();
    type Input = JavaPatternMatchingInput;
    type Output = JavaPatternMatchingNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaPatternMatchingInput::Pattern(input) => {
                JavaPatternMatchingNode::Pattern(Box::new(lower_pattern(*input)))
            }
            JavaPatternMatchingInput::Match(input) => {
                JavaPatternMatchingNode::Match(Box::new(lower_match(*input)))
            }
        })
    }
}

fn lower_pattern(input: JavaPatternInput) -> JavaLoweredPattern {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let (condition, bindings) = match input {
        JavaPatternInput::Wildcard => (bool_literal(true), vec![]),
        JavaPatternInput::Bool { matched, value } => (
            binary(
                JavaBinaryOperator::Equal,
                *matched,
                bool_literal(value),
                boolean,
            ),
            vec![],
        ),
        JavaPatternInput::EnumVariant {
            matched,
            variant_type,
            variant_name,
            bindings,
        } => {
            let variant_name = identifier(&variant_name);
            let condition = JavaExpr {
                ty: boolean,
                precedence: JavaPrecedence::Relational,
                kind: JavaExprKind::InstanceOf {
                    value: matched,
                    target: variant_type.clone(),
                    binding: Some(variant_name.clone()),
                },
            };
            let receiver = JavaExpr::local(variant_type, variant_name);
            let bindings = bindings
                .into_iter()
                .map(|binding| JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: binding.binding_type,
                    name: identifier(&binding.binding_name),
                    value: Some(member_call(
                        receiver.clone(),
                        &binding.field_name,
                        vec![],
                        binding.field_type,
                        JavaMemberOrigin::GeneratedField(binding.field),
                    )),
                })
                .collect();
            (condition, bindings)
        }
        JavaPatternInput::None { matched } => {
            let some = runtime_call(
                JavaRuntimeCallable::OptionIsSome,
                vec![*matched],
                boolean.clone(),
            );
            (unary(JavaUnaryOperator::Not, some, boolean), vec![])
        }
        JavaPatternInput::Some {
            matched,
            binding_name,
            binding_type,
        } => {
            let condition = runtime_call(
                JavaRuntimeCallable::OptionIsSome,
                vec![matched.as_ref().clone()],
                boolean,
            );
            let value = runtime_call(
                JavaRuntimeCallable::OptionValue,
                vec![*matched],
                binding_type.clone(),
            );
            (
                condition,
                vec![JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: binding_type,
                    name: identifier(&binding_name),
                    value: Some(value),
                }],
            )
        }
        JavaPatternInput::Ok {
            matched,
            binding_name,
            binding_type,
        } => lower_result_pattern(*matched, binding_name, binding_type, true, boolean),
        JavaPatternInput::Err {
            matched,
            binding_name,
            binding_type,
        } => lower_result_pattern(*matched, binding_name, binding_type, false, boolean),
    };
    JavaLoweredPattern {
        condition,
        bindings,
    }
}

fn lower_result_pattern(
    matched: JavaExpr,
    binding_name: String,
    binding_type: JavaType,
    success: bool,
    boolean: JavaType,
) -> (JavaExpr, Vec<JavaStmt>) {
    let is_ok = runtime_call(
        JavaRuntimeCallable::ValueResultIsOk,
        vec![matched.clone()],
        boolean.clone(),
    );
    let condition = if success {
        is_ok
    } else {
        unary(JavaUnaryOperator::Not, is_ok, boolean)
    };
    let callable = if success {
        JavaRuntimeCallable::ValueResultValue
    } else {
        JavaRuntimeCallable::ValueResultError
    };
    let value = runtime_call(callable, vec![matched], binding_type.clone());
    (
        condition,
        vec![JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: binding_type,
            name: identifier(&binding_name),
            value: Some(value),
        }],
    )
}

fn lower_match(input: JavaMatchInput) -> JavaPatternMatchPlan {
    let matched_type = input.matched.ty.clone();
    let result_local = JavaExpr::local(input.result_type.clone(), input.result_name.clone());
    let mut statements = input.prefix;
    statements.push(JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: matched_type,
        name: input.matched_name,
        value: Some(input.matched),
    });
    statements.push(JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        ty: input.result_type,
        name: input.result_name,
        value: None,
    });
    let mut otherwise = JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(
        "verified CoreIR match was unexpectedly non-exhaustive",
    ))]);
    for arm in input.arms.into_iter().rev() {
        let mut body = arm.pattern.bindings;
        body.extend(arm.body.statements);
        otherwise = JavaBlock::new(vec![JavaStmt::If {
            condition: arm.pattern.condition,
            then_block: JavaBlock::new(body),
            else_block: Some(otherwise),
        }]);
    }
    statements.extend(otherwise.statements);
    JavaPatternMatchPlan {
        statements,
        value: result_local,
    }
}
