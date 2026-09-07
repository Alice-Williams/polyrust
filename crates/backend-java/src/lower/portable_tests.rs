//! Java lowering: portable tests.

use super::{Lowering, diagnostic};
use crate::ast::{JavaExpr, JavaMethodSignature, JavaTypeDeclaration};
use crate::capabilities::{
    JavaPortableFunctionInvocationInput, JavaPortableMethodInvocationInput,
    JavaPortableTestCaseInput, JavaPortableTestExpectation, JavaPortableTestHarnessInput,
    JavaPortableTestsInput, JavaPortableTestsNode,
};
use portable_build::CapabilityMapping;
use portable_core_ir::{CoreTestInvocation, CoreTypedValue};
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn test_declaration(
        &self,
        class_name: &str,
    ) -> Result<JavaTypeDeclaration, Vec<Diagnostic>> {
        let expected_test_count = i32::try_from(self.core.tests().len())
            .map_err(|_| vec![diagnostic("Java conformance test count exceeds i32")])?;
        let mut cases = Vec::with_capacity(self.core.tests().len());
        for (index, test) in self.core.tests().iter().enumerate() {
            let actual = match &test.invocation {
                CoreTestInvocation::Function {
                    function,
                    arguments,
                } => {
                    let function_value = self.core.function(*function).expect("verified function");
                    let arguments = arguments
                        .iter()
                        .map(|value| self.typed_value(value))
                        .collect::<Result<Vec<_>, _>>()?;
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
                    self.lower_portable_test_invocation(
                        JavaPortableTestsInput::FunctionInvocation(
                            JavaPortableFunctionInvocationInput {
                                symbol: self.functions[function],
                                signature,
                                arguments,
                            },
                        ),
                    )?
                }
                CoreTestInvocation::Method {
                    method,
                    receiver,
                    arguments,
                    ..
                } => {
                    let method_value = self
                        .core
                        .implementation_method(*method)
                        .expect("verified method");
                    let interface_method = self
                        .core
                        .interface_method(method_value.interface_method)
                        .expect("verified interface method");
                    let receiver = self.typed_value(receiver)?;
                    let arguments = arguments
                        .iter()
                        .map(|value| self.typed_value(value))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(method_value.return_type)?;
                    self.lower_portable_test_invocation(JavaPortableTestsInput::MethodInvocation(
                        JavaPortableMethodInvocationInput {
                            receiver,
                            method_name: interface_method.header.name.clone(),
                            arguments,
                            result,
                            method: *method,
                        },
                    ))?
                }
            };
            let expected = match &test.expected {
                portable_core_ir::CoreExpectedOutcome::Value(value) => {
                    JavaPortableTestExpectation::Value(self.typed_value(value)?)
                }
                portable_core_ir::CoreExpectedOutcome::Error(value) => {
                    JavaPortableTestExpectation::Error(self.typed_value(value)?)
                }
            };
            let JavaPortableTestsNode::Case(statements) = self
                .features
                .mapping_for::<portable_build::PortableTests>()
                .lower(
                    &mut (),
                    JavaPortableTestsInput::Case(Box::new(JavaPortableTestCaseInput {
                        index,
                        name: test.header.name.clone(),
                        actual,
                        expected,
                    })),
                )?
            else {
                return Err(vec![diagnostic(
                    "Java PortableTests mapping returned a harness for a case",
                )]);
            };
            cases.push(statements);
        }
        match self
            .features
            .mapping_for::<portable_build::PortableTests>()
            .lower(
                &mut (),
                JavaPortableTestsInput::Harness(JavaPortableTestHarnessInput {
                    class_name: class_name.to_owned(),
                    cases,
                    expected_test_count,
                }),
            )? {
            JavaPortableTestsNode::Harness(declaration) => Ok(declaration),
            JavaPortableTestsNode::Case(_) | JavaPortableTestsNode::Expression(_) => {
                Err(vec![diagnostic(
                    "Java PortableTests mapping returned a case for a harness",
                )])
            }
        }
    }

    fn lower_portable_test_invocation(
        &self,
        input: JavaPortableTestsInput,
    ) -> Result<JavaExpr, Vec<Diagnostic>> {
        match self
            .features
            .mapping_for::<portable_build::PortableTests>()
            .lower(&mut (), input)?
        {
            JavaPortableTestsNode::Expression(value) => Ok(value),
            JavaPortableTestsNode::Case(_) | JavaPortableTestsNode::Harness(_) => {
                Err(vec![diagnostic(
                    "Java PortableTests mapping returned a non-expression for an invocation",
                )])
            }
        }
    }

    fn typed_value(&self, value: &CoreTypedValue) -> Result<JavaExpr, Vec<Diagnostic>> {
        self.value(&value.value, value.ty)
    }
}
