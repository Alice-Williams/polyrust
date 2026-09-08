//! Conservative pinned-javac reservations; see java/compiler-budget.md.

mod classes;
mod expressions;
mod statements;

use crate::ast::{JavaMember, JavaType, JavaTypeDeclaration};
use portable_diagnostics::Diagnostic;

#[derive(Clone, Debug, Default)]
pub(crate) struct Code {
    pub name: String,
    pub bytes: usize,
    pub pool: usize,
    pub locals: usize,
    pub stack: usize,
    pub exceptions: usize,
    pub bootstraps: usize,
    pub bootstrap_arguments: usize,
    pub helper: Helper,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Helper {
    pub switches: usize,
    pub arms: usize,
}

impl Helper {
    fn add(&mut self, other: &Self) {
        self.switches = self.switches.saturating_add(other.switches);
        self.arms = self.arms.saturating_add(other.arms);
    }
}

impl Code {
    fn add(&mut self, other: &Self) {
        self.bytes = self.bytes.saturating_add(other.bytes);
        self.pool = self.pool.saturating_add(other.pool);
        self.locals = self.locals.saturating_add(other.locals);
        self.stack = self.stack.saturating_add(other.stack);
        self.exceptions = self.exceptions.saturating_add(other.exceptions);
        self.bootstraps = self.bootstraps.saturating_add(other.bootstraps);
        self.bootstrap_arguments = self.bootstrap_arguments.max(other.bootstrap_arguments);
        self.helper.add(&other.helper);
    }

    fn ty(&mut self, ty: &JavaType) {
        self.pool = self.pool.saturating_add(type_pool(ty));
    }

    fn argument(&mut self) {
        self.bytes = self.bytes.saturating_add(16);
        self.pool = self.pool.saturating_add(8);
        self.stack = self.stack.saturating_add(2);
    }

    fn method(&mut self, parameters: usize) {
        self.bytes = self.bytes.saturating_add(64);
        self.pool = self.pool.saturating_add(32);
        self.locals = self.locals.saturating_add(parameters).saturating_add(4);
        self.stack = self.stack.saturating_add(8);
    }
}

fn type_pool(ty: &JavaType) -> usize {
    let children = match ty {
        JavaType::Primitive(_)
        | JavaType::Boxed(_)
        | JavaType::Reference(_)
        | JavaType::TypeVariable(_) => 0,
        JavaType::Array { component, .. } => type_pool(component),
        JavaType::Generic { arguments, .. } => arguments
            .iter()
            .fold(0usize, |total, ty| total.saturating_add(type_pool(ty))),
        JavaType::Wildcard { bound } => bound.as_ref().map_or(0, |(_, ty)| type_pool(ty)),
    };
    8usize.saturating_add(children)
}

#[derive(Debug)]
pub(crate) enum ClassFileKind {
    Declared,
    EnumSwitchHelper,
}

#[derive(Debug)]
pub(crate) struct ClassBudget {
    pub kind: ClassFileKind,
    pub name: String,
    pub pool: usize,
    pub fields: usize,
    pub methods: Vec<Code>,
    pub metadata: usize,
}

impl ClassBudget {
    fn method(&mut self, name: &str, mut code: Code, parameters: usize) {
        code.name = name.to_owned();
        code.method(parameters);
        self.pool = self.pool.saturating_add(code.pool);
        self.methods.push(code);
    }

    fn validate(&self, path: &str, errors: &mut Vec<Diagnostic>) {
        let path = format!("{path}:{}", self.name);
        for (label, count, max) in [
            ("constant-pool", self.pool, 65_534),
            ("field", self.fields, 65_535),
            ("method", self.methods.len(), 65_535),
            ("inner/nest metadata", self.metadata, 65_535),
        ] {
            super::limit(
                errors,
                &path,
                &format!("conservative {label} budget"),
                count,
                max,
            );
        }
        let mut bootstraps = 0usize;
        for method in &self.methods {
            bootstraps = bootstraps.saturating_add(method.bootstraps);
            let method_path = format!("{path}::{}", method.name);
            for (label, count) in [
                ("method code bytes", method.bytes),
                ("local slots", method.locals),
                ("stack slots", method.stack),
                ("exception entries", method.exceptions),
                ("stack-map entries", method.bytes),
                ("bootstrap arguments", method.bootstrap_arguments),
            ] {
                super::limit(
                    errors,
                    &method_path,
                    &format!("conservative {label} budget"),
                    count,
                    65_535,
                );
            }
        }
        super::limit(
            errors,
            &path,
            "conservative bootstrap entries budget",
            bootstraps,
            65_535,
        );
    }
}

pub(crate) fn declaration_count(value: &JavaTypeDeclaration) -> usize {
    value.members.iter().fold(1usize, |count, member| {
        count.saturating_add(match member {
            JavaMember::NestedType(child) => declaration_count(child),
            JavaMember::Field(_)
            | JavaMember::CompileFailField(_)
            | JavaMember::EnumConstant(_)
            | JavaMember::Method(_)
            | JavaMember::Constructor(_) => 0,
        })
    })
}

pub(super) fn check(
    declaration: &JavaTypeDeclaration,
    members: &[&JavaMember],
    type_count: usize,
    binary_name_length: usize,
    path: &str,
    errors: &mut Vec<Diagnostic>,
) {
    for class in classes::report(declaration, members, type_count) {
        if matches!(class.kind, ClassFileKind::EnumSwitchHelper) {
            // Each occupied numeric suffix requires a declared class. The
            // single map class per outer nest therefore needs at most the
            // digits of (package declaration count + 1), even with '$' names.
            let suffix_bytes =
                1usize.saturating_add(type_count.saturating_add(1).to_string().len());
            super::limit(
                errors,
                path,
                "conservative enum-switch helper binary name bytes",
                binary_name_length.saturating_add(suffix_bytes),
                super::types::MAX_UTF8,
            );
        }
        class.validate(path, errors);
    }
}

#[cfg(test)]
pub(crate) use classes::report;
