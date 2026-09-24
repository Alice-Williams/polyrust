use super::{source_constant_fixture as c, source_dependency_fixture as f};
use crate::{ast::*, dialect::*};
use portable_codegen::TargetAstPackage;

pub fn read(value: &JavaImportedValue) -> JavaExpr {
    JavaExpr {
        ty: value.ty().clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::Dependency(value.clone())),
    }
}
pub fn consumer(
    crate_id: u64,
    bindings: JavaDependencyBindings,
    values: &[JavaImportedValue],
    callable: Option<&JavaImportedCallable>,
) -> TargetAstPackage<JavaDialect> {
    let mut functions: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(i, value)| f::Function {
            hash: 20 + i as u64,
            public: true,
            name: f::name(&format!("read{i}")),
            parameters: vec![],
            result: value.ty().clone(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(read(value)))]),
        })
        .collect();
    if let Some(callable) = callable {
        assert!(callable.signature().parameters.is_empty());
        functions.push(f::Function {
            hash: 80,
            public: true,
            name: f::name("callBridge"),
            parameters: vec![],
            result: callable.signature().result.clone(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: callable.signature().result.clone(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Dependency(callable.clone()),
                    receiver: None,
                    arguments: vec![],
                },
            }))]),
        });
    }
    if functions.is_empty() {
        functions = f::functions(42);
    }
    f::package_with_dependencies(crate_id, functions, bindings)
}
pub struct Fixture {
    pub owner: JavaDependencyApi,
    pub bindings: JavaDependencyBindings,
    pub values: Vec<JavaImportedValue>,
    pub callable: Option<JavaImportedCallable>,
}
impl Fixture {
    pub fn new(mixed: bool) -> Self {
        let owner = c::api(mixed);
        let mut scope = JavaDependencyScope::new();
        let mut values = vec![];
        for constant in owner.constants() {
            let (next, value) = scope.import_constant(constant.clone()).unwrap();
            scope = next;
            values.push(value);
        }
        let callable = if mixed {
            let (next, callable) = scope
                .import(owner.function(f::id(0x35c, 110)).unwrap().clone())
                .unwrap();
            scope = next;
            Some(callable)
        } else {
            None
        };
        Self {
            owner,
            bindings: scope.finish(),
            values,
            callable,
        }
    }
    pub fn draft(&self, used: bool) -> TargetAstPackage<JavaDialect> {
        consumer(
            0x500,
            self.bindings.clone(),
            if used { &self.values } else { &[] },
            self.callable.as_ref(),
        )
    }
}
