//! One actual counted-loop fixture; no caller-authored proof metadata.
use super::contextual_reconstruction::{fixture, key, package};
use super::*;

pub(super) struct Fixture {
    pub registry: CRegistry,
    pub file: CFileRef,
    pub function: CFunctionRef,
    pub scope: CScopeRef,
    pub body: CScopeRef,
    pub identity: CLoopRef,
    pub counter: CLocalRef,
    pub bound: CLocalRef,
}
impl Fixture {
    pub fn new() -> Self {
        let (mut registry, file, function, scope) = fixture();
        let body = registry
            .register_scope(&function, Some(&scope), key("body"))
            .unwrap();
        let identity = registry.register_loop(&scope, key("iteration")).unwrap();
        let size = CObjectType::scalar(CScalarType::Size);
        let counter = registry
            .register_local(&scope, key("counter"), size.clone())
            .unwrap();
        let bound = registry
            .register_local(
                &scope,
                key("bound"),
                size.with_constness(CConstness::Const).unwrap(),
            )
            .unwrap();
        Self {
            registry,
            file,
            function,
            scope,
            body,
            identity,
            counter,
            bound,
        }
    }
    pub fn expressions(&self) -> CExpressions<'_> {
        CExpressions::new(&self.registry)
    }
    pub fn statements(&self) -> CStatements<'_> {
        CStatements::new(&self.registry, self.function.clone()).unwrap()
    }
    pub fn size(&self, value: u64) -> CValue {
        self.expressions()
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(value)))
            .unwrap()
    }
    pub fn read(&self, local: &CLocalRef) -> CValue {
        let values = self.expressions();
        values.read(values.local(local.clone()).unwrap()).unwrap()
    }
    pub fn declare(&self, local: &CLocalRef, value: CValue) -> CStatement {
        self.statements()
            .declare(
                local.clone(),
                Some(self.expressions().expression_initializer(value).unwrap()),
            )
            .unwrap()
    }
    pub fn condition(&self) -> CValue {
        let values = self.expressions();
        values
            .numeric_conversion(
                CScalarType::Bool,
                values
                    .binary(
                        CBinaryOperator::Less,
                        self.read(&self.counter),
                        self.read(&self.bound),
                    )
                    .unwrap(),
            )
            .unwrap()
    }
    pub fn step(&self) -> CStatement {
        let values = self.expressions();
        self.statements()
            .assign(
                values.local(self.counter.clone()).unwrap(),
                values
                    .binary(CBinaryOperator::Add, self.read(&self.counter), self.size(1))
                    .unwrap(),
            )
            .unwrap()
    }
    pub fn iteration(&self, body: Vec<CStatement>) -> CStatement {
        let ast = self.statements();
        ast.counted_loop(
            self.identity.clone(),
            self.counter.clone(),
            self.bound.clone(),
            self.condition(),
            ast.block(self.body.clone(), body).unwrap(),
        )
        .unwrap()
    }
    pub fn source(&self, statements: Vec<CStatement>) -> CSourceFile {
        package(
            &self.registry,
            self.file.clone(),
            self.function.clone(),
            self.scope.clone(),
            statements,
        )
    }
    pub fn run(&self, body: Vec<CStatement>) -> Result<(), CSafetyError> {
        self.check(vec![
            self.declare(&self.counter, self.size(0)),
            self.declare(&self.bound, self.size(3)),
            self.iteration(body),
        ])
    }
    pub fn check(&self, statements: Vec<CStatement>) -> Result<(), CSafetyError> {
        self.registry
            .check_counted_loops(&[self.source(statements)])
    }
    pub fn child(&mut self, parent: &CScopeRef, name: &str) -> CScopeRef {
        self.registry
            .register_scope(&self.function, Some(parent), key(name))
            .unwrap()
    }
}
