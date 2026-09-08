//! Constructor coverage of exact array bounds and aggregate member inventories.

use super::{
    CAggregateRef, CExpressions, CInitializer, CInitializerError as E, CInitializerKind,
    CMemberRef, CObjectType, CObjectTypeKind, CRegistryError, CStructRef, CUnionRef, CValue,
};

impl CExpressions<'_> {
    fn initializer(&self, ty: CObjectType, kind: CInitializerKind) -> CInitializer {
        CInitializer {
            brand: self.registry.expression_brand(),
            ty: ty.canonical(),
            kind,
        }
    }

    pub(super) fn check_initializer(&self, value: &CInitializer) -> Result<(), E> {
        if value.brand == self.registry.expression_brand() {
            Ok(())
        } else {
            Err(CRegistryError::CrossRegistry.into())
        }
    }

    pub(super) fn initializer_fits(
        &self,
        destination: &CObjectType,
        value: &CInitializer,
    ) -> Result<(), E> {
        self.check_initializer(value)?;
        if self.registry.types_match(
            &destination.canonical().without_top_level_const(),
            &value.ty().clone().without_top_level_const(),
        )? {
            Ok(())
        } else {
            Err(E::TypeMismatch)
        }
    }

    pub fn expression_initializer(&self, value: CValue) -> Result<CInitializer, E> {
        self.check_value(&value)?;
        Ok(self.initializer(value.ty().clone(), CInitializerKind::Expression(value)))
    }

    pub fn zero_initializer(&self, ty: CObjectType) -> Result<CInitializer, E> {
        self.registry.check_type(&ty)?;
        ty.require_storable()?;
        Ok(self.initializer(ty.clone(), CInitializerKind::Zero(ty)))
    }

    pub fn array_initializer(
        &self,
        declared_type: CObjectType,
        elements: Vec<CInitializer>,
    ) -> Result<CInitializer, E> {
        self.registry.check_type(&declared_type)?;
        let ty = declared_type.canonical();
        let CObjectTypeKind::Array { element, length } = ty.kind() else {
            return Err(E::ExpectedArray);
        };
        if u64::try_from(elements.len()) != Ok(length.get()) {
            return Err(E::ElementCount {
                expected: length.get(),
                actual: elements.len(),
            });
        }
        for value in &elements {
            self.initializer_fits(element, value)?;
        }
        Ok(self.initializer(
            ty,
            CInitializerKind::Array {
                declared_type,
                elements,
            },
        ))
    }

    pub fn struct_initializer(
        &self,
        owner: CStructRef,
        members: Vec<(CMemberRef, CInitializer)>,
    ) -> Result<CInitializer, E> {
        let aggregate = CAggregateRef::Struct(owner.clone());
        let registered = self
            .registry
            .members(&aggregate)?
            .ok_or(E::IncompleteAggregate)?;
        for (member, value) in &members {
            self.registry.check_member(&aggregate, member)?;
            self.initializer_fits(member.ty(), value)?;
        }
        if !registered
            .iter()
            .eq(members.iter().map(|(member, _)| member))
        {
            return Err(E::MemberInventoryMismatch);
        }
        Ok(self.initializer(
            CObjectType::structure(owner.clone()),
            CInitializerKind::Struct { owner, members },
        ))
    }

    pub fn union_initializer(
        &self,
        owner: CUnionRef,
        member: CMemberRef,
        value: CInitializer,
    ) -> Result<CInitializer, E> {
        let aggregate = CAggregateRef::Union(owner.clone());
        self.registry
            .members(&aggregate)?
            .ok_or(E::IncompleteAggregate)?;
        self.registry.check_member(&aggregate, &member)?;
        self.initializer_fits(member.ty(), &value)?;
        Ok(self.initializer(
            CObjectType::union(owner.clone()),
            CInitializerKind::Union {
                owner,
                member,
                value: Box::new(value),
            },
        ))
    }
}
