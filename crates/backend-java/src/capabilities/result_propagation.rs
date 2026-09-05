//! Java mapping for the complete `ResultPropagation` capability.

use portable_build::{CapabilityMapping, ResultPropagation};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaBlock, JavaExpr, JavaIdentifier, JavaKnownType, JavaLocalFinality, JavaMemberOrigin,
        JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType,
    },
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::{member_call, runtime_call, unary},
};

#[doc(hidden)]
pub struct JavaResultPropagationInput {
    pub(crate) prefix: Vec<JavaStmt>,
    pub(crate) call: JavaExpr,
    pub(crate) result_name: JavaIdentifier,
    pub(crate) value_type: JavaType,
    pub(crate) callable_result_type: JavaType,
}

#[doc(hidden)]
pub struct JavaResultPropagationPlan {
    pub(crate) statements: Vec<JavaStmt>,
    pub(crate) value: JavaExpr,
}

impl sealed::JavaMappingOutput for JavaResultPropagationPlan {}
impl super::support::JavaMappingOutput for JavaResultPropagationPlan {}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaResultPropagation;

impl sealed::JavaCapabilityMapping for JavaResultPropagation {}
impl JavaCapabilityMapping for JavaResultPropagation {}

impl CapabilityMapping<JavaDialect> for JavaResultPropagation {
    type Capability = ResultPropagation;
    type Context = ();
    type Input = JavaResultPropagationInput;
    type Output = JavaResultPropagationPlan;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let result_type = input.call.ty.clone();
        let result_name = input.result_name;
        let result = JavaExpr::local(result_type.clone(), result_name.clone());
        let mut statements = input.prefix;
        statements.push(JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: result_type,
            name: result_name,
            value: Some(input.call),
        });
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let ok = member_call(
            result.clone(),
            "ok",
            vec![],
            boolean.clone(),
            JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultOk),
        );
        let error = member_call(
            result.clone(),
            "error",
            vec![],
            JavaType::known(JavaKnownType::RuntimeError),
            JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultError),
        );
        let string = JavaType::known(JavaKnownType::String);
        let failure = runtime_call(
            JavaRuntimeCallable::Fail,
            vec![
                member_call(
                    error.clone(),
                    "code",
                    vec![],
                    string.clone(),
                    JavaMemberOrigin::Runtime(JavaRuntimeMember::ErrorCode),
                ),
                member_call(
                    error,
                    "message",
                    vec![],
                    string,
                    JavaMemberOrigin::Runtime(JavaRuntimeMember::ErrorMessage),
                ),
            ],
            input.callable_result_type,
        );
        statements.push(JavaStmt::If {
            condition: unary(crate::ast::JavaUnaryOperator::Not, ok, boolean),
            then_block: JavaBlock::new(vec![JavaStmt::Return(Some(failure))]),
            else_block: None,
        });
        Ok(JavaResultPropagationPlan {
            statements,
            value: member_call(
                result,
                "value",
                vec![],
                input.value_type,
                JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultValue),
            ),
        })
    }
}
