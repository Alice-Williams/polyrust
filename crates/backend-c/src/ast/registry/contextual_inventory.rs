//! Typed views derived directly from every authoritative registry collection.

use super::{
    CAllocationRef, CCleanupExitRef, CEnumRef, CEnumeratorRef, CFileRef, CFunctionRef,
    CInterfaceAdapterRef, CInterfaceTableRef, CInterfaceWitnessRef, CLocalRef, CLoopRef,
    CMemberOwnershipRef, CMemberRef, CObjectRef, COwnerSlotRef, CParameterRef, CRegistry,
    CScopeRef, CStructRef, CSwitchRef, CTypedefRef, CUnionRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CRegistered<'a> {
    Struct(&'a CStructRef),
    Union(&'a CUnionRef),
    Enum(&'a CEnumRef),
    Typedef(&'a CTypedefRef),
    Member(&'a CMemberRef),
    Enumerator(&'a CEnumeratorRef),
    Function(&'a CFunctionRef),
    Object(&'a CObjectRef),
    Parameter(&'a CParameterRef),
    Scope(&'a CScopeRef),
    Local(&'a CLocalRef),
    OwnerSlot(&'a COwnerSlotRef),
    MemberOwnership(&'a CMemberRef, &'a CMemberOwnershipRef),
    Loop(&'a CLoopRef),
    Switch(&'a CSwitchRef),
    CleanupExit(&'a CCleanupExitRef),
    Allocation(&'a CAllocationRef),
    Witness(&'a CInterfaceWitnessRef),
    Table(&'a CInterfaceTableRef),
    Adapter(&'a CInterfaceAdapterRef),
}

impl CRegistry {
    pub(in crate::ast) fn registered_files(&self) -> impl Iterator<Item = &CFileRef> {
        self.files.iter()
    }

    pub(crate) fn contextual_inventory(&self) -> Vec<CRegistered<'_>> {
        // No mutable projection: deleting an AST item cannot delete its
        // registration obligation. Each item still carries its actual owner.
        let mut values = Vec::new();
        values.extend(self.structs.keys().map(CRegistered::Struct));
        values.extend(self.unions.keys().map(CRegistered::Union));
        values.extend(self.enums.keys().map(CRegistered::Enum));
        values.extend(self.typedefs.iter().map(CRegistered::Typedef));
        values.extend(self.members.iter().map(CRegistered::Member));
        values.extend(self.enumerators.iter().map(CRegistered::Enumerator));
        values.extend(self.functions.iter().map(CRegistered::Function));
        values.extend(self.objects.iter().map(CRegistered::Object));
        values.extend(self.parameters.iter().map(CRegistered::Parameter));
        values.extend(self.scopes.iter().map(CRegistered::Scope));
        values.extend(self.locals.iter().map(CRegistered::Local));
        values.extend(self.owner_slots.iter().map(CRegistered::OwnerSlot));
        values.extend(
            self.member_ownership
                .iter()
                .map(|(member, role)| CRegistered::MemberOwnership(member, role)),
        );
        values.extend(self.loops.iter().map(CRegistered::Loop));
        values.extend(self.switches.iter().map(CRegistered::Switch));
        values.extend(self.cleanup_exits.iter().map(CRegistered::CleanupExit));
        values.extend(self.allocations.iter().map(CRegistered::Allocation));
        values.extend(self.witnesses.iter().map(CRegistered::Witness));
        values.extend(self.tables.iter().map(CRegistered::Table));
        values.extend(self.adapters.iter().map(CRegistered::Adapter));
        values
    }
}
