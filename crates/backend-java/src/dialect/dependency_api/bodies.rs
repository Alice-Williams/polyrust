//! Closed local-only body admission; a signature's pure flag is not evidence.
mod expressions;
mod results;
use super::inventory::{scalar, signature};
use crate::ast::{
    JavaBlock, JavaCallableRef, JavaExpr, JavaExprKind, JavaIdentifier, JavaLocalFinality,
    JavaMethod, JavaPrimitive, JavaStmt, JavaType, JavaTypeDeclaration, JavaTypeName, JavaValueRef,
};
use portable_codegen::{GeneratedCallableId, GeneratedTypeId, GeneratedValueId};
use std::collections::{BTreeMap, BTreeSet};

const MAX_FUNCTIONS: usize = 4096;
const MAX_STEPS: usize = 100_000;
const MAX_DEPTH: usize = 128;

pub(super) struct Budget {
    remaining: usize,
}

impl Budget {
    pub(super) fn new() -> Self {
        Self {
            remaining: MAX_STEPS,
        }
    }
    pub(super) fn charge(&mut self, depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err("Java dependency body depth limit exceeded".into());
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or("Java dependency body visit limit exceeded")?;
        Ok(())
    }
}

#[cfg(test)]
pub(super) fn verify(
    methods: &BTreeMap<GeneratedCallableId, &JavaMethod>,
    records: &BTreeMap<GeneratedTypeId, &JavaTypeDeclaration>,
    constants: &BTreeMap<GeneratedValueId, JavaType>,
    budget: &mut Budget,
) -> Result<BTreeMap<GeneratedCallableId, usize>, String> {
    verify_with_results(
        methods,
        records,
        constants,
        &super::result_profile::ResultProfile::default(),
        budget,
    )
}
pub(super) fn verify_with_results(
    methods: &BTreeMap<GeneratedCallableId, &JavaMethod>,
    records: &BTreeMap<GeneratedTypeId, &JavaTypeDeclaration>,
    constants: &BTreeMap<GeneratedValueId, JavaType>,
    results: &super::result_profile::ResultProfile,
    budget: &mut Budget,
) -> Result<BTreeMap<GeneratedCallableId, usize>, String> {
    if methods.len() > MAX_FUNCTIONS {
        return Err("Java dependency function limit exceeded".into());
    }
    let mut reader = Reader {
        methods,
        records,
        constants,
        budget,
        calls: BTreeSet::new(),
        imported_height: 0,
        mutable_bools: BTreeSet::new(),
        results,
        nonnull_results: BTreeSet::new(),
    };
    let mut edges = BTreeMap::new();
    let mut bases = BTreeMap::new();
    for (id, method) in methods {
        reader.calls.clear();
        reader.imported_height = 0;
        reader.mutable_bools.clear();
        reader.nonnull_results.clear();
        reader.block(
            method
                .body
                .as_ref()
                .ok_or("Java dependency method lacks a body")?,
            0,
        )?;
        edges.insert(*id, reader.calls.clone());
        bases.insert(*id, reader.imported_height + 1);
    }
    call_heights_with_bases(&edges, &bases)
}

struct Reader<'a> {
    methods: &'a BTreeMap<GeneratedCallableId, &'a JavaMethod>,
    records: &'a BTreeMap<GeneratedTypeId, &'a JavaTypeDeclaration>,
    constants: &'a BTreeMap<GeneratedValueId, JavaType>,
    budget: &'a mut Budget,
    calls: BTreeSet<GeneratedCallableId>,
    imported_height: usize,
    mutable_bools: BTreeSet<JavaIdentifier>,
    results: &'a super::result_profile::ResultProfile,
    nonnull_results: BTreeSet<JavaIdentifier>,
}

impl Reader<'_> {
    fn charge(&mut self, depth: usize) -> Result<(), String> {
        self.budget.charge(depth)
    }
    fn ty(&self, ty: &JavaType) -> bool {
        scalar(ty)
            || self.results.role(ty).is_some()
            || matches!(ty, JavaType::Reference(JavaTypeName::Generated(id)) if self.records.contains_key(id))
    }
    fn block(&mut self, block: &JavaBlock, depth: usize) -> Result<(), String> {
        self.charge(depth)?;
        let outer_mutable_bools = self.mutable_bools.clone();
        let outer_nonnull_results = self.nonnull_results.clone();
        for statement in &block.statements {
            self.charge(depth)?;
            match statement {
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty,
                    name,
                    value: Some(value),
                } if self.ty(ty) => {
                    self.expression(value, depth + 1)?;
                    if self.results.role(ty).is_some() {
                        self.nonnull_results.insert(name.clone());
                    }
                }
                JavaStmt::Local {
                    finality: JavaLocalFinality::Mutable,
                    ty,
                    name,
                    value: Some(value),
                } if *ty == JavaType::primitive(JavaPrimitive::Boolean) => {
                    self.expression(value, depth + 1)?;
                    self.mutable_bools.insert(name.clone());
                }
                JavaStmt::Assign { target, value }
                    if target.ty == JavaType::primitive(JavaPrimitive::Boolean)
                        && value.ty == target.ty
                        && matches!(&target.kind, JavaExprKind::Value(JavaValueRef::Local(name)) if self.mutable_bools.contains(name)) =>
                {
                    self.expression(value, depth + 1)?;
                }
                JavaStmt::Return(Some(value)) => self.expression(value, depth + 1)?,
                JavaStmt::Return(None) => {}
                JavaStmt::Expression(value) => self.effect(value, depth + 1)?,
                JavaStmt::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    self.expression(condition, depth + 1)?;
                    let branch_nonnull = self.nonnull_results.clone();
                    if let JavaExprKind::InstanceOf {
                        binding: Some(binding),
                        target,
                        ..
                    } = &condition.kind
                        && self.results.variant(target)
                    {
                        self.nonnull_results.insert(binding.clone());
                    }
                    self.block(then_block, depth + 1)?;
                    self.nonnull_results = branch_nonnull;
                    if let Some(else_block) = else_block {
                        self.block(else_block, depth + 1)?;
                    }
                }
                _ => return Err("Java dependency body contains an unadmitted statement".into()),
            }
        }
        self.mutable_bools = outer_mutable_bools;
        self.nonnull_results = outer_nonnull_results;
        Ok(())
    }
    fn effect(&mut self, value: &JavaExpr, depth: usize) -> Result<(), String> {
        self.charge(depth)?;
        if value.ty != JavaType::primitive(JavaPrimitive::Void) {
            return Err("Java dependency effect requires a void call".into());
        }
        let JavaExprKind::Call {
            callable,
            receiver: None,
            arguments,
        } = &value.kind
        else {
            return Err("Java dependency effect requires a direct call".into());
        };
        self.call(callable, arguments, depth)
    }
    fn call(
        &mut self,
        callable: &JavaCallableRef,
        arguments: &[JavaExpr],
        depth: usize,
    ) -> Result<(), String> {
        match callable {
            JavaCallableRef::Generated {
                symbol,
                signature: actual,
            } => {
                let method = self
                    .methods
                    .get(symbol)
                    .ok_or("Java dependency call has no closed local definition")?;
                if actual != &signature(method) {
                    return Err(
                        "Java dependency call signature disagrees with its definition".into(),
                    );
                }
                self.calls.insert(*symbol);
            }
            JavaCallableRef::Dependency(callable) => {
                self.imported_height = self.imported_height.max(callable.function().call_height());
            }
            JavaCallableRef::Known {
                callable,
                signature,
            } if matches!(
                callable,
                crate::dialect::JavaKnownCallable::MathFloor
                    | crate::dialect::JavaKnownCallable::MathCeil
            ) && signature == &callable.signature() => {}
            _ => return Err("Java dependency call requires a certified direct target".into()),
        }
        for argument in arguments {
            self.expression(argument, depth + 1)?;
        }
        Ok(())
    }
}

/// Bottom-up graph traversal avoids recursive/exponential proof walks.
#[cfg(test)]
fn call_heights(
    edges: &BTreeMap<GeneratedCallableId, BTreeSet<GeneratedCallableId>>,
) -> Result<BTreeMap<GeneratedCallableId, usize>, String> {
    call_heights_with_bases(edges, &BTreeMap::new())
}

fn call_heights_with_bases(
    edges: &BTreeMap<GeneratedCallableId, BTreeSet<GeneratedCallableId>>,
    bases: &BTreeMap<GeneratedCallableId, usize>,
) -> Result<BTreeMap<GeneratedCallableId, usize>, String> {
    let mut pending = edges
        .iter()
        .map(|(id, calls)| (*id, calls.len()))
        .collect::<BTreeMap<_, _>>();
    let mut callers = BTreeMap::<_, Vec<_>>::new();
    for (caller, calls) in edges {
        for callee in calls {
            callers.entry(*callee).or_default().push(*caller);
        }
    }
    let mut ready = pending
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<Vec<_>>();
    let mut heights = edges
        .keys()
        .map(|id| (*id, bases.get(id).copied().unwrap_or(1)))
        .collect::<BTreeMap<_, _>>();
    let mut visited = 0;
    while let Some(id) = ready.pop() {
        visited += 1;
        let height = heights[&id];
        if height > MAX_DEPTH {
            return Err("Java dependency call height limit exceeded".into());
        }
        for caller in callers.get(&id).into_iter().flatten() {
            heights
                .entry(*caller)
                .and_modify(|value| *value = (*value).max(height + 1));
            let count = pending
                .get_mut(caller)
                .expect("caller belongs to complete graph");
            *count -= 1;
            if *count == 0 {
                ready.push(*caller);
            }
        }
    }
    if visited != edges.len() {
        return Err("Java dependency recursive call graph is not admitted".into());
    }
    Ok(heights)
}

#[cfg(test)]
#[path = "../../tests/dependency_body_limits.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/dependency_i64_bodies.rs"]
mod i64_tests;

#[cfg(test)]
#[path = "../../tests/dependency_bitwise_bodies.rs"]
mod bitwise_tests;

#[cfg(test)]
#[path = "../../tests/dependency_arithmetic_bodies.rs"]
mod arithmetic_tests;

#[cfg(test)]
#[path = "../../tests/dependency_signed_widening.rs"]
mod signed_widening_tests;

#[cfg(test)]
#[path = "../../tests/result_body_limits.rs"]
mod result_tests;
