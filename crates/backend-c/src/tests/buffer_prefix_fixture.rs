//! Small actual counted-loop fixtures; steps are ordinary supplied AST actions.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};

pub(crate) struct Counted {
    pub counter: CLocalRef,
    pub scope: CScopeRef,
    pub identity: CLoopRef,
}
impl Counted {
    pub fn new(f: &mut Fixture, name: &str) -> Self {
        let counter = f.local(CScalarType::Size, &format!("{name}_counter"));
        let scope = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key(&format!("{name}_body")))
            .unwrap();
        let identity = f.registry.register_loop(&f.scope, key(name)).unwrap();
        Self {
            counter,
            scope,
            identity,
        }
    }
    pub fn declaration(&self, f: &Fixture) -> CStatement {
        f.declare(&self.counter, f.size(0))
    }
    pub fn step(&self, f: &Fixture) -> CStatement {
        f.ast()
            .assign(
                f.values().local(self.counter.clone()).unwrap(),
                f.binary(CBinaryOperator::Add, f.read(&self.counter), f.size(1)),
            )
            .unwrap()
    }
    pub fn finish(&self, f: &Fixture, bound: &CLocalRef, actions: Vec<CStatement>) -> CStatement {
        f.ast()
            .counted_loop(
                self.identity.clone(),
                self.counter.clone(),
                bound.clone(),
                f.compare(CBinaryOperator::Less, f.read(&self.counter), f.read(bound)),
                f.ast().block(self.scope.clone(), actions).unwrap(),
            )
            .unwrap()
    }
    pub fn branch(
        &self,
        f: &mut Fixture,
        condition: CValue,
        yes: Vec<CStatement>,
        no: Vec<CStatement>,
    ) -> CStatement {
        let outer = std::mem::replace(&mut f.scope, self.scope.clone());
        let branch = f.branch(condition, yes, no);
        f.scope = outer;
        branch
    }
    pub fn break_now(&self, f: &Fixture) -> CStatement {
        f.ast()
            .break_statement(CBreakTarget::Loop(self.identity.clone()))
            .unwrap()
    }
    pub fn continue_now(&self, f: &Fixture) -> CStatement {
        f.ast().continue_statement(self.identity.clone()).unwrap()
    }
}
pub(crate) fn immutable_size(f: &mut Fixture, name: &str) -> CLocalRef {
    f.registry
        .register_local(
            &f.scope,
            key(name),
            CObjectType::scalar(CScalarType::Size)
                .with_constness(CConstness::Const)
                .unwrap(),
        )
        .unwrap()
}
