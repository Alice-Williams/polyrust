//! Actual immutable bindings and pure products; never descriptor-authored capacity.
use crate::ast::{
    CBinaryOperator, CBufferCountRef, CConstness, CConversion, CInitializerKind, CLocalRef,
    CObjectType, CObjectTypeKind, CPlaceKind, CRegistry, CScalarType, CValue, CValueKind,
    contextual::flow_graph::{Action, Graph},
};
use crate::ownership::{
    constants,
    layout::Layouts,
    numeric_flow::{State, storage::Key},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Products {
    candidates: BTreeMap<(CBufferCountRef, u64), (u64, u64)>,
}
impl Products {
    pub(super) fn actual(
        registry: &CRegistry,
        graph: &Graph<'_>,
        bytes: &CValue,
        state: &State<'_>,
    ) -> Self {
        let initializers = Initializers::actual(graph);
        let mut result = Self::default();
        let mut pending = vec![(bytes, 1_u64)];
        let mut visited = BTreeSet::new();
        let mut layouts = Layouts::new(registry);
        while let Some((value, stride)) = pending.pop() {
            if !size_type(value.ty()) || !visited.insert((std::ptr::from_ref(value), stride)) {
                continue;
            }
            match value.kind() {
                CValueKind::Read(place) => {
                    let CPlaceKind::Local(local) = place.kind() else {
                        continue;
                    };
                    if let Ok(count) = registry.buffer_count_reference(local)
                        && let Ok(bounds) = state
                            .number(&Key::local(local), CScalarType::Size)
                            .and_then(|number| number.extent_bounds())
                    {
                        result.candidates.insert((count, stride), bounds);
                    }
                    if let Some(initializer) = initializers.get(local) {
                        pending.push((initializer, stride));
                    }
                }
                CValueKind::Convert {
                    conversion: CConversion::Numeric(_),
                    operand,
                } if size_type(operand.ty()) => pending.push((operand, stride)),
                CValueKind::Binary {
                    operator: CBinaryOperator::Multiply,
                    left,
                    right,
                } => {
                    for (factor, operand) in [(left, right), (right, left)] {
                        if let Some(factor) = initializers.constant(&mut layouts, factor)
                            && let Some(product) = stride.checked_mul(factor)
                        {
                            pending.push((operand, product));
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }

    pub(super) fn bounds(&self, count: &CBufferCountRef, stride: u64) -> Option<(u64, u64)> {
        self.candidates.get(&(count.clone(), stride)).copied()
    }

    pub(super) fn valid_for(&self, function: &crate::ast::CFunctionRef) -> bool {
        self.candidates
            .iter()
            .all(|((count, stride), (first, last))| {
                count.local().scope().function() == function && *stride > 0 && first <= last
            })
    }

    pub(super) fn join(&self, other: &Self) -> Self {
        Self {
            candidates: self
                .candidates
                .iter()
                .filter_map(|(key, left)| {
                    let right = other.candidates.get(key)?;
                    Some((key.clone(), (left.0.min(right.0), left.1.max(right.1))))
                })
                .collect(),
        }
    }
}

struct Initializers<'a> {
    values: BTreeMap<&'a CLocalRef, &'a CValue>,
}
impl<'a> Initializers<'a> {
    fn actual(graph: &Graph<'a>) -> Self {
        let mut values = BTreeMap::new();
        for node in graph.nodes() {
            if let Action::Declare(declaration) = node.action()
                && let Some(initializer) = declaration.initializer()
                && let CInitializerKind::Expression(value) = initializer.kind()
                && declaration.local().ty().canonical().constness() == CConstness::Const
                && size_type(declaration.local().ty())
            {
                values.insert(declaration.local(), value);
            }
        }
        Self { values }
    }

    fn get(&self, local: &CLocalRef) -> Option<&'a CValue> {
        self.values.get(local).copied()
    }

    fn constant<'b>(&'b self, layouts: &mut Layouts<'_>, mut value: &'b CValue) -> Option<u64> {
        let mut visited = BTreeSet::new();
        loop {
            if !size_type(value.ty()) || !visited.insert(std::ptr::from_ref(value)) {
                return None;
            }
            match value.kind() {
                CValueKind::Read(place) => {
                    let CPlaceKind::Local(local) = place.kind() else {
                        return None;
                    };
                    value = self.get(local)?;
                }
                CValueKind::Convert {
                    conversion: CConversion::Numeric(_),
                    operand,
                } if size_type(operand.ty()) => value = operand,
                _ => {
                    let number = constants::evaluate(layouts, value).ok()?.integer().ok()?;
                    return u64::try_from(number.value())
                        .ok()
                        .filter(|value| *value > 0);
                }
            }
        }
    }
}

fn size_type(ty: &CObjectType) -> bool {
    matches!(
        ty.canonical().kind(),
        CObjectTypeKind::Scalar(CScalarType::Size | CScalarType::U64)
    )
}
