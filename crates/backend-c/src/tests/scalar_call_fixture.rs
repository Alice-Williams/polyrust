//! Registered functions for effect-summary tests; no fabricated proof payloads.
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

pub(super) struct Fixture {
    pub registry: CRegistry,
    pub file: CFileRef,
    pub functions: Vec<CFunctionRef>,
    pub scopes: Vec<CScopeRef>,
    pub parameters: Vec<Vec<CParameterRef>>,
}

impl Fixture {
    pub fn new(arities: &[usize]) -> Self {
        Self::typed(
            &arities
                .iter()
                .map(|n| vec![CObjectType::scalar(CScalarType::I32); *n])
                .collect::<Vec<_>>(),
        )
    }
    pub fn typed(parameters: &[Vec<CObjectType>]) -> Self {
        Self::with_result(parameters, CObjectType::scalar(CScalarType::I32))
    }
    pub fn with_result(parameters: &[Vec<CObjectType>], result: CObjectType) -> Self {
        let mut registry = CRegistry::new();
        let file = registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new("tests/scalar_calls.c").unwrap(),
                role: CFileRole::TestSource,
            })
            .unwrap();
        let mut functions = vec![];
        let mut scopes = vec![];
        let mut registered = vec![];
        for (index, types) in parameters.iter().enumerate() {
            let function = registry
                .register_function(
                    &file,
                    key(&format!("function{index}")),
                    CFunctionType::new(
                        CReturnType::Value(CReturnValue::new(result.clone()).unwrap()),
                        types
                            .iter()
                            .cloned()
                            .map(|ty| CParameterType::new(ty).unwrap())
                            .collect(),
                    ),
                )
                .unwrap();
            scopes.push(
                registry
                    .register_scope(&function, None, key(&format!("scope{index}")))
                    .unwrap(),
            );
            registered.push(
                types
                    .iter()
                    .enumerate()
                    .map(|(parameter, _)| {
                        registry
                            .register_parameter(
                                &function,
                                parameter,
                                key(&format!("input{parameter}")),
                                CConstness::Unqualified,
                            )
                            .unwrap()
                    })
                    .collect(),
            );
            functions.push(function);
        }
        Self {
            registry,
            file,
            functions,
            scopes,
            parameters: registered,
        }
    }
    pub fn values(&self) -> CExpressions<'_> {
        CExpressions::new(&self.registry)
    }
    pub fn statements(&self, index: usize) -> CStatements<'_> {
        CStatements::new(&self.registry, self.functions[index].clone()).unwrap()
    }
    pub fn input(&self, function: usize, index: usize) -> CValue {
        self.values()
            .read(
                self.values()
                    .parameter(self.parameters[function][index].clone())
                    .unwrap(),
            )
            .unwrap()
    }
    pub fn literal(&self, number: i32) -> CValue {
        self.values()
            .literal(CLiteral::Signed(CSignedLiteral::I32(number)))
            .unwrap()
    }
    pub fn call(&self, function: usize, arguments: Vec<CValue>) -> CValue {
        self.values()
            .call_value(
                self.values()
                    .direct(self.functions[function].clone())
                    .unwrap(),
                arguments,
            )
            .unwrap()
    }
    pub fn returning(&self, function: usize, value: CValue) -> Vec<CStatement> {
        vec![
            self.statements(function)
                .return_statement(Some(value))
                .unwrap(),
        ]
    }
    pub fn source(&self, bodies: Vec<Vec<CStatement>>) -> CSourceFile {
        let declarations = CDeclarations::new(&self.registry, self.file.clone()).unwrap();
        let mut items: Vec<_> = self
            .functions
            .iter()
            .map(|function| {
                CFileItem::Declaration(
                    declarations
                        .function_prototype(function.clone(), CLinkage::External)
                        .unwrap(),
                )
            })
            .collect();
        for (index, body) in bodies.into_iter().enumerate() {
            let body = self
                .statements(index)
                .block(self.scopes[index].clone(), body)
                .unwrap();
            items.push(CFileItem::Definition(
                declarations
                    .function_definition(
                        self.functions[index].clone(),
                        CLinkage::External,
                        self.parameters[index].clone(),
                        body,
                    )
                    .unwrap(),
            ));
        }
        declarations.source_file(items).unwrap()
    }
}
