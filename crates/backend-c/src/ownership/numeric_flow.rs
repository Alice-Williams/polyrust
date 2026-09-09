//! Converged numeric diagnostics and borrowed operation obligations, not rendering.
mod aggregate_copy;
mod calls;
mod evaluate;
mod facts;
mod provenance;
mod refine;
mod relations;
mod sites;
mod solve;
mod state;
mod statements;
mod storage;

use super::{
    CSafetyError as E,
    context_facts::ContextFacts,
    layout::Layouts,
    ranges::{DomainTransfer, NumericDomain, NumericLoss},
};
use crate::ast::{
    CCall, CFunctionRef, CPlace, CRegistry, CSourceFile, CValue, contextual::flow_graph::Point,
};
pub(super) use facts::NumericFacts;
use state::{Number, State};
use std::collections::BTreeSet;
use storage::Root;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode<'a> {
    Solve,
    Derive,
    Verify(Site<'a>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Site<'a> {
    function: &'a CFunctionRef,
    point: Point,
}
struct LocatedObligation<'a> {
    site: Site<'a>,
    kind: Obligation<'a>,
}
struct FunctionFacts<'a> {
    function: &'a CFunctionRef,
    incoming: Vec<Option<State<'a>>>,
}
struct Analysis<'a> {
    functions: Vec<FunctionFacts<'a>>,
    obligations: Vec<LocatedObligation<'a>>,
}

enum Obligation<'a> {
    Index {
        place: &'a CPlace,
        index: Number<'a>,
    },
    Call {
        call: &'a CCall,
        arguments: Vec<Option<Number<'a>>>,
        byte_product: Option<DomainTransfer>,
    },
    Calculation {
        value: &'a CValue,
        number: Number<'a>,
    },
}

struct Engine<'a> {
    registry: &'a CRegistry,
    layouts: Layouts<'a>,
    addresses: BTreeSet<Root>,
    mode: Mode<'a>,
    obligations: Vec<LocatedObligation<'a>>,
}
impl<'a> Engine<'a> {
    fn numeric(&mut self, value: &'a CValue, state: &mut State<'a>) -> Result<Number<'a>, E> {
        self.expression(value, state)?
            .ok_or(E::ExpectedNumericValue)
    }
    fn calculation(
        &mut self,
        value: &'a CValue,
        result: Result<DomainTransfer, E>,
        children: &[&Number<'a>],
    ) -> Result<Number<'a>, E> {
        let mut number = match result {
            Ok(result) => {
                let mut number = Number::domain(result.domain);
                if result.loss == NumericLoss::MayWrap {
                    number.losses.push(provenance::Origin::Arithmetic(value));
                }
                number
            }
            Err(_) if self.mode == Mode::Solve => Number::domain(NumericDomain::full(
                storage::scalar(self.registry, value.ty())?.ok_or(E::ExpectedNumericValue)?,
            )?),
            Err(error) => return Err(error),
        };
        for child in children {
            number.inherit_losses(child);
        }
        self.record(|| Obligation::Calculation {
            value,
            number: number.clone(),
        });
        Ok(number)
    }
    fn record(&mut self, make: impl FnOnce() -> Obligation<'a>) {
        if let Mode::Verify(site) = self.mode {
            self.obligations
                .push(LocatedObligation { site, kind: make() });
        }
    }
}

impl CRegistry {
    /// Adds converged numeric checks; pointer/storage/call safety is still separate.
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::numeric_flow::Engine;
    /// fn forge<'a>() -> Engine<'a> { todo!() }
    /// ```
    ///
    /// ```compile_fail
    /// use portable_backend_c::ownership::numeric_flow::NumericFacts;
    /// fn forge<'a>() -> NumericFacts<'a> { todo!() }
    /// ```
    pub fn check_numeric_flow(&self, files: &[CSourceFile]) -> Result<(), E> {
        NumericFacts::check(self, files)?;
        Ok(())
    }
}
