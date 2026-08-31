#![forbid(unsafe_code)]

//! Verbose Rust authoring API for the portable model.

use portable_check::{CheckedModule, Diagnostic, check};
pub use portable_ir::{
    Constant, Contract, Expression, Field, Function, Implementation, MethodSignature, Module,
    Parameter, PortableTest, Record, Type, Value,
};

pub struct ModuleBuilder {
    module: Module,
}

impl ModuleBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            module: Module {
                name: name.into(),
                constants: Vec::new(),
                records: Vec::new(),
                contracts: Vec::new(),
                implementations: Vec::new(),
                functions: Vec::new(),
                tests: Vec::new(),
            },
        }
    }

    pub fn constant(&mut self, name: impl Into<String>, ty: Type, value: Value) -> &mut Self {
        self.module.constants.push(Constant {
            name: name.into(),
            ty,
            value,
        });
        self
    }

    pub fn record(&mut self, record: Record) -> &mut Self {
        self.module.records.push(record);
        self
    }

    pub fn contract(&mut self, contract: Contract) -> &mut Self {
        self.module.contracts.push(contract);
        self
    }

    pub fn implementation(&mut self, implementation: Implementation) -> &mut Self {
        self.module.implementations.push(implementation);
        self
    }

    pub fn function(&mut self, function: Function) -> &mut Self {
        self.module.functions.push(function);
        self
    }

    pub fn portable_test(&mut self, test: PortableTest) -> &mut Self {
        self.module.tests.push(test);
        self
    }

    pub fn finish_unchecked(self) -> Module {
        self.module
    }

    pub fn finish(self) -> Result<CheckedModule, Vec<Diagnostic>> {
        check(self.module)
    }
}
