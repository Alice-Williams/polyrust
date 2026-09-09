//! Compose context, all-syntax layout checks and evaluated static constants.
use super::{CSafetyError as E, constants::evaluate, layout::Layouts};
use crate::ast::{
    CAggregateRef, CConversion, CDefinitionKind, CFileItem, CIndexBase, CInitializer,
    CInitializerKind, CLiteral, CObjectType, CObjectTypeKind, CPlace, CPlaceKind, CRegistry,
    CReturnType, CSourceFile, CValue, CValueKind,
    contextual::{
        access_statements,
        access_walk::{Access, Visitor},
    },
    registry::CRegistered,
};

impl CRegistry {
    /// Diagnostic constant/layout checks after complete contextual checking.
    /// Runtime ranges, pointer/ownership/call safety, resources and linking
    /// remain mandatory. This method returns no facts or rendering certificate.
    pub fn check_constants_and_layout(&self, files: &[CSourceFile]) -> Result<(), E> {
        self.check_context(files)?;
        PackageConstants {
            registry: self,
            files,
            layouts: Layouts::new(self),
        }
        .check()
    }
}

struct PackageConstants<'a> {
    registry: &'a CRegistry,
    files: &'a [CSourceFile],
    layouts: Layouts<'a>,
}

impl PackageConstants<'_> {
    fn check(mut self) -> Result<(), E> {
        self.inventory()?;
        for file in self.files {
            // This walk never skips an unselected SizeOf/type-form operand.
            access_statements::file(&mut self, file)?;
        }
        for file in self.files {
            for item in file.items() {
                match item {
                    CFileItem::StaticAssert(value) => {
                        if evaluate(&mut self.layouts, value.condition())?
                            .integer()?
                            .value()
                            == 0
                        {
                            return Err(E::FalseAssertion);
                        }
                    }
                    CFileItem::Definition(value) => {
                        if let CDefinitionKind::Object { initializer, .. } = value.kind() {
                            self.initializer(initializer)?;
                        }
                    }
                    CFileItem::Declaration(_) | CFileItem::Comment(_) => {}
                }
            }
        }
        Ok(())
    }

    fn object(&mut self, ty: &CObjectType) -> Result<(), E> {
        self.layouts.form(ty)?;
        self.layouts.object(ty)?;
        Ok(())
    }

    fn inventory(&mut self) -> Result<(), E> {
        for item in self.registry.contextual_inventory() {
            match item {
                CRegistered::Struct(value) => {
                    if self
                        .registry
                        .members(&CAggregateRef::Struct(value.clone()))?
                        .is_some()
                    {
                        self.object(&CObjectType::structure(value.clone()))?;
                    }
                }
                CRegistered::Union(value) => {
                    if self
                        .registry
                        .members(&CAggregateRef::Union(value.clone()))?
                        .is_some()
                    {
                        self.object(&CObjectType::union(value.clone()))?;
                    }
                }
                CRegistered::Enum(value) => {
                    self.object(&CObjectType::enumeration(value.clone()))?
                }
                CRegistered::Typedef(value) => self.layouts.form(value.target())?,
                CRegistered::Member(value) => self.object(value.ty())?,
                CRegistered::Object(value) => self.object(value.ty())?,
                CRegistered::Parameter(value) => self.object(value.ty())?,
                CRegistered::Local(value) => self.object(value.ty())?,
                CRegistered::OwnerSlot(value) => self.object(value.local().ty())?,
                CRegistered::Allocation(value) => self.object(value.object_type())?,
                CRegistered::Function(value) => {
                    if let CReturnType::Value(result) = value.signature().return_type() {
                        self.object(result.declared_type())?;
                    }
                    for parameter in value.signature().parameters() {
                        self.object(parameter.declared_type())?;
                    }
                }
                CRegistered::Enumerator(_)
                | CRegistered::Scope(_)
                | CRegistered::Loop(_)
                | CRegistered::Switch(_)
                | CRegistered::CleanupExit(_)
                | CRegistered::Witness(_)
                | CRegistered::Table(_)
                | CRegistered::Adapter(_) => {}
            }
        }
        Ok(())
    }

    fn initializer(&mut self, value: &CInitializer) -> Result<(), E> {
        match value.kind() {
            CInitializerKind::Zero(_) => {}
            CInitializerKind::Expression(value) => {
                if matches!(value.ty().kind(), CObjectTypeKind::Pointer(_)) {
                    self.address(value)?;
                } else {
                    evaluate(&mut self.layouts, value)?;
                }
            }
            CInitializerKind::Array { elements, .. } => {
                for value in elements {
                    self.initializer(value)?;
                }
            }
            CInitializerKind::Struct { members, .. } => {
                for (_, value) in members {
                    self.initializer(value)?;
                }
            }
            CInitializerKind::Union { value, .. } => self.initializer(value)?,
        }
        Ok(())
    }

    fn address(&mut self, value: &CValue) -> Result<(), E> {
        match value.kind() {
            CValueKind::Literal(CLiteral::NullPointer(_)) | CValueKind::FunctionAddress(_) => {
                Ok(())
            }
            CValueKind::AddressOf(place) => self.address_place(place),
            CValueKind::Convert {
                conversion: CConversion::AddConst(_) | CConversion::ObjectToVoid(_),
                operand,
            } => self.address(operand),
            _ => Err(E::ExpectedNumericConstant),
        }
    }

    fn address_place(&mut self, place: &CPlace) -> Result<(), E> {
        match place.kind() {
            CPlaceKind::Global(_) => Ok(()),
            CPlaceKind::Member { base, .. } => self.address_place(base),
            CPlaceKind::Index {
                base: CIndexBase::Array(base),
                index,
            } => {
                self.address_place(base)?;
                evaluate(&mut self.layouts, index)?.integer()?;
                Ok(())
            }
            _ => Err(E::ExpectedNumericConstant),
        }
    }
}

impl Visitor for PackageConstants<'_> {
    type Error = E;
    fn place(&mut self, place: &CPlace, access: Access) -> Result<(), E> {
        self.layouts.form(place.ty())?;
        if access != Access::Address || matches!(place.kind(), CPlaceKind::Index { .. }) {
            self.layouts.object(place.ty())?;
        }
        Ok(())
    }
    fn value(&mut self, value: &CValue) -> Result<(), E> {
        self.layouts.form(value.ty())?;
        if let CValueKind::SizeOf(ty) | CValueKind::AlignOf(ty) = value.kind() {
            self.object(ty)?;
        }
        Ok(())
    }
}
