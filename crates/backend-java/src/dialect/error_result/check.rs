//! Exact semantic-role inventory; target enum ordinals are never a wire format.
use super::JavaErrorKindValues;
use crate::ast::{
    JavaDeclarationKind, JavaHeritage, JavaMember, JavaSourceDeclaration, JavaSourceInventory,
    JavaType, JavaTypeDeclaration, JavaVisibility,
};
use portable_codegen::{
    GeneratedOrigin, GeneratedSymbolId, RustIntegerErrorKind, SynthesisReason, TargetTypeRef,
};
use std::collections::BTreeSet;

pub(in crate::dialect) fn error_variant(
    error: &JavaTypeDeclaration,
    interface: &JavaType,
    kinds: JavaErrorKindValues,
    inventory: &JavaSourceInventory,
) -> Result<(), String> {
    if error.kind != JavaDeclarationKind::Enum
        || error.visibility != JavaVisibility::Public
        || !error.modifiers.is_empty()
        || !error.type_parameters.is_empty()
        || !error.record_components.is_empty()
        || !error.permits.is_empty()
        || error.heritage != JavaHeritage::Interfaces(vec![interface.clone()])
        || error.members.len() != 6
    {
        return Err("Java error-kind variant requires exactly six immutable enum constants".into());
    }
    let owner = error
        .declared
        .ok_or("Java error-kind enum lacks its original identity")?;
    let mut seen = BTreeSet::new();
    for kind in RustIntegerErrorKind::ALL {
        let id = kinds.value(kind);
        if !seen.insert(id) {
            return Err("Java error-kind roles must select distinct original constants".into());
        }
        let Some(JavaMember::EnumConstant(constant)) = error.members.iter().find(
            |member| matches!(member, JavaMember::EnumConstant(value) if value.declared == id),
        ) else {
            return Err("Java error-kind role does not belong to the selected enum".into());
        };
        let spelling = match kind {
            RustIntegerErrorKind::Empty => "EMPTY",
            RustIntegerErrorKind::InvalidDigit => "INVALID_DIGIT",
            RustIntegerErrorKind::PosOverflow => "POS_OVERFLOW",
            RustIntegerErrorKind::NegOverflow => "NEG_OVERFLOW",
            RustIntegerErrorKind::Zero => "ZERO",
            RustIntegerErrorKind::NotAPowerOfTwo => "NOT_A_POWER_OF_TWO",
        };
        let Some(JavaSourceDeclaration::Value(registered)) =
            inventory.get(GeneratedSymbolId::Value(id))
        else {
            return Err("Java error-kind constant lacks its original adapter registration".into());
        };
        if constant.name.as_str() != spelling
            || registered.name != spelling
            || registered.ty != TargetTypeRef::Generated(owner)
            || registered.visibility != JavaVisibility::Public
            || registered.origin != GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter)
        {
            return Err(
                "Java error-kind role disagrees with its original constant registration".into(),
            );
        }
    }
    Ok(())
}
