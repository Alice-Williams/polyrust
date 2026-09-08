//! Independent mapping-owned output checks.
use super::{JavaErasedTypeAlias, JavaTypeAliasInput};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::identifier;
java_input_plan!(JavaTypeAliasInput, JavaErasedTypeAlias);
fn representation(_: &JavaTypeAliasInput) -> R {
    R::Erased
}
fn verify(input: &JavaTypeAliasInput, output: &JavaErasedTypeAlias) -> bool {
    output._name == identifier(&input.name) && output._target == input.target
}
