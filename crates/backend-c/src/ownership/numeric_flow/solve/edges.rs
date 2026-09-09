//! Actual edge meaning owns polarity and case selection, independent of Vec order.
use crate::ast::{
    CCaseConstant, CScalarRepresentation, CSignedLiteral as S, CUnsignedLiteral as U,
    contextual::flow_graph::{Edge, EdgeMeaning, Node, Polarity, Selection},
};
use crate::ownership::numeric_flow::{E, Engine, NumericDomain, State};

impl<'a> Engine<'a> {
    pub(in crate::ownership::numeric_flow) fn edge(
        &mut self,
        state: &State<'a>,
        node: &Node<'a>,
        edge: &Edge<'a>,
    ) -> Result<Option<State<'a>>, E> {
        match edge.meaning() {
            EdgeMeaning::Predicate {
                condition,
                polarity,
                ..
            } => self.refine(state, condition, polarity == Polarity::True),
            EdgeMeaning::Switch {
                value, selection, ..
            } => {
                let mut state = state.clone();
                let number = self.numeric(value, &mut state)?;
                let promoted = number
                    .domain
                    .ty()
                    .integer_promotion()
                    .ok_or(E::ExpectedNumericValue)?;
                let mut domain = match selection {
                    Selection::Cases(_) => NumericDomain::empty(number.domain.ty()),
                    Selection::Default => number.domain.clone(),
                };
                match selection {
                    Selection::Cases(cases) => {
                        for case in cases {
                            let value = converted_case(case, promoted.representation())?;
                            domain = domain.join(&number.domain.restrict_integer(value, value)?)?;
                        }
                    }
                    Selection::Default => {
                        for edge in node.successors() {
                            if let EdgeMeaning::Switch {
                                selection: Selection::Cases(cases),
                                ..
                            } = edge.meaning()
                            {
                                for case in cases {
                                    domain = domain.exclude_integer(converted_case(
                                        case,
                                        promoted.representation(),
                                    )?)?;
                                }
                            }
                        }
                    }
                }
                if self.restrict_value(&mut state, value, domain)? {
                    Ok(Some(state))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(Some(state.clone())),
        }
    }
}

fn converted_case(case: &CCaseConstant, representation: CScalarRepresentation) -> Result<i128, E> {
    let value = match case {
        CCaseConstant::Enumerator(value) => i128::from(value.value()),
        CCaseConstant::Signed(value) => match value {
            S::PlainChar(v) | S::I8(v) => i128::from(*v),
            S::I16(v) => i128::from(*v),
            S::Int(v) | S::I32(v) => i128::from(*v),
            S::I64(v) => i128::from(*v),
        },
        CCaseConstant::Unsigned(value) => match value {
            U::U8(v) => i128::from(*v),
            U::U16(v) => i128::from(*v),
            U::U32(v) => i128::from(*v),
            U::U64(v) | U::Size(v) => i128::from(*v),
        },
    };
    let (width, signed) = match representation {
        CScalarRepresentation::Signed(width) => (width, true),
        CScalarRepresentation::Unsigned(width) => (width, false),
        _ => return Err(E::ExpectedNumericValue),
    };
    let modulus = 1_i128 << width.bits();
    let residue = value.rem_euclid(modulus);
    Ok(if signed && residue >= modulus / 2 {
        residue - modulus
    } else {
        residue
    })
}
