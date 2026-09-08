//! Independent mapping-owned output checks.
use super::*;
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaTypeAliasInput, JavaErasedTypeAlias);
fn representation(_: &JavaTypeAliasInput) -> R {
    R::Erased
}
fn verify(input: &JavaTypeAliasInput, output: &JavaErasedTypeAlias) -> bool {
    output._name == identifier(&input.name) && output._target == input.target
}
