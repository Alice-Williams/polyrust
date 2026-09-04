//! Java mapping for the complete `PortableTests` capability.

use portable_build::{CapabilityMapping, PortableTests};
use portable_diagnostics::Diagnostic;

use super::support::{JavaCapabilityMapping, sealed};
use crate::{
    ast::{
        JavaArrayOwnership, JavaBlock, JavaDeclarationKind, JavaExpr, JavaHeritage, JavaKnownType,
        JavaLocalFinality, JavaMember, JavaMemberOrigin, JavaMethod, JavaMethodDeclaration,
        JavaModifier, JavaParameter, JavaPrimitive, JavaRuntimeMember, JavaStmt, JavaType,
        JavaTypeDeclaration, JavaUnaryOperator, JavaVisibility,
    },
    dialect::{JavaDialect, JavaRuntimeCallable},
    lower::{
        binary, i32_literal, identifier, member_call, private_constructor, runtime_call,
        string_literal, unary,
    },
};

#[doc(hidden)]
pub enum JavaPortableTestExpectation {
    Value(JavaExpr),
    Error(JavaExpr),
}

#[doc(hidden)]
pub struct JavaPortableTestCaseInput {
    pub(crate) index: usize,
    pub(crate) name: String,
    pub(crate) actual: JavaExpr,
    pub(crate) expected: JavaPortableTestExpectation,
}

#[doc(hidden)]
pub struct JavaPortableTestHarnessInput {
    pub(crate) class_name: String,
    pub(crate) cases: Vec<Vec<JavaStmt>>,
    pub(crate) expected_test_count: i32,
}

#[doc(hidden)]
pub enum JavaPortableTestsInput {
    Case(Box<JavaPortableTestCaseInput>),
    Harness(JavaPortableTestHarnessInput),
}

#[doc(hidden)]
pub enum JavaPortableTestsNode {
    Case(Vec<JavaStmt>),
    Harness(JavaTypeDeclaration),
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct JavaPortableTests;

impl sealed::JavaCapabilityMapping for JavaPortableTests {}
impl JavaCapabilityMapping for JavaPortableTests {}

impl CapabilityMapping<JavaDialect> for JavaPortableTests {
    type Capability = PortableTests;
    type Context = ();
    type Input = JavaPortableTestsInput;
    type Output = JavaPortableTestsNode;
    type Error = Vec<Diagnostic>;

    fn lower(
        &self,
        _context: &mut Self::Context,
        input: Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        Ok(match input {
            JavaPortableTestsInput::Case(input) => JavaPortableTestsNode::Case(lower_case(*input)),
            JavaPortableTestsInput::Harness(input) => {
                JavaPortableTestsNode::Harness(lower_harness(input))
            }
        })
    }
}

fn lower_case(input: JavaPortableTestCaseInput) -> Vec<JavaStmt> {
    let JavaPortableTestCaseInput {
        index,
        name,
        actual,
        expected,
    } = input;
    let result_type = actual.ty.clone();
    let actual_name = identifier(&format!("actual{index}"));
    let actual_local = JavaExpr::local(result_type.clone(), actual_name.clone());
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let ok = member_call(
        actual_local.clone(),
        "ok",
        vec![],
        boolean.clone(),
        JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultOk),
    );
    let mut statements = vec![JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: result_type,
        name: actual_name,
        value: Some(actual),
    }];

    match expected {
        JavaPortableTestExpectation::Value(expected) => {
            statements.push(assert_true(
                ok,
                false,
                format!("portable test {index} ({name}) unexpectedly failed"),
            ));
            let actual_value = member_call(
                actual_local,
                "value",
                vec![],
                expected.ty.clone(),
                JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultValue),
            );
            statements.push(assert_true(
                runtime_call(
                    JavaRuntimeCallable::DeepEqual,
                    vec![actual_value, expected],
                    boolean,
                ),
                false,
                format!("portable test {index} ({name}) value mismatch"),
            ));
        }
        JavaPortableTestExpectation::Error(expected) => {
            statements.push(assert_true(
                ok,
                true,
                format!("portable test {index} ({name}) unexpectedly succeeded"),
            ));
            let error = member_call(
                actual_local,
                "error",
                vec![],
                JavaType::known(JavaKnownType::RuntimeError),
                JavaMemberOrigin::Runtime(JavaRuntimeMember::ResultError),
            );
            let string = JavaType::known(JavaKnownType::String);
            let actual_code = member_call(
                error.clone(),
                "code",
                vec![],
                string.clone(),
                JavaMemberOrigin::Runtime(JavaRuntimeMember::ErrorCode),
            );
            statements.push(assert_true(
                runtime_call(
                    JavaRuntimeCallable::SemanticEqual,
                    vec![actual_code, expected.clone()],
                    boolean.clone(),
                ),
                false,
                format!("portable test {index} ({name}) error code mismatch"),
            ));
            let actual_message = member_call(
                error,
                "message",
                vec![],
                string,
                JavaMemberOrigin::Runtime(JavaRuntimeMember::ErrorMessage),
            );
            statements.push(assert_true(
                runtime_call(
                    JavaRuntimeCallable::SemanticEqual,
                    vec![actual_message, expected],
                    boolean,
                ),
                false,
                format!("portable test {index} ({name}) error message mismatch"),
            ));
        }
    }
    statements
}

fn lower_harness(input: JavaPortableTestHarnessInput) -> JavaTypeDeclaration {
    let expected_test_count = input.expected_test_count;
    let int = JavaType::primitive(JavaPrimitive::Int);
    let completed_name = identifier("completed");
    let completed = JavaExpr::local(int.clone(), completed_name.clone());
    let mut statements = vec![JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        ty: int.clone(),
        name: completed_name,
        value: Some(i32_literal(0)),
    }];
    for mut case in input.cases {
        statements.append(&mut case);
        statements.push(JavaStmt::Assign {
            target: completed.clone(),
            value: binary(
                crate::ast::JavaBinaryOperator::Add,
                completed.clone(),
                i32_literal(1),
                int.clone(),
            ),
        });
    }
    statements.push(assert_true(
        binary(
            crate::ast::JavaBinaryOperator::Equal,
            completed,
            i32_literal(expected_test_count),
            JavaType::primitive(JavaPrimitive::Boolean),
        ),
        false,
        format!("portable conformance inventory mismatch: expected {expected_test_count} tests"),
    ));

    JavaTypeDeclaration {
        declared: None,
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: identifier(&input.class_name),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(private_constructor(&input.class_name)),
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Structural,
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                type_parameters: vec![],
                return_type: JavaType::primitive(JavaPrimitive::Void),
                name: identifier("main"),
                parameters: vec![JavaParameter {
                    ty: JavaType::Array {
                        component: Box::new(JavaType::known(JavaKnownType::String)),
                        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
                    },
                    name: identifier("arguments"),
                    final_parameter: true,
                }],
                body: Some(JavaBlock::new(statements)),
            }),
        ],
    }
}

fn assert_true(condition: JavaExpr, negate: bool, message: String) -> JavaStmt {
    let condition = if negate {
        condition
    } else {
        unary(
            JavaUnaryOperator::Not,
            condition,
            JavaType::primitive(JavaPrimitive::Boolean),
        )
    };
    JavaStmt::If {
        condition,
        then_block: JavaBlock::new(vec![JavaStmt::ThrowAssertion(string_literal(&message))]),
        else_block: None,
    }
}
