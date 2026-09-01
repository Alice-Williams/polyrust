use std::collections::BTreeMap;

use portable_check::v0::CheckedProgram;
use portable_ir::v0::{
    Block, ConstantExpression, Declaration, ExpectedOutcome, Expression, Intrinsic, MatchArm,
    MethodDispatch, NodeId, Pattern, Statement, TestDeclaration, TestInvocation, Value, ValueField,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluationLimits {
    pub fuel: u64,
    pub call_depth: u32,
    pub collection_size: usize,
}

impl Default for EvaluationLimits {
    fn default() -> Self {
        Self {
            fuel: 100_000,
            call_depth: 64,
            collection_size: 100_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvaluationError {
    CheckedOverflow { operation: &'static str },
    DivisionByZero,
    RemainderByZero,
    InvalidShift { amount: i64, width: u8 },
    NarrowingOutOfRange { value: i64 },
    IndexOutOfBounds { index: i64, length: u64 },
    InvalidUtf8,
    FuelExhausted { limit: u64 },
    CallDepthExceeded { limit: u32 },
    CollectionLimitExceeded { limit: usize, requested: usize },
    InvariantViolation { message: String },
}

impl EvaluationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::CheckedOverflow { .. } => "checked_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::RemainderByZero => "remainder_by_zero",
            Self::InvalidShift { .. } => "invalid_shift",
            Self::NarrowingOutOfRange { .. } => "narrowing_out_of_range",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::FuelExhausted { .. } => "fuel_exhausted",
            Self::CallDepthExceeded { .. } => "call_depth_exceeded",
            Self::CollectionLimitExceeded { .. } => "collection_limit_exceeded",
            Self::InvariantViolation { .. } => "invariant_violation",
        }
    }

    pub fn portable_value(&self) -> Value {
        Value::String(self.code().to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvaluationOutcome {
    Value(Value),
    Error(EvaluationError),
}

impl From<Result<Value, EvaluationError>> for EvaluationOutcome {
    fn from(result: Result<Value, EvaluationError>) -> Self {
        match result {
            Ok(value) => Self::Value(value),
            Err(error) => Self::Error(error),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortableTestResult {
    pub declaration: NodeId,
    pub name: String,
    pub actual: EvaluationOutcome,
    pub passed: bool,
}

pub struct Evaluator<'a> {
    program: &'a CheckedProgram,
    limits: EvaluationLimits,
}

impl<'a> Evaluator<'a> {
    pub fn new(program: &'a CheckedProgram) -> Self {
        Self::with_limits(program, EvaluationLimits::default())
    }

    pub const fn with_limits(program: &'a CheckedProgram, limits: EvaluationLimits) -> Self {
        Self { program, limits }
    }

    pub fn invoke_function(&self, function: NodeId, arguments: &[Value]) -> EvaluationOutcome {
        Session::new(self.program, self.limits)
            .invoke_function(function, arguments)
            .into()
    }

    pub fn invoke_method(
        &self,
        implementation: NodeId,
        method: NodeId,
        receiver: Value,
        arguments: &[Value],
    ) -> EvaluationOutcome {
        Session::new(self.program, self.limits)
            .invoke_method(implementation, method, receiver, arguments)
            .into()
    }

    pub fn run_test(&self, declaration: NodeId) -> PortableTestResult {
        let test =
            self.program
                .module()
                .declarations
                .iter()
                .find_map(|candidate| match candidate {
                    Declaration::Test(test) if test.header.node.id == declaration => Some(test),
                    _ => None,
                });
        match test {
            Some(test) => self.execute_test(test),
            None => PortableTestResult {
                declaration,
                name: format!("<missing:{}>", declaration.0),
                actual: EvaluationOutcome::Error(EvaluationError::InvariantViolation {
                    message: format!("test declaration {} does not exist", declaration.0),
                }),
                passed: false,
            },
        }
    }

    pub fn run_all_tests(&self) -> Vec<PortableTestResult> {
        self.program
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Test(test) => Some(self.execute_test(test)),
                _ => None,
            })
            .collect()
    }

    fn execute_test(&self, test: &TestDeclaration) -> PortableTestResult {
        let actual = match &test.invocation {
            TestInvocation::Function {
                function,
                arguments,
            } => self.invoke_function(
                *function,
                &arguments
                    .iter()
                    .map(|argument| argument.value.clone())
                    .collect::<Vec<_>>(),
            ),
            TestInvocation::Method {
                implementation,
                method,
                receiver,
                arguments,
            } => self.invoke_method(
                *implementation,
                *method,
                receiver.value.clone(),
                &arguments
                    .iter()
                    .map(|argument| argument.value.clone())
                    .collect::<Vec<_>>(),
            ),
        };
        let passed = match (&test.expected, &actual) {
            (ExpectedOutcome::Value(expected), EvaluationOutcome::Value(actual)) => {
                test_equal(&expected.value, actual)
            }
            (ExpectedOutcome::Error(expected), EvaluationOutcome::Error(actual)) => {
                test_equal(&expected.value, &actual.portable_value())
            }
            _ => false,
        };
        PortableTestResult {
            declaration: test.header.node.id,
            name: test.header.name.clone(),
            actual,
            passed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Flow<T> {
    Continue(T),
    Return(Value),
}

macro_rules! flow_value {
    ($flow:expr) => {
        match $flow {
            Flow::Continue(value) => value,
            Flow::Return(value) => return Ok(Flow::Return(value)),
        }
    };
}

type Environment = BTreeMap<String, Value>;

struct Session<'a> {
    program: &'a CheckedProgram,
    limits: EvaluationLimits,
    remaining_fuel: u64,
    call_depth: u32,
}

impl<'a> Session<'a> {
    const fn new(program: &'a CheckedProgram, limits: EvaluationLimits) -> Self {
        Self {
            program,
            limits,
            remaining_fuel: limits.fuel,
            call_depth: 0,
        }
    }

    fn burn(&mut self) -> Result<(), EvaluationError> {
        if self.remaining_fuel == 0 {
            return Err(EvaluationError::FuelExhausted {
                limit: self.limits.fuel,
            });
        }
        self.remaining_fuel -= 1;
        Ok(())
    }

    fn enter_call<T>(
        &mut self,
        evaluate: impl FnOnce(&mut Self) -> Result<T, EvaluationError>,
    ) -> Result<T, EvaluationError> {
        if self.call_depth >= self.limits.call_depth {
            return Err(EvaluationError::CallDepthExceeded {
                limit: self.limits.call_depth,
            });
        }
        self.call_depth += 1;
        let result = evaluate(self);
        self.call_depth -= 1;
        result
    }

    fn invoke_function(
        &mut self,
        function_id: NodeId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        self.burn()?;
        let function = self
            .program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Function(function) if function.header.node.id == function_id => {
                    Some(function)
                }
                _ => None,
            })
            .ok_or_else(|| self.invariant(format!("function {} is missing", function_id.0)))?;
        if function.parameters.len() != arguments.len() {
            return Err(self.invariant("checked function argument count changed"));
        }
        for argument in arguments {
            self.check_value_size(argument)?;
        }
        let environment = function
            .parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.header.name.clone(), argument.clone()))
            .collect();
        self.enter_call(|session| {
            let flow = session.eval_block(&function.body, &environment, None)?;
            Ok(terminal_value(flow))
        })
    }

    fn invoke_method(
        &mut self,
        implementation_id: NodeId,
        method_id: NodeId,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        self.burn()?;
        let implementation = self
            .program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Implementation(value) if value.header.node.id == implementation_id => {
                    Some(value)
                }
                _ => None,
            })
            .ok_or_else(|| self.invariant("checked implementation is missing"))?;
        let method = implementation
            .methods
            .iter()
            .find(|method| method.header.node.id == method_id)
            .ok_or_else(|| self.invariant("checked method is missing"))?;
        if method.parameters.len() != arguments.len() {
            return Err(self.invariant("checked method argument count changed"));
        }
        self.check_value_size(&receiver)?;
        for argument in arguments {
            self.check_value_size(argument)?;
        }
        let environment = method
            .parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| (parameter.header.name.clone(), argument.clone()))
            .collect();
        self.enter_call(|session| {
            let flow = session.eval_block(&method.body, &environment, Some(receiver.clone()))?;
            Ok(terminal_value(flow))
        })
    }

    fn eval_block(
        &mut self,
        block: &Block,
        outer: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Value>, EvaluationError> {
        self.burn()?;
        let mut environment = outer.clone();
        for statement in &block.statements {
            self.burn()?;
            match statement {
                Statement::Let { name, value, .. } => {
                    let flow = self.eval_expression(value, &environment, self_value.clone())?;
                    environment.insert(name.clone(), flow_value!(flow));
                }
                Statement::ForEach {
                    binding,
                    iterable,
                    body,
                    ..
                } => {
                    let flow = self.eval_expression(iterable, &environment, self_value.clone())?;
                    let Value::List(elements) = flow_value!(flow) else {
                        return Err(self.invariant("checked for-each input is not a list"));
                    };
                    for element in elements {
                        self.burn()?;
                        let mut loop_environment = environment.clone();
                        loop_environment.insert(binding.clone(), element);
                        let flow = self.eval_block(body, &loop_environment, self_value.clone())?;
                        if let Flow::Return(value) = flow {
                            return Ok(Flow::Return(value));
                        }
                    }
                }
                Statement::Return { value, .. } => {
                    let value = match value {
                        Some(expression) => terminal_value(self.eval_expression(
                            expression,
                            &environment,
                            self_value.clone(),
                        )?),
                        None => Value::Unit,
                    };
                    return Ok(Flow::Return(value));
                }
                Statement::Expression { value, .. } => {
                    let flow = self.eval_expression(value, &environment, self_value.clone())?;
                    if let Flow::Return(value) = flow {
                        return Ok(Flow::Return(value));
                    }
                }
            }
        }
        match &block.result {
            Some(result) => self.eval_expression(result, &environment, self_value),
            None => Ok(Flow::Continue(Value::Unit)),
        }
    }

    fn eval_expression(
        &mut self,
        expression: &Expression,
        environment: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Value>, EvaluationError> {
        self.burn()?;
        match expression {
            Expression::Literal { value, .. } => {
                self.check_value_size(value)?;
                Ok(Flow::Continue(value.clone()))
            }
            Expression::Local { name, .. } => environment
                .get(name)
                .cloned()
                .map(Flow::Continue)
                .ok_or_else(|| self.invariant(format!("resolved local {name:?} is absent"))),
            Expression::Constant { declaration, .. } => {
                self.eval_constant_by_id(*declaration).map(Flow::Continue)
            }
            Expression::SelfValue { .. } => self_value
                .map(Flow::Continue)
                .ok_or_else(|| self.invariant("checked self value is absent")),
            Expression::ConstructRecord {
                declaration,
                fields,
                ..
            } => {
                let flow = self.eval_fields(fields, environment, self_value)?;
                Ok(Flow::Continue(Value::Record {
                    declaration: *declaration,
                    fields: flow_value!(flow),
                }))
            }
            Expression::ConstructEnum {
                declaration,
                variant,
                fields,
                ..
            } => {
                let flow = self.eval_fields(fields, environment, self_value)?;
                Ok(Flow::Continue(Value::Enum {
                    declaration: *declaration,
                    variant: *variant,
                    fields: flow_value!(flow),
                }))
            }
            Expression::ConstructSome { value, .. } => {
                let flow = self.eval_expression(value, environment, self_value)?;
                Ok(Flow::Continue(Value::Some(Box::new(flow_value!(flow)))))
            }
            Expression::ConstructNone { .. } => Ok(Flow::Continue(Value::None)),
            Expression::ConstructOk { value, .. } => {
                let flow = self.eval_expression(value, environment, self_value)?;
                Ok(Flow::Continue(Value::Ok(Box::new(flow_value!(flow)))))
            }
            Expression::ConstructErr { value, .. } => {
                let flow = self.eval_expression(value, environment, self_value)?;
                Ok(Flow::Continue(Value::Err(Box::new(flow_value!(flow)))))
            }
            Expression::ConstructList { elements, .. } => {
                self.check_collection_size(elements.len())?;
                let mut values = Vec::with_capacity(elements.len());
                for element in elements {
                    let flow = self.eval_expression(element, environment, self_value.clone())?;
                    values.push(flow_value!(flow));
                }
                Ok(Flow::Continue(Value::List(values)))
            }
            Expression::Field { base, field, .. } => {
                let flow = self.eval_expression(base, environment, self_value)?;
                let Value::Record { fields, .. } = flow_value!(flow) else {
                    return Err(self.invariant("checked field receiver is not a record"));
                };
                fields
                    .into_iter()
                    .find(|candidate| candidate.field == *field)
                    .map(|field| Flow::Continue(field.value))
                    .ok_or_else(|| self.invariant(format!("record field {} is absent", field.0)))
            }
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                let flow = self.eval_arguments(arguments, environment, self_value)?;
                self.invoke_function(*function, &flow_value!(flow))
                    .map(Flow::Continue)
            }
            Expression::MethodCall {
                receiver,
                dispatch,
                arguments,
                ..
            } => {
                let flow = self.eval_expression(receiver, environment, self_value.clone())?;
                let receiver = flow_value!(flow);
                let flow = self.eval_arguments(arguments, environment, self_value)?;
                let arguments = flow_value!(flow);
                let (implementation, method) = match dispatch {
                    MethodDispatch::Concrete {
                        implementation,
                        method,
                    } => (*implementation, *method),
                    MethodDispatch::Contract { contract, method } => {
                        self.resolve_contract_dispatch(&receiver, *contract, *method)?
                    }
                };
                self.invoke_method(implementation, method, receiver, &arguments)
                    .map(Flow::Continue)
            }
            Expression::Intrinsic {
                operation,
                arguments,
                ..
            } => self.eval_intrinsic_expression(*operation, arguments, environment, self_value),
            Expression::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let flow = self.eval_expression(condition, environment, self_value.clone())?;
                match flow_value!(flow) {
                    Value::Bool(true) => self.eval_block(then_block, environment, self_value),
                    Value::Bool(false) => self.eval_block(else_block, environment, self_value),
                    _ => Err(self.invariant("checked if condition is not boolean")),
                }
            }
            Expression::Match { value, arms, .. } => {
                let flow = self.eval_expression(value, environment, self_value.clone())?;
                self.eval_match(&flow_value!(flow), arms, environment, self_value)
            }
            Expression::Block(block) => self.eval_block(block, environment, self_value),
        }
    }

    fn eval_fields(
        &mut self,
        fields: &[portable_ir::v0::ExpressionField],
        environment: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Vec<ValueField>>, EvaluationError> {
        self.check_collection_size(fields.len())?;
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            let flow = self.eval_expression(&field.value, environment, self_value.clone())?;
            values.push(ValueField {
                field: field.field,
                value: flow_value!(flow),
            });
        }
        Ok(Flow::Continue(values))
    }

    fn eval_arguments(
        &mut self,
        arguments: &[Expression],
        environment: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Vec<Value>>, EvaluationError> {
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let flow = self.eval_expression(argument, environment, self_value.clone())?;
            values.push(flow_value!(flow));
        }
        Ok(Flow::Continue(values))
    }

    fn eval_intrinsic_expression(
        &mut self,
        operation: Intrinsic,
        arguments: &[Expression],
        environment: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Value>, EvaluationError> {
        if matches!(operation, Intrinsic::BoolAnd | Intrinsic::BoolOr) {
            let flow = self.eval_expression(&arguments[0], environment, self_value.clone())?;
            let Value::Bool(first) = flow_value!(flow) else {
                return Err(self.invariant("checked boolean operand is not boolean"));
            };
            if (operation == Intrinsic::BoolAnd && !first)
                || (operation == Intrinsic::BoolOr && first)
            {
                return Ok(Flow::Continue(Value::Bool(first)));
            }
            let flow = self.eval_expression(&arguments[1], environment, self_value)?;
            let Value::Bool(second) = flow_value!(flow) else {
                return Err(self.invariant("checked boolean operand is not boolean"));
            };
            return Ok(Flow::Continue(Value::Bool(second)));
        }
        let flow = self.eval_arguments(arguments, environment, self_value)?;
        self.eval_intrinsic(operation, &flow_value!(flow))
            .map(Flow::Continue)
    }

    fn eval_match(
        &mut self,
        value: &Value,
        arms: &[MatchArm],
        environment: &Environment,
        self_value: Option<Value>,
    ) -> Result<Flow<Value>, EvaluationError> {
        for arm in arms {
            let mut arm_environment = environment.clone();
            if self.pattern_matches(&arm.pattern, value, &mut arm_environment)? {
                return self.eval_block(&arm.body, &arm_environment, self_value);
            }
        }
        Err(self.invariant("checked exhaustive match selected no arm"))
    }

    fn pattern_matches(
        &self,
        pattern: &Pattern,
        value: &Value,
        environment: &mut Environment,
    ) -> Result<bool, EvaluationError> {
        Ok(match (pattern, value) {
            (Pattern::Wildcard { .. }, _) => true,
            (
                Pattern::Bool {
                    value: expected, ..
                },
                Value::Bool(actual),
            ) => expected == actual,
            (Pattern::None { .. }, Value::None) => true,
            (Pattern::Some { binding, .. }, Value::Some(inner))
            | (Pattern::Ok { binding, .. }, Value::Ok(inner))
            | (Pattern::Err { binding, .. }, Value::Err(inner)) => {
                environment.insert(binding.clone(), (**inner).clone());
                true
            }
            (
                Pattern::EnumVariant {
                    declaration,
                    variant,
                    bindings,
                    ..
                },
                Value::Enum {
                    declaration: actual_declaration,
                    variant: actual_variant,
                    fields,
                },
            ) if declaration == actual_declaration && variant == actual_variant => {
                for binding in bindings {
                    let field = fields
                        .iter()
                        .find(|field| field.field == binding.field)
                        .ok_or_else(|| self.invariant("checked enum payload field is absent"))?;
                    environment.insert(binding.binding.clone(), field.value.clone());
                }
                true
            }
            _ => false,
        })
    }

    fn resolve_contract_dispatch(
        &self,
        receiver: &Value,
        contract: NodeId,
        contract_method: NodeId,
    ) -> Result<(NodeId, NodeId), EvaluationError> {
        let Value::Record {
            declaration: record,
            ..
        } = receiver
        else {
            return Err(self.invariant("checked contract receiver is not a record"));
        };
        self.program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Implementation(implementation)
                    if implementation.contract == contract && implementation.record == *record =>
                {
                    implementation
                        .methods
                        .iter()
                        .find(|method| method.contract_method == contract_method)
                        .map(|method| (implementation.header.node.id, method.header.node.id))
                }
                _ => None,
            })
            .ok_or_else(|| self.invariant("checked contract dispatch target is absent"))
    }

    fn eval_constant_by_id(&mut self, id: NodeId) -> Result<Value, EvaluationError> {
        self.burn()?;
        let constant = self
            .program
            .module()
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Constant(constant) if constant.header.node.id == id => Some(constant),
                _ => None,
            })
            .ok_or_else(|| self.invariant("checked constant is absent"))?;
        self.eval_constant(&constant.value)
    }

    fn eval_constant(&mut self, expression: &ConstantExpression) -> Result<Value, EvaluationError> {
        self.burn()?;
        match expression {
            ConstantExpression::Literal { value, .. } => {
                self.check_value_size(value)?;
                Ok(value.clone())
            }
            ConstantExpression::Reference { declaration, .. } => {
                self.eval_constant_by_id(*declaration)
            }
            ConstantExpression::Record {
                declaration,
                fields,
                ..
            }
            | ConstantExpression::Enum {
                declaration,
                fields,
                ..
            } => {
                self.check_collection_size(fields.len())?;
                let fields = fields
                    .iter()
                    .map(|field| {
                        Ok(ValueField {
                            field: field.field,
                            value: self.eval_constant(&field.value)?,
                        })
                    })
                    .collect::<Result<Vec<_>, EvaluationError>>()?;
                match expression {
                    ConstantExpression::Record { .. } => Ok(Value::Record {
                        declaration: *declaration,
                        fields,
                    }),
                    ConstantExpression::Enum { variant, .. } => Ok(Value::Enum {
                        declaration: *declaration,
                        variant: *variant,
                        fields,
                    }),
                    _ => Err(self.invariant("constant aggregate changed while evaluating")),
                }
            }
            ConstantExpression::Some { value, .. } => {
                Ok(Value::Some(Box::new(self.eval_constant(value)?)))
            }
            ConstantExpression::None { .. } => Ok(Value::None),
            ConstantExpression::Ok { value, .. } => {
                Ok(Value::Ok(Box::new(self.eval_constant(value)?)))
            }
            ConstantExpression::Err { value, .. } => {
                Ok(Value::Err(Box::new(self.eval_constant(value)?)))
            }
            ConstantExpression::List { elements, .. } => {
                self.check_collection_size(elements.len())?;
                elements
                    .iter()
                    .map(|element| self.eval_constant(element))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::List)
            }
            ConstantExpression::Intrinsic {
                operation,
                arguments,
                ..
            } => {
                if matches!(operation, Intrinsic::BoolAnd | Intrinsic::BoolOr) {
                    let Value::Bool(first) = self.eval_constant(&arguments[0])? else {
                        return Err(self.invariant("checked constant boolean is not boolean"));
                    };
                    if (*operation == Intrinsic::BoolAnd && !first)
                        || (*operation == Intrinsic::BoolOr && first)
                    {
                        return Ok(Value::Bool(first));
                    }
                    return self.eval_constant(&arguments[1]);
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.eval_constant(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                self.eval_intrinsic(*operation, &arguments)
            }
        }
    }

    fn eval_intrinsic(
        &mut self,
        operation: Intrinsic,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        use Intrinsic::*;
        match operation {
            BoolNot => unary_bool(arguments, |value| !value, self),
            BoolAnd => binary_bool(arguments, |left, right| left && right, self),
            BoolOr => binary_bool(arguments, |left, right| left || right, self),
            Equal => Ok(Value::Bool(semantic_equal(&arguments[0], &arguments[1]))),
            NotEqual => Ok(Value::Bool(!semantic_equal(&arguments[0], &arguments[1]))),
            Less => self.compare(arguments, std::cmp::Ordering::is_lt),
            LessEqual => self.compare(arguments, std::cmp::Ordering::is_le),
            Greater => self.compare(arguments, std::cmp::Ordering::is_gt),
            GreaterEqual => self.compare(arguments, std::cmp::Ordering::is_ge),
            IntNegChecked => {
                self.integer_unary(arguments, "neg", i32::checked_neg, i64::checked_neg)
            }
            IntAddChecked => {
                self.integer_binary(arguments, "add", i32::checked_add, i64::checked_add)
            }
            IntSubChecked => {
                self.integer_binary(arguments, "sub", i32::checked_sub, i64::checked_sub)
            }
            IntMulChecked => {
                self.integer_binary(arguments, "mul", i32::checked_mul, i64::checked_mul)
            }
            IntDivChecked => self.integer_div(arguments),
            IntRemChecked => self.integer_rem(arguments),
            IntNegWrapping => {
                self.integer_unary_infallible(arguments, i32::wrapping_neg, i64::wrapping_neg)
            }
            IntAddWrapping => {
                self.integer_binary_infallible(arguments, i32::wrapping_add, i64::wrapping_add)
            }
            IntSubWrapping => {
                self.integer_binary_infallible(arguments, i32::wrapping_sub, i64::wrapping_sub)
            }
            IntMulWrapping => {
                self.integer_binary_infallible(arguments, i32::wrapping_mul, i64::wrapping_mul)
            }
            IntBitNot => self.integer_unary_infallible(arguments, |value| !value, |value| !value),
            IntBitAnd => self.integer_binary_infallible(
                arguments,
                |left, right| left & right,
                |left, right| left & right,
            ),
            IntBitOr => self.integer_binary_infallible(
                arguments,
                |left, right| left | right,
                |left, right| left | right,
            ),
            IntBitXor => self.integer_binary_infallible(
                arguments,
                |left, right| left ^ right,
                |left, right| left ^ right,
            ),
            IntShiftLeftChecked => self.integer_shift(arguments, true),
            IntShiftRightChecked => self.integer_shift(arguments, false),
            FloatNeg => unary_float(arguments, |value| -value, self),
            FloatTrunc => unary_float(arguments, f64::trunc, self),
            FloatAdd => binary_float(arguments, |left, right| left + right, self),
            FloatSub => binary_float(arguments, |left, right| left - right, self),
            FloatMul => binary_float(arguments, |left, right| left * right, self),
            FloatDiv => binary_float(arguments, |left, right| left / right, self),
            FloatRemTrunc => binary_float(arguments, |left, right| left % right, self),
            StringConcat => {
                let (left, right) = strings(arguments, self)?;
                self.check_collection_sum(left.len(), right.len())?;
                Ok(Value::String(format!("{left}{right}")))
            }
            StringScalarLength => {
                let value = string(arguments.first(), self)?;
                Ok(Value::I64(usize_to_i64(value.chars().count(), self)?))
            }
            StringIsEmpty => Ok(Value::Bool(string(arguments.first(), self)?.is_empty())),
            StringContains => {
                let (left, right) = strings(arguments, self)?;
                Ok(Value::Bool(left.contains(right)))
            }
            StringStartsWith => {
                let (left, right) = strings(arguments, self)?;
                Ok(Value::Bool(left.starts_with(right)))
            }
            StringStripPrefix => {
                let (source, prefix) = strings(arguments, self)?;
                Ok(Value::String(
                    source.strip_prefix(prefix).unwrap_or(source).to_owned(),
                ))
            }
            StringEndsWith => {
                let (left, right) = strings(arguments, self)?;
                Ok(Value::Bool(left.ends_with(right)))
            }
            StringReplaceAll => {
                let source = string(arguments.first(), self)?;
                let needle = string(arguments.get(1), self)?;
                let replacement = string(arguments.get(2), self)?;
                let matches = if needle.is_empty() {
                    source.chars().count().checked_add(1).ok_or(
                        EvaluationError::CollectionLimitExceeded {
                            limit: self.limits.collection_size,
                            requested: usize::MAX,
                        },
                    )?
                } else {
                    source.match_indices(needle).count()
                };
                let removed = matches.checked_mul(needle.len()).ok_or(
                    EvaluationError::CollectionLimitExceeded {
                        limit: self.limits.collection_size,
                        requested: usize::MAX,
                    },
                )?;
                let added = matches.checked_mul(replacement.len()).ok_or(
                    EvaluationError::CollectionLimitExceeded {
                        limit: self.limits.collection_size,
                        requested: usize::MAX,
                    },
                )?;
                let requested = source
                    .len()
                    .checked_sub(removed)
                    .and_then(|remaining| remaining.checked_add(added))
                    .ok_or(EvaluationError::CollectionLimitExceeded {
                        limit: self.limits.collection_size,
                        requested: usize::MAX,
                    })?;
                self.check_collection_size(requested)?;
                Ok(Value::String(source.replace(needle, replacement)))
            }
            StringReplaceMany => {
                if arguments.len() < 3 || arguments.len().is_multiple_of(2) {
                    return Err(EvaluationError::InvariantViolation {
                        message: "StringReplaceMany requires a source and one or more needle/replacement pairs"
                            .to_owned(),
                    });
                }
                let source = string(arguments.first(), self)?;
                let mut mappings = Vec::with_capacity((arguments.len() - 1) / 2);
                for pair in arguments[1..].as_chunks::<2>().0 {
                    mappings.push((string(pair.first(), self)?, string(pair.get(1), self)?));
                }
                let mut output = String::new();
                let mut offset = 0;
                loop {
                    let remaining = &source[offset..];
                    if let Some((needle, replacement)) = mappings
                        .iter()
                        .find(|(needle, _)| remaining.starts_with(*needle))
                    {
                        let requested = output.len().checked_add(replacement.len()).ok_or(
                            EvaluationError::CollectionLimitExceeded {
                                limit: self.limits.collection_size,
                                requested: usize::MAX,
                            },
                        )?;
                        self.check_collection_size(requested)?;
                        output.push_str(replacement);
                        if needle.is_empty() {
                            let Some(character) = remaining.chars().next() else {
                                break;
                            };
                            let width = character.len_utf8();
                            let requested = output.len().checked_add(width).ok_or(
                                EvaluationError::CollectionLimitExceeded {
                                    limit: self.limits.collection_size,
                                    requested: usize::MAX,
                                },
                            )?;
                            self.check_collection_size(requested)?;
                            output.push_str(&remaining[..width]);
                            offset += width;
                        } else {
                            offset += needle.len();
                        }
                    } else {
                        let Some(character) = remaining.chars().next() else {
                            break;
                        };
                        let width = character.len_utf8();
                        let requested = output.len().checked_add(width).ok_or(
                            EvaluationError::CollectionLimitExceeded {
                                limit: self.limits.collection_size,
                                requested: usize::MAX,
                            },
                        )?;
                        self.check_collection_size(requested)?;
                        output.push_str(&remaining[..width]);
                        offset += width;
                    }
                }
                Ok(Value::String(output))
            }
            StringTruncateUtf8Bytes => {
                let source = string(arguments.first(), self)?;
                let budget = match arguments.get(1) {
                    Some(Value::F64(value)) => value.to_f64(),
                    _ => return Err(self.invariant("checked UTF-8 budget operand is not f64")),
                };
                for (offset, character) in source.char_indices() {
                    let end = offset + character.len_utf8();
                    let consumed = end as f64;
                    if consumed == budget {
                        return Ok(Value::String(source[..end].to_owned()));
                    }
                    if consumed > budget {
                        return Ok(Value::String(source[..offset].to_owned()));
                    }
                }
                Ok(Value::String(source.to_owned()))
            }
            StringTrimStart => {
                let source = string(arguments.first(), self)?;
                let characters = string(arguments.get(1), self)?;
                Ok(Value::String(
                    source
                        .trim_start_matches(|character| characters.contains(character))
                        .to_owned(),
                ))
            }
            StringTrimEnd => {
                let source = string(arguments.first(), self)?;
                let characters = string(arguments.get(1), self)?;
                Ok(Value::String(
                    source
                        .trim_end_matches(|character| characters.contains(character))
                        .to_owned(),
                ))
            }
            BytesConcat => {
                let (left, right) = bytes(arguments, self)?;
                self.check_collection_sum(left.len(), right.len())?;
                let mut value = left.to_vec();
                value.extend_from_slice(right);
                Ok(Value::Bytes(value))
            }
            BytesLength => Ok(Value::I64(usize_to_i64(
                byte_string(arguments.first(), self)?.len(),
                self,
            )?)),
            BytesIsEmpty => Ok(Value::Bool(
                byte_string(arguments.first(), self)?.is_empty(),
            )),
            ListLength => Ok(Value::I64(usize_to_i64(
                list(arguments.first(), self)?.len(),
                self,
            )?)),
            ListIsEmpty => Ok(Value::Bool(list(arguments.first(), self)?.is_empty())),
            ListGetChecked => {
                let elements = list(arguments.first(), self)?;
                let index = i64_value(arguments.get(1), self)?;
                let valid = usize::try_from(index)
                    .ok()
                    .filter(|index| *index < elements.len());
                let Some(index) = valid else {
                    return Err(EvaluationError::IndexOutOfBounds {
                        index,
                        length: u64::try_from(elements.len()).unwrap_or(u64::MAX),
                    });
                };
                Ok(elements[index].clone())
            }
            ListAppend => {
                let elements = list(arguments.first(), self)?;
                self.check_collection_sum(elements.len(), 1)?;
                let mut output = elements.to_vec();
                output.push(arguments[1].clone());
                Ok(Value::List(output))
            }
            ListConcat => {
                let (left, right) = lists(arguments, self)?;
                self.check_collection_sum(left.len(), right.len())?;
                let mut output = left.to_vec();
                output.extend_from_slice(right);
                Ok(Value::List(output))
            }
            ListContains => {
                let elements = list(arguments.first(), self)?;
                Ok(Value::Bool(
                    elements
                        .iter()
                        .any(|element| semantic_equal(element, &arguments[1])),
                ))
            }
            OptionIsSome => Ok(Value::Bool(matches!(
                arguments.first(),
                Some(Value::Some(_))
            ))),
            OptionIsNone => Ok(Value::Bool(matches!(arguments.first(), Some(Value::None)))),
            OptionUnwrapOr => match arguments.first() {
                Some(Value::Some(value)) => Ok((**value).clone()),
                Some(Value::None) => Ok(arguments[1].clone()),
                _ => Err(self.invariant("checked option operand is not an option")),
            },
            ResultIsOk => Ok(Value::Bool(matches!(arguments.first(), Some(Value::Ok(_))))),
            ResultIsErr => Ok(Value::Bool(matches!(
                arguments.first(),
                Some(Value::Err(_))
            ))),
            WidenI32ToI64 => Ok(Value::I64(i64::from(i32_value(arguments.first(), self)?))),
            NarrowI64ToI32Checked => {
                let value = i64_value(arguments.first(), self)?;
                i32::try_from(value)
                    .map(Value::I32)
                    .map_err(|_| EvaluationError::NarrowingOutOfRange { value })
            }
            StringToUtf8 => {
                let value = string(arguments.first(), self)?;
                self.check_collection_size(value.len())?;
                Ok(Value::Bytes(value.as_bytes().to_vec()))
            }
            StringFromUtf8Checked => {
                let bytes = byte_string(arguments.first(), self)?;
                let value = std::str::from_utf8(bytes).map_err(|_| EvaluationError::InvalidUtf8)?;
                self.check_collection_size(value.len())?;
                Ok(Value::String(value.to_owned()))
            }
        }
    }

    fn integer_unary(
        &self,
        arguments: &[Value],
        operation: &'static str,
        i32_operation: fn(i32) -> Option<i32>,
        i64_operation: fn(i64) -> Option<i64>,
    ) -> Result<Value, EvaluationError> {
        match arguments.first() {
            Some(Value::I32(value)) => i32_operation(*value)
                .map(Value::I32)
                .ok_or(EvaluationError::CheckedOverflow { operation }),
            Some(Value::I64(value)) => i64_operation(*value)
                .map(Value::I64)
                .ok_or(EvaluationError::CheckedOverflow { operation }),
            _ => Err(self.invariant("checked integer operand is not an integer")),
        }
    }

    fn integer_binary(
        &self,
        arguments: &[Value],
        operation: &'static str,
        i32_operation: fn(i32, i32) -> Option<i32>,
        i64_operation: fn(i64, i64) -> Option<i64>,
    ) -> Result<Value, EvaluationError> {
        match arguments {
            [Value::I32(left), Value::I32(right)] => i32_operation(*left, *right)
                .map(Value::I32)
                .ok_or(EvaluationError::CheckedOverflow { operation }),
            [Value::I64(left), Value::I64(right)] => i64_operation(*left, *right)
                .map(Value::I64)
                .ok_or(EvaluationError::CheckedOverflow { operation }),
            _ => Err(self.invariant("checked integer operands do not match")),
        }
    }

    fn integer_unary_infallible(
        &self,
        arguments: &[Value],
        i32_operation: fn(i32) -> i32,
        i64_operation: fn(i64) -> i64,
    ) -> Result<Value, EvaluationError> {
        match arguments.first() {
            Some(Value::I32(value)) => Ok(Value::I32(i32_operation(*value))),
            Some(Value::I64(value)) => Ok(Value::I64(i64_operation(*value))),
            _ => Err(self.invariant("checked integer operand is not an integer")),
        }
    }

    fn integer_binary_infallible(
        &self,
        arguments: &[Value],
        i32_operation: fn(i32, i32) -> i32,
        i64_operation: fn(i64, i64) -> i64,
    ) -> Result<Value, EvaluationError> {
        match arguments {
            [Value::I32(left), Value::I32(right)] => Ok(Value::I32(i32_operation(*left, *right))),
            [Value::I64(left), Value::I64(right)] => Ok(Value::I64(i64_operation(*left, *right))),
            _ => Err(self.invariant("checked integer operands do not match")),
        }
    }

    fn integer_div(&self, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match arguments {
            [Value::I32(_), Value::I32(0)] | [Value::I64(_), Value::I64(0)] => {
                Err(EvaluationError::DivisionByZero)
            }
            [Value::I32(left), Value::I32(right)] => left
                .checked_div(*right)
                .map(Value::I32)
                .ok_or(EvaluationError::CheckedOverflow { operation: "div" }),
            [Value::I64(left), Value::I64(right)] => left
                .checked_div(*right)
                .map(Value::I64)
                .ok_or(EvaluationError::CheckedOverflow { operation: "div" }),
            _ => Err(self.invariant("checked division operands do not match")),
        }
    }

    fn integer_rem(&self, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match arguments {
            [Value::I32(_), Value::I32(0)] | [Value::I64(_), Value::I64(0)] => {
                Err(EvaluationError::RemainderByZero)
            }
            [Value::I32(left), Value::I32(right)] => left
                .checked_rem(*right)
                .map(Value::I32)
                .ok_or(EvaluationError::CheckedOverflow { operation: "rem" }),
            [Value::I64(left), Value::I64(right)] => left
                .checked_rem(*right)
                .map(Value::I64)
                .ok_or(EvaluationError::CheckedOverflow { operation: "rem" }),
            _ => Err(self.invariant("checked remainder operands do not match")),
        }
    }

    fn integer_shift(
        &self,
        arguments: &[Value],
        left_shift: bool,
    ) -> Result<Value, EvaluationError> {
        match arguments {
            [Value::I32(value), Value::I32(amount)] => {
                let shift = u32::try_from(*amount).map_err(|_| EvaluationError::InvalidShift {
                    amount: i64::from(*amount),
                    width: 32,
                })?;
                let result = if left_shift {
                    value.checked_shl(shift)
                } else {
                    value.checked_shr(shift)
                };
                result.map(Value::I32).ok_or(EvaluationError::InvalidShift {
                    amount: i64::from(*amount),
                    width: 32,
                })
            }
            [Value::I64(value), Value::I64(amount)] => {
                let shift = u32::try_from(*amount).map_err(|_| EvaluationError::InvalidShift {
                    amount: *amount,
                    width: 64,
                })?;
                let result = if left_shift {
                    value.checked_shl(shift)
                } else {
                    value.checked_shr(shift)
                };
                result.map(Value::I64).ok_or(EvaluationError::InvalidShift {
                    amount: *amount,
                    width: 64,
                })
            }
            _ => Err(self.invariant("checked shift operands do not match")),
        }
    }

    fn compare(
        &self,
        arguments: &[Value],
        predicate: impl FnOnce(std::cmp::Ordering) -> bool,
    ) -> Result<Value, EvaluationError> {
        let ordering = match arguments {
            [Value::I32(left), Value::I32(right)] => left.cmp(right),
            [Value::I64(left), Value::I64(right)] => left.cmp(right),
            [Value::Char(left), Value::Char(right)] => left.cmp(right),
            [Value::String(left), Value::String(right)] => left.cmp(right),
            [Value::F64(left), Value::F64(right)] => {
                return Ok(Value::Bool(
                    left.to_f64()
                        .partial_cmp(&right.to_f64())
                        .is_some_and(predicate),
                ));
            }
            _ => return Err(self.invariant("checked comparison operands do not match")),
        };
        Ok(Value::Bool(predicate(ordering)))
    }

    fn check_value_size(&self, value: &Value) -> Result<(), EvaluationError> {
        match value {
            Value::String(value) => self.check_collection_size(value.len()),
            Value::Bytes(value) => self.check_collection_size(value.len()),
            Value::List(values) => {
                self.check_collection_size(values.len())?;
                for value in values {
                    self.check_value_size(value)?;
                }
                Ok(())
            }
            Value::Some(value) | Value::Ok(value) | Value::Err(value) => {
                self.check_value_size(value)
            }
            Value::Record { fields, .. } | Value::Enum { fields, .. } => {
                self.check_collection_size(fields.len())?;
                for field in fields {
                    self.check_value_size(&field.value)?;
                }
                Ok(())
            }
            Value::Unit
            | Value::Bool(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::F64(_)
            | Value::Char(_)
            | Value::None => Ok(()),
        }
    }

    fn check_collection_size(&self, requested: usize) -> Result<(), EvaluationError> {
        if requested > self.limits.collection_size {
            Err(EvaluationError::CollectionLimitExceeded {
                limit: self.limits.collection_size,
                requested,
            })
        } else {
            Ok(())
        }
    }

    fn check_collection_sum(&self, left: usize, right: usize) -> Result<(), EvaluationError> {
        let requested =
            left.checked_add(right)
                .ok_or(EvaluationError::CollectionLimitExceeded {
                    limit: self.limits.collection_size,
                    requested: usize::MAX,
                })?;
        self.check_collection_size(requested)
    }

    fn invariant(&self, message: impl Into<String>) -> EvaluationError {
        EvaluationError::InvariantViolation {
            message: message.into(),
        }
    }
}

fn terminal_value(flow: Flow<Value>) -> Value {
    match flow {
        Flow::Continue(value) | Flow::Return(value) => value,
    }
}

pub(crate) fn semantic_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::F64(left), Value::F64(right)) => left.to_f64() == right.to_f64(),
        (Value::List(left), Value::List(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| semantic_equal(left, right))
        }
        (Value::Some(left), Value::Some(right))
        | (Value::Ok(left), Value::Ok(right))
        | (Value::Err(left), Value::Err(right)) => semantic_equal(left, right),
        (
            Value::Record {
                declaration: left_declaration,
                fields: left_fields,
            },
            Value::Record {
                declaration: right_declaration,
                fields: right_fields,
            },
        ) => left_declaration == right_declaration && equal_fields(left_fields, right_fields),
        (
            Value::Enum {
                declaration: left_declaration,
                variant: left_variant,
                fields: left_fields,
            },
            Value::Enum {
                declaration: right_declaration,
                variant: right_variant,
                fields: right_fields,
            },
        ) => {
            left_declaration == right_declaration
                && left_variant == right_variant
                && equal_fields(left_fields, right_fields)
        }
        _ => left == right,
    }
}

fn equal_fields(left: &[ValueField], right: &[ValueField]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.field == right.field && semantic_equal(&left.value, &right.value)
        })
}

fn test_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::F64(left), Value::F64(right)) => {
            let left = left.to_f64();
            let right = right.to_f64();
            (left.is_nan() && right.is_nan()) || left.to_bits() == right.to_bits()
        }
        (Value::List(left), Value::List(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| test_equal(left, right))
        }
        (Value::Some(left), Value::Some(right))
        | (Value::Ok(left), Value::Ok(right))
        | (Value::Err(left), Value::Err(right)) => test_equal(left, right),
        (
            Value::Record {
                declaration: left_declaration,
                fields: left_fields,
            },
            Value::Record {
                declaration: right_declaration,
                fields: right_fields,
            },
        ) => left_declaration == right_declaration && test_equal_fields(left_fields, right_fields),
        (
            Value::Enum {
                declaration: left_declaration,
                variant: left_variant,
                fields: left_fields,
            },
            Value::Enum {
                declaration: right_declaration,
                variant: right_variant,
                fields: right_fields,
            },
        ) => {
            left_declaration == right_declaration
                && left_variant == right_variant
                && test_equal_fields(left_fields, right_fields)
        }
        _ => left == right,
    }
}

fn test_equal_fields(left: &[ValueField], right: &[ValueField]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.field == right.field && test_equal(&left.value, &right.value))
}

fn unary_bool(
    arguments: &[Value],
    operation: impl FnOnce(bool) -> bool,
    session: &Session<'_>,
) -> Result<Value, EvaluationError> {
    match arguments.first() {
        Some(Value::Bool(value)) => Ok(Value::Bool(operation(*value))),
        _ => Err(session.invariant("checked boolean operand is not boolean")),
    }
}

fn binary_bool(
    arguments: &[Value],
    operation: impl FnOnce(bool, bool) -> bool,
    session: &Session<'_>,
) -> Result<Value, EvaluationError> {
    match arguments {
        [Value::Bool(left), Value::Bool(right)] => Ok(Value::Bool(operation(*left, *right))),
        _ => Err(session.invariant("checked boolean operands do not match")),
    }
}

fn unary_float(
    arguments: &[Value],
    operation: impl FnOnce(f64) -> f64,
    session: &Session<'_>,
) -> Result<Value, EvaluationError> {
    match arguments.first() {
        Some(Value::F64(value)) => Ok(Value::F64(portable_ir::v0::F64Bits::from_f64(operation(
            value.to_f64(),
        )))),
        _ => Err(session.invariant("checked float operand is not a float")),
    }
}

fn binary_float(
    arguments: &[Value],
    operation: impl FnOnce(f64, f64) -> f64,
    session: &Session<'_>,
) -> Result<Value, EvaluationError> {
    match arguments {
        [Value::F64(left), Value::F64(right)] => Ok(Value::F64(
            portable_ir::v0::F64Bits::from_f64(operation(left.to_f64(), right.to_f64())),
        )),
        _ => Err(session.invariant("checked float operands do not match")),
    }
}

fn string<'a>(value: Option<&'a Value>, session: &Session<'_>) -> Result<&'a str, EvaluationError> {
    match value {
        Some(Value::String(value)) => Ok(value),
        _ => Err(session.invariant("checked string operand is not a string")),
    }
}

fn strings<'a>(
    values: &'a [Value],
    session: &Session<'_>,
) -> Result<(&'a str, &'a str), EvaluationError> {
    Ok((
        string(values.first(), session)?,
        string(values.get(1), session)?,
    ))
}

fn byte_string<'a>(
    value: Option<&'a Value>,
    session: &Session<'_>,
) -> Result<&'a [u8], EvaluationError> {
    match value {
        Some(Value::Bytes(value)) => Ok(value),
        _ => Err(session.invariant("checked bytes operand is not bytes")),
    }
}

fn bytes<'a>(
    values: &'a [Value],
    session: &Session<'_>,
) -> Result<(&'a [u8], &'a [u8]), EvaluationError> {
    Ok((
        byte_string(values.first(), session)?,
        byte_string(values.get(1), session)?,
    ))
}

fn list<'a>(
    value: Option<&'a Value>,
    session: &Session<'_>,
) -> Result<&'a [Value], EvaluationError> {
    match value {
        Some(Value::List(value)) => Ok(value),
        _ => Err(session.invariant("checked list operand is not a list")),
    }
}

fn lists<'a>(
    values: &'a [Value],
    session: &Session<'_>,
) -> Result<(&'a [Value], &'a [Value]), EvaluationError> {
    Ok((
        list(values.first(), session)?,
        list(values.get(1), session)?,
    ))
}

fn i32_value(value: Option<&Value>, session: &Session<'_>) -> Result<i32, EvaluationError> {
    match value {
        Some(Value::I32(value)) => Ok(*value),
        _ => Err(session.invariant("checked i32 operand is not i32")),
    }
}

fn i64_value(value: Option<&Value>, session: &Session<'_>) -> Result<i64, EvaluationError> {
    match value {
        Some(Value::I64(value)) => Ok(*value),
        _ => Err(session.invariant("checked i64 operand is not i64")),
    }
}

fn usize_to_i64(value: usize, session: &Session<'_>) -> Result<i64, EvaluationError> {
    i64::try_from(value).map_err(|_| session.invariant("collection length does not fit i64"))
}

#[cfg(test)]
mod tests {
    use portable_check::v0::check_program;
    use portable_ir::v0::{Document, F64Bits, IrVersion, Module};

    use super::*;

    fn checked_empty() -> CheckedProgram {
        check_program(Document::new(
            IrVersion::CURRENT,
            Module {
                name: "evaluator_unit".to_owned(),
                declarations: vec![],
            },
        ))
        .expect("empty module checks")
    }

    fn evaluate(operation: Intrinsic, arguments: Vec<Value>) -> Result<Value, EvaluationError> {
        let checked = checked_empty();
        Session::new(&checked, EvaluationLimits::default()).eval_intrinsic(operation, &arguments)
    }

    #[test]
    fn portable_expectations_are_bit_exact_without_changing_ieee_language_equality() {
        let nan = Value::F64(F64Bits(f64::NAN.to_bits()));
        let positive_zero = Value::F64(F64Bits(0.0_f64.to_bits()));
        let negative_zero = Value::F64(F64Bits((-0.0_f64).to_bits()));

        assert!(!semantic_equal(&nan, &nan));
        assert!(test_equal(&nan, &nan));
        assert!(semantic_equal(&positive_zero, &negative_zero));
        assert!(!test_equal(&positive_zero, &negative_zero));
        assert!(test_equal(
            &Value::List(vec![nan.clone(), negative_zero.clone()]),
            &Value::List(vec![nan, negative_zero]),
        ));
    }

    #[test]
    fn semantic_vector_corpus_covers_more_than_twenty_operations_and_faults() {
        use Intrinsic::*;
        let nan = Value::F64(F64Bits(f64::NAN.to_bits()));
        let positive_zero = Value::F64(F64Bits(0.0_f64.to_bits()));
        let negative_zero = Value::F64(F64Bits((-0.0_f64).to_bits()));
        let cases = vec![
            (BoolNot, vec![Value::Bool(true)], Ok(Value::Bool(false))),
            (Equal, vec![nan.clone(), nan], Ok(Value::Bool(false))),
            (
                Equal,
                vec![positive_zero, negative_zero],
                Ok(Value::Bool(true)),
            ),
            (
                Less,
                vec![Value::String("a".into()), Value::String("b".into())],
                Ok(Value::Bool(true)),
            ),
            (
                IntAddChecked,
                vec![Value::I32(20), Value::I32(22)],
                Ok(Value::I32(42)),
            ),
            (
                IntAddChecked,
                vec![Value::I64(i64::MAX), Value::I64(1)],
                Err(EvaluationError::CheckedOverflow { operation: "add" }),
            ),
            (
                IntDivChecked,
                vec![Value::I64(1), Value::I64(0)],
                Err(EvaluationError::DivisionByZero),
            ),
            (
                IntDivChecked,
                vec![Value::I32(i32::MIN), Value::I32(-1)],
                Err(EvaluationError::CheckedOverflow { operation: "div" }),
            ),
            (
                IntRemChecked,
                vec![Value::I32(1), Value::I32(0)],
                Err(EvaluationError::RemainderByZero),
            ),
            (
                IntAddWrapping,
                vec![Value::I32(i32::MAX), Value::I32(1)],
                Ok(Value::I32(i32::MIN)),
            ),
            (
                IntBitXor,
                vec![Value::I64(0b1010), Value::I64(0b1100)],
                Ok(Value::I64(0b0110)),
            ),
            (
                IntShiftRightChecked,
                vec![Value::I64(-8), Value::I64(2)],
                Ok(Value::I64(-2)),
            ),
            (
                IntShiftLeftChecked,
                vec![Value::I32(1), Value::I32(32)],
                Err(EvaluationError::InvalidShift {
                    amount: 32,
                    width: 32,
                }),
            ),
            (
                FloatNeg,
                vec![Value::F64(F64Bits(0.0_f64.to_bits()))],
                Ok(Value::F64(F64Bits((-0.0_f64).to_bits()))),
            ),
            (
                FloatTrunc,
                vec![Value::F64(F64Bits((-1.75_f64).to_bits()))],
                Ok(Value::F64(F64Bits((-1.0_f64).to_bits()))),
            ),
            (
                StringScalarLength,
                vec![Value::String("a🦀e\u{301}".into())],
                Ok(Value::I64(4)),
            ),
            (
                StringContains,
                vec![Value::String("x🦀y".into()), Value::String("🦀".into())],
                Ok(Value::Bool(true)),
            ),
            (
                StringStripPrefix,
                vec![Value::String("🦀value".into()), Value::String("🦀".into())],
                Ok(Value::String("value".into())),
            ),
            (
                StringStripPrefix,
                vec![Value::String("value".into()), Value::String(String::new())],
                Ok(Value::String("value".into())),
            ),
            (
                StringReplaceAll,
                vec![
                    Value::String("a🦀a".into()),
                    Value::String("a".into()),
                    Value::String("$&".into()),
                ],
                Ok(Value::String("$&🦀$&".into())),
            ),
            (
                StringReplaceAll,
                vec![
                    Value::String("a🦀".into()),
                    Value::String(String::new()),
                    Value::String("-".into()),
                ],
                Ok(Value::String("-a-🦀-".into())),
            ),
            (
                StringReplaceMany,
                vec![
                    Value::String("&amp;lt;&lt;".into()),
                    Value::String("&amp;".into()),
                    Value::String("&".into()),
                    Value::String("&lt;".into()),
                    Value::String("<".into()),
                ],
                Ok(Value::String("&lt;<".into())),
            ),
            (
                StringReplaceMany,
                vec![
                    Value::String("a🦀".into()),
                    Value::String("a".into()),
                    Value::String("X".into()),
                    Value::String(String::new()),
                    Value::String("-".into()),
                ],
                Ok(Value::String("X-🦀-".into())),
            ),
            (
                StringTruncateUtf8Bytes,
                vec![
                    Value::String("a☃🦀".into()),
                    Value::F64(F64Bits::from_f64(4.0)),
                ],
                Ok(Value::String("a☃".into())),
            ),
            (
                StringTruncateUtf8Bytes,
                vec![
                    Value::String("a☃".into()),
                    Value::F64(F64Bits::from_f64(3.5)),
                ],
                Ok(Value::String("a".into())),
            ),
            (
                StringTruncateUtf8Bytes,
                vec![
                    Value::String("a☃".into()),
                    Value::F64(F64Bits::from_f64(f64::NAN)),
                ],
                Ok(Value::String("a☃".into())),
            ),
            (
                StringTruncateUtf8Bytes,
                vec![
                    Value::String("a☃".into()),
                    Value::F64(F64Bits::from_f64(f64::NEG_INFINITY)),
                ],
                Ok(Value::String(String::new())),
            ),
            (
                StringTrimStart,
                vec![
                    Value::String("\r\n🦀\n".into()),
                    Value::String("\r\n".into()),
                ],
                Ok(Value::String("🦀\n".into())),
            ),
            (
                StringTrimEnd,
                vec![
                    Value::String("\n🦀\r\n".into()),
                    Value::String("\r\n".into()),
                ],
                Ok(Value::String("\n🦀".into())),
            ),
            (
                BytesConcat,
                vec![Value::Bytes(vec![0, 1]), Value::Bytes(vec![254, 255])],
                Ok(Value::Bytes(vec![0, 1, 254, 255])),
            ),
            (
                ListGetChecked,
                vec![Value::List(vec![Value::I64(7)]), Value::I64(-1)],
                Err(EvaluationError::IndexOutOfBounds {
                    index: -1,
                    length: 1,
                }),
            ),
            (
                ListAppend,
                vec![Value::List(vec![Value::I32(1)]), Value::I32(2)],
                Ok(Value::List(vec![Value::I32(1), Value::I32(2)])),
            ),
            (
                OptionUnwrapOr,
                vec![Value::Some(Box::new(Value::I64(5))), Value::I64(9)],
                Ok(Value::I64(5)),
            ),
            (
                OptionUnwrapOr,
                vec![Value::None, Value::I64(9)],
                Ok(Value::I64(9)),
            ),
            (
                ResultIsErr,
                vec![Value::Err(Box::new(Value::String("x".into())))],
                Ok(Value::Bool(true)),
            ),
            (
                WidenI32ToI64,
                vec![Value::I32(i32::MIN)],
                Ok(Value::I64(i64::from(i32::MIN))),
            ),
            (
                NarrowI64ToI32Checked,
                vec![Value::I64(i64::MAX)],
                Err(EvaluationError::NarrowingOutOfRange { value: i64::MAX }),
            ),
            (
                StringToUtf8,
                vec![Value::String("🦀".into())],
                Ok(Value::Bytes("🦀".as_bytes().to_vec())),
            ),
            (
                StringFromUtf8Checked,
                vec![Value::Bytes(vec![0xff])],
                Err(EvaluationError::InvalidUtf8),
            ),
        ];
        assert!(cases.len() >= 20);
        for (operation, arguments, expected) in cases {
            assert_eq!(evaluate(operation, arguments), expected, "{operation:?}");
        }
    }

    #[test]
    fn wrapping_integer_properties_match_mathematical_modulo() {
        let i32_values = [i32::MIN, -1, 0, 1, i32::MAX];
        for left in i32_values {
            for right in i32_values {
                let actual = evaluate(
                    Intrinsic::IntAddWrapping,
                    vec![Value::I32(left), Value::I32(right)],
                );
                let modulus = 1_i128 << 32;
                let unsigned = (i128::from(left) + i128::from(right)).rem_euclid(modulus);
                let signed = if unsigned >= (1_i128 << 31) {
                    unsigned - modulus
                } else {
                    unsigned
                };
                assert_eq!(actual, Ok(Value::I32(i32::try_from(signed).unwrap())));
            }
        }

        let i64_values = [i64::MIN, -1, 0, 1, i64::MAX];
        for left in i64_values {
            for right in i64_values {
                let actual = evaluate(
                    Intrinsic::IntMulWrapping,
                    vec![Value::I64(left), Value::I64(right)],
                );
                let modulus = 1_i128 << 64;
                let unsigned = (i128::from(left) * i128::from(right)).rem_euclid(modulus);
                let signed = if unsigned >= (1_i128 << 63) {
                    unsigned - modulus
                } else {
                    unsigned
                };
                assert_eq!(actual, Ok(Value::I64(i64::try_from(signed).unwrap())));
            }
        }
    }

    #[test]
    fn fuel_collection_and_call_depth_limits_are_structured() {
        let checked = checked_empty();
        let mut fuel = Session::new(
            &checked,
            EvaluationLimits {
                fuel: 0,
                ..EvaluationLimits::default()
            },
        );
        assert_eq!(
            fuel.burn(),
            Err(EvaluationError::FuelExhausted { limit: 0 })
        );

        let collection = Session::new(
            &checked,
            EvaluationLimits {
                collection_size: 2,
                ..EvaluationLimits::default()
            },
        );
        assert_eq!(
            collection.check_collection_size(3),
            Err(EvaluationError::CollectionLimitExceeded {
                limit: 2,
                requested: 3,
            })
        );

        let mut depth = Session::new(
            &checked,
            EvaluationLimits {
                call_depth: 1,
                ..EvaluationLimits::default()
            },
        );
        depth.call_depth = 1;
        assert_eq!(
            depth.enter_call(|_| Ok(())),
            Err(EvaluationError::CallDepthExceeded { limit: 1 })
        );
    }

    #[test]
    fn list_append_does_not_alias_its_input() {
        let original = Value::List(vec![Value::I64(1)]);
        let appended =
            evaluate(Intrinsic::ListAppend, vec![original.clone(), Value::I64(2)]).unwrap();
        assert_eq!(original, Value::List(vec![Value::I64(1)]));
        assert_eq!(appended, Value::List(vec![Value::I64(1), Value::I64(2)]));
    }

    #[test]
    fn expression_arguments_are_left_to_right_and_enum_patterns_bind_fields() {
        let checked = checked_empty();
        let mut session = Session::new(&checked, EvaluationLimits::default());
        let source = || portable_ir::v0::SourceRef::logical(["evaluation_order"]);
        let literal = |id, value| Expression::Literal {
            node: portable_ir::v0::NodeMeta::new(NodeId(id), source()),
            value,
        };
        let first = Expression::Intrinsic {
            node: portable_ir::v0::NodeMeta::new(NodeId(3), source()),
            operation: Intrinsic::IntDivChecked,
            arguments: vec![literal(1, Value::I64(1)), literal(2, Value::I64(0))],
        };
        let second = Expression::Intrinsic {
            node: portable_ir::v0::NodeMeta::new(NodeId(6), source()),
            operation: Intrinsic::IntAddChecked,
            arguments: vec![literal(4, Value::I64(i64::MAX)), literal(5, Value::I64(1))],
        };
        assert_eq!(
            session.eval_arguments(&[first, second], &Environment::new(), None),
            Err(EvaluationError::DivisionByZero)
        );

        let pattern = Pattern::EnumVariant {
            node: portable_ir::v0::NodeMeta::new(NodeId(7), source()),
            declaration: NodeId(10),
            variant: NodeId(11),
            bindings: vec![portable_ir::v0::FieldBinding {
                field: NodeId(12),
                binding: "payload".to_owned(),
            }],
        };
        let value = Value::Enum {
            declaration: NodeId(10),
            variant: NodeId(11),
            fields: vec![ValueField {
                field: NodeId(12),
                value: Value::String("bound".to_owned()),
            }],
        };
        let mut environment = Environment::new();
        assert_eq!(
            session.pattern_matches(&pattern, &value, &mut environment),
            Ok(true)
        );
        assert_eq!(
            environment.get("payload"),
            Some(&Value::String("bound".to_owned()))
        );
    }
}
