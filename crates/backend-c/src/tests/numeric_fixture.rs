//! Runtime parameters prevent guards from being accidentally constant fixtures.
use super::contextual_reconstruction::key;
use super::*;
use portable_codegen::RelativeOutputPath;

pub(crate) struct Fixture {
    pub registry: CRegistry,
    pub file: CFileRef,
    pub function: CFunctionRef,
    pub scope: CScopeRef,
    pub parameters: Vec<CParameterRef>,
    serial: usize,
}
impl Fixture {
    pub fn new(types: &[CScalarType]) -> Self {
        let mut registry = CRegistry::new();
        let file = registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new("src/numeric.c").unwrap(),
                role: CFileRole::TestSource,
            })
            .unwrap();
        let signature = CFunctionType::new(
            CReturnType::Void,
            types
                .iter()
                .map(|ty| CParameterType::new(CObjectType::scalar(*ty)).unwrap())
                .collect(),
        );
        let function = registry
            .register_function(&file, key("run"), signature)
            .unwrap();
        let scope = registry
            .register_scope(&function, None, key("root"))
            .unwrap();
        let parameters = types
            .iter()
            .enumerate()
            .map(|(index, _)| {
                registry
                    .register_parameter(
                        &function,
                        index,
                        key(&format!("input_{index}")),
                        CConstness::Unqualified,
                    )
                    .unwrap()
            })
            .collect();
        Self {
            registry,
            file,
            function,
            scope,
            parameters,
            serial: 0,
        }
    }
    pub fn values(&self) -> CExpressions<'_> {
        CExpressions::new(&self.registry)
    }
    pub fn ast(&self) -> CStatements<'_> {
        CStatements::new(&self.registry, self.function.clone()).unwrap()
    }
    pub fn input(&self, index: usize) -> CValue {
        self.values()
            .read(
                self.values()
                    .parameter(self.parameters[index].clone())
                    .unwrap(),
            )
            .unwrap()
    }
    pub fn int(&self, value: i32) -> CValue {
        self.values()
            .literal(CLiteral::Signed(CSignedLiteral::Int(value)))
            .unwrap()
    }
    pub fn size(&self, value: u64) -> CValue {
        self.values()
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(value)))
            .unwrap()
    }
    pub fn boolean(&self, value: CValue) -> CValue {
        self.values()
            .numeric_conversion(CScalarType::Bool, value)
            .unwrap()
    }
    pub fn binary(&self, operator: CBinaryOperator, left: CValue, right: CValue) -> CValue {
        self.values().binary(operator, left, right).unwrap()
    }
    pub fn compare(&self, operator: CBinaryOperator, left: CValue, right: CValue) -> CValue {
        self.boolean(self.binary(operator, left, right))
    }
    pub fn discard(&self, value: CValue) -> CStatement {
        self.ast().discard(value).unwrap()
    }
    pub fn local(&mut self, ty: CScalarType, name: &str) -> CLocalRef {
        self.registry
            .register_local(&self.scope, key(name), CObjectType::scalar(ty))
            .unwrap()
    }
    pub fn declare(&self, local: &CLocalRef, value: CValue) -> CStatement {
        self.ast()
            .declare(
                local.clone(),
                Some(self.values().expression_initializer(value).unwrap()),
            )
            .unwrap()
    }
    pub fn read(&self, local: &CLocalRef) -> CValue {
        self.values()
            .read(self.values().local(local.clone()).unwrap())
            .unwrap()
    }
    pub fn branch(
        &mut self,
        condition: CValue,
        yes: Vec<CStatement>,
        no: Vec<CStatement>,
    ) -> CStatement {
        let then_scope = self
            .registry
            .register_scope(
                &self.function,
                Some(&self.scope),
                key(&format!("then_{}", self.serial)),
            )
            .unwrap();
        let else_scope = self
            .registry
            .register_scope(
                &self.function,
                Some(&self.scope),
                key(&format!("else_{}", self.serial)),
            )
            .unwrap();
        self.serial += 1;
        self.ast()
            .if_statement(
                condition,
                self.ast().block(then_scope, yes).unwrap(),
                self.ast().block(else_scope, no).unwrap(),
            )
            .unwrap()
    }
    pub fn source(&self, body: Vec<CStatement>) -> CSourceFile {
        let declarations = CDeclarations::new(&self.registry, self.file.clone()).unwrap();
        let definition = declarations
            .function_definition(
                self.function.clone(),
                CLinkage::External,
                self.parameters.clone(),
                self.ast().block(self.scope.clone(), body).unwrap(),
            )
            .unwrap();
        declarations
            .source_file(vec![CFileItem::Definition(definition)])
            .unwrap()
    }
    pub fn check(&self, body: Vec<CStatement>) -> Result<(), CSafetyError> {
        self.registry.check_numeric_flow(&[self.source(body)])
    }
}
