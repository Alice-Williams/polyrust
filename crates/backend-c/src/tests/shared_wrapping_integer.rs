//! Test-only original-owner packages shared by signed modular operations.
use super::{CDependencyApi, CDependencyFunction, CDialect, dependency_fixture as f};
use crate::ast::*;
use portable_codegen::*;

#[path = "shared_wrapping_integer_body.rs"]
pub(super) mod body;
pub(super) use body::{Operation, Variant};
#[path = "shared_wrapping_integer_fixture.rs"]
pub(super) mod fixture;

pub(super) fn api(source: &f::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(source)).unwrap(),
    )
    .unwrap()
}

pub(super) fn chain(operation: Operation, variant: Variant) -> Vec<CDependencyApi> {
    let producer = api(&fixture::build(
        701,
        &[],
        fixture::Body::Arithmetic(operation, variant),
    ));
    let imports: Vec<CDependencyFunction> = producer.functions().cloned().collect();
    let consumer = api(&fixture::build(702, &imports, fixture::Body::Forward));
    vec![producer, consumer]
}
