//! Java mapping for `ResultValues`.

mod mapping_plan;

use portable_build::{CapabilityMapping, ResultValues};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, JavaValueNode, sealed};
use crate::{
    ast::{JavaExpr, JavaKnownType, JavaType},
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::runtime_call,
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaResultInput {
    Type { ok: JavaType, error: JavaType },
    Ok { value: JavaExpr, result: JavaType },
    Err { value: JavaExpr, result: JavaType },
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaResultValues;

impl sealed::JavaCapabilityMapping for JavaResultValues {}
impl JavaCapabilityMapping for JavaResultValues {
    type Plan = mapping_plan::Plan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        mapping_plan::select(input)
    }
}

impl CapabilityMapping<JavaDialect> for JavaResultValues {
    type Capability = ResultValues;
    type Context = ();
    type Input = JavaResultInput;
    type Output = JavaValueNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let (callable, value, result) = match input {
            JavaResultInput::Type { ok, error } => {
                return Ok(JavaValueNode::Type(JavaType::generic(
                    JavaKnownType::RuntimeValueResult,
                    vec![ok.boxed(), error.boxed()],
                )));
            }
            JavaResultInput::Ok { value, result } => {
                (JavaRuntimeCallable::ValueResultOk, value, result)
            }
            JavaResultInput::Err { value, result } => {
                (JavaRuntimeCallable::ValueResultErr, value, result)
            }
        };
        Ok(JavaValueNode::Expression(Box::new(runtime_call(
            callable,
            vec![value],
            result,
        ))))
    }
}
