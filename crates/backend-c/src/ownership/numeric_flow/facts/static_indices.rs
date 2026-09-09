//! Static address indices have initializer origins, never invented CFG points.
use crate::ast::{
    CConversion, CDefinitionKind, CFileItem, CIndexBase, CInitializer, CInitializerKind, CLiteral,
    CObjectTypeKind, CPlace, CPlaceKind, CValue, CValueKind,
};
use crate::ownership::{
    CSafetyError as E, constants,
    context_facts::ContextFacts,
    layout::Layouts,
    numeric_flow::{
        Engine, Mode,
        state::{Number, State},
    },
};
use std::collections::BTreeSet;

pub(super) struct Location<'a> {
    pub(super) initializer: &'a CInitializer,
    pub(super) place: &'a CPlace,
}
pub(super) struct Observation<'a> {
    pub(super) location: Location<'a>,
    pub(super) number: Number<'a>,
}

pub(super) fn check<'a>(context: &ContextFacts<'a>) -> Result<Vec<Observation<'a>>, E> {
    let mut engine = Engine {
        registry: context.registry(),
        layouts: Layouts::new(context.registry()),
        addresses: BTreeSet::new(),
        mode: Mode::Derive,
        obligations: vec![],
    };
    let mut observations = vec![];
    for location in locations(context)? {
        let CPlaceKind::Index { index, .. } = location.place.kind() else {
            return Err(E::InvalidNumericSite);
        };
        // The static grammar/constant model, not an empty runtime environment,
        // establishes that this operand is an actual integer constant tree.
        constants::evaluate(&mut engine.layouts, index)?.integer()?;
        let number = engine.numeric(index, &mut State::default())?;
        observations.push(Observation { location, number });
    }
    Ok(observations)
}

pub(super) fn validate(
    context: &ContextFacts<'_>,
    observations: &[Observation<'_>],
) -> Result<(), E> {
    let actual = locations(context)?;
    if actual.len() != observations.len()
        || actual.iter().zip(observations).any(|(a, b)| {
            !std::ptr::eq(a.initializer, b.location.initializer)
                || !std::ptr::eq(a.place, b.location.place)
        })
    {
        return Err(E::InvalidNumericSite);
    }
    Ok(())
}

fn locations<'a>(context: &ContextFacts<'a>) -> Result<Vec<Location<'a>>, E> {
    let mut output = vec![];
    for file in context.files() {
        for item in file.items() {
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Object {
                    initializer: value, ..
                } = definition.kind()
            {
                initializer(value, value, &mut output)?;
            }
        }
    }
    Ok(output)
}

fn initializer<'a>(
    root: &'a CInitializer,
    value: &'a CInitializer,
    output: &mut Vec<Location<'a>>,
) -> Result<(), E> {
    match value.kind() {
        CInitializerKind::Expression(value)
            if matches!(value.ty().kind(), CObjectTypeKind::Pointer(_)) =>
        {
            address(root, value, output)?
        }
        CInitializerKind::Expression(_) | CInitializerKind::Zero(_) => {}
        CInitializerKind::Array { elements, .. } => {
            for value in elements {
                initializer(root, value, output)?;
            }
        }
        CInitializerKind::Struct { members, .. } => {
            for (_, value) in members {
                initializer(root, value, output)?;
            }
        }
        CInitializerKind::Union { value, .. } => initializer(root, value, output)?,
    }
    Ok(())
}

fn address<'a>(
    root: &'a CInitializer,
    value: &'a CValue,
    output: &mut Vec<Location<'a>>,
) -> Result<(), E> {
    match value.kind() {
        CValueKind::Literal(CLiteral::NullPointer(_)) | CValueKind::FunctionAddress(_) => Ok(()),
        CValueKind::AddressOf(value) => place(root, value, output),
        CValueKind::Convert {
            conversion: CConversion::AddConst(_) | CConversion::ObjectToVoid(_),
            operand,
        } => address(root, operand, output),
        _ => Err(E::ExpectedNumericConstant),
    }
}

fn place<'a>(
    root: &'a CInitializer,
    value: &'a CPlace,
    output: &mut Vec<Location<'a>>,
) -> Result<(), E> {
    match value.kind() {
        CPlaceKind::Global(_) => {}
        CPlaceKind::Member { base, .. } => place(root, base, output)?,
        CPlaceKind::Index {
            base: CIndexBase::Array(base),
            ..
        } => {
            place(root, base, output)?;
            output.push(Location {
                initializer: root,
                place: value,
            });
        }
        _ => return Err(E::ExpectedNumericConstant),
    }
    Ok(())
}
