//! Closed grammar, authenticated references and actual declaration occurrences.
use super::{E, LoopEvidence};
use crate::ast::{
    CBinaryOperator as B, CConversion, CCountedStep, CInitializerKind, CLiteral, CLocalDeclaration,
    CLocalRef, CPlaceKind, CScalarType, CScopeRef, CUnsignedLiteral, CValue, CValueKind as V,
    contextual::flow_graph::{Action, Graph},
};

pub(super) fn declaration<'a>(
    graph: &Graph<'a>,
    local: &CLocalRef,
) -> Result<&'a CLocalDeclaration, E> {
    graph
        .nodes()
        .iter()
        .find_map(|node| match node.action() {
            Action::Declare(value) if value.local() == local => Some(*value),
            _ => None,
        })
        .ok_or(E::InvalidCountedLoop)
}

pub(super) fn check(value: &LoopEvidence<'_>) -> Result<(), E> {
    // ContextFacts has already checked exact Size/const Size types, unique
    // occurrences, lexical declaration dominance and registered scope owners.
    let Some(initializer) = value.counter.initializer() else {
        return Err(E::InvalidCountedLoop);
    };
    if !matches!(initializer.kind(), CInitializerKind::Expression(v) if size(v, 0))
        || value.bound.initializer().is_none()
        || value.progress.step() != CCountedStep::One
        || value.statement.function() != value.identity.scope().function()
    {
        return Err(E::InvalidCountedLoop);
    }
    let V::Convert {
        conversion: CConversion::Numeric(CScalarType::Bool),
        operand,
    } = value.condition.kind()
    else {
        return Err(E::InvalidCountedLoop);
    };
    if !matches!(operand.kind(),
        V::Binary { operator: B::Less, left, right }
        if read(left, value.progress.counter()) && read(right, value.progress.bound()))
    {
        return Err(E::InvalidCountedLoop);
    }
    Ok(())
}

pub(super) fn step(value: &CValue, counter: &CLocalRef) -> bool {
    matches!(value.kind(), V::Binary { operator: B::Add, left, right }
        if read(left, counter) && size(right, 1))
}

fn read(value: &CValue, local: &CLocalRef) -> bool {
    matches!(value.kind(), V::Read(place) if matches!(place.kind(), CPlaceKind::Local(v) if v == local))
}

fn size(value: &CValue, expected: u64) -> bool {
    matches!(value.kind(), V::Literal(CLiteral::Unsigned(CUnsignedLiteral::Size(v))) if *v == expected)
}

pub(super) fn contains(outer: &CScopeRef, inner: &CScopeRef) -> bool {
    let mut scope = Some(inner);
    while let Some(current) = scope {
        if current == outer {
            return true;
        }
        scope = current.parent();
    }
    false
}
