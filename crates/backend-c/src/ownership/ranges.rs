//! Private abstract numeric transfer; graph dominance is a separate obligation.
mod conversion;
mod domain;
mod floating;
mod integer;
mod integer_operations;
mod model;
mod operators;
mod shifts;

pub(super) use domain::{DomainTransfer, NumericDomain, NumericLoss};
pub(super) use model::{CScalarRange, CTransfer};

#[cfg(test)]
#[path = "../tests/range_floating.rs"]
mod floating_tests;
#[cfg(test)]
#[path = "../tests/range_integer_properties.rs"]
mod integer_tests;
#[cfg(test)]
#[path = "../tests/range_singleton_matrix.rs"]
mod singleton_tests;
#[cfg(test)]
#[path = "../tests/range_wrapping.rs"]
mod wrapping_tests;

#[cfg(test)]
#[path = "../tests/flow_domains.rs"]
mod flow_domain_tests;
