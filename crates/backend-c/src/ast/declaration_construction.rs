//! Declaration payloads come from the authoritative registry, not copied flags.

use super::{
    CAggregateRef, CBlock, CDeclaration, CDeclarationKind as D, CDefinition, CDefinitionKind as F,
    CEnumRef, CExpressions, CFileError as E, CFileRef, CFileRole, CFunctionRef, CInitializer,
    CLinkage, CObjectRef, CObjectType, CParameterRef, CRegistry, CTypedefRef,
};

pub struct CDeclarations<'a> {
    pub(super) registry: &'a CRegistry,
    pub(super) file: CFileRef,
}

impl<'a> CDeclarations<'a> {
    pub fn new(registry: &'a CRegistry, file: CFileRef) -> Result<Self, E> {
        registry.check_file(&file)?;
        Ok(Self { registry, file })
    }

    fn declaration(&self, kind: D) -> CDeclaration {
        CDeclaration {
            file: self.file.clone(),
            kind,
        }
    }
    fn definition(&self, kind: F) -> CDefinition {
        CDefinition {
            file: self.file.clone(),
            kind,
        }
    }

    pub(super) fn same_file(&self, file: &CFileRef) -> Result<(), E> {
        self.registry.check_file(file)?;
        if file == &self.file {
            Ok(())
        } else {
            Err(E::WrongFile)
        }
    }

    pub fn forward_tag(&self, owner: CAggregateRef) -> Result<CDeclaration, E> {
        self.registry.check_aggregate(&owner)?;
        Ok(self.declaration(D::ForwardTag(owner)))
    }

    pub fn typedef(&self, value: CTypedefRef) -> Result<CDeclaration, E> {
        self.registry
            .check_type(&CObjectType::typedef(value.clone()))?;
        self.same_file(value.file())?;
        Ok(self.declaration(D::Typedef(value)))
    }

    pub fn aggregate(&self, owner: CAggregateRef) -> Result<CDeclaration, E> {
        let members = self
            .registry
            .members(&owner)?
            .ok_or(E::IncompleteDefinition)?
            .to_vec();
        self.same_file(match &owner {
            CAggregateRef::Struct(value) => value.file(),
            CAggregateRef::Union(value) => value.file(),
        })?;
        Ok(self.declaration(D::Aggregate { owner, members }))
    }

    pub fn enumeration(&self, owner: CEnumRef) -> Result<CDeclaration, E> {
        let values = self
            .registry
            .enumerators(&owner)?
            .ok_or(E::IncompleteDefinition)?
            .to_vec();
        self.same_file(owner.file())?;
        Ok(self.declaration(D::Enum { owner, values }))
    }

    pub fn function_prototype(
        &self,
        function: CFunctionRef,
        linkage: CLinkage,
    ) -> Result<CDeclaration, E> {
        self.registry.check_function(&function)?;
        self.same_file(function.file())?;
        validate_linkage(linkage, function.file())?;
        Ok(self.declaration(D::FunctionPrototype { function, linkage }))
    }

    pub fn object_declaration(&self, object: CObjectRef) -> Result<CDeclaration, E> {
        self.registry.check_object(&object)?;
        self.same_file(object.file())?;
        Ok(self.declaration(D::ObjectDeclaration(object)))
    }

    pub fn function_definition(
        &self,
        function: CFunctionRef,
        linkage: CLinkage,
        parameters: Vec<CParameterRef>,
        body: CBlock,
    ) -> Result<CDefinition, E> {
        self.registry.check_function(&function)?;
        definition_file(function.file(), &self.file)?;
        validate_linkage(linkage, function.file())?;
        if body.scope().function() != &function || body.scope().parent().is_some() {
            return Err(E::InvalidFunctionRoot);
        }
        self.registry.check_lexical_scope(body.scope())?;
        if parameters.len() != function.signature().parameters().len() {
            return Err(E::ParameterInventory);
        }
        for (index, parameter) in parameters.iter().enumerate() {
            self.registry.check_parameter(&function, parameter)?;
            if parameter.index() != index {
                return Err(E::ParameterInventory);
            }
        }
        Ok(self.definition(F::Function {
            function,
            linkage,
            parameters,
            body: Box::new(body),
        }))
    }

    pub fn object_definition(
        &self,
        object: CObjectRef,
        linkage: CLinkage,
        initializer: CInitializer,
    ) -> Result<CDefinition, E> {
        self.registry.check_object(&object)?;
        definition_file(object.file(), &self.file)?;
        validate_linkage(linkage, object.file())?;
        CExpressions::new(self.registry).initializer_fits(object.ty(), &initializer)?;
        if !super::constant_expressions::is_static_initializer(&initializer) {
            return Err(E::ExpectedStaticInitializer);
        }
        Ok(self.definition(F::Object {
            object,
            linkage,
            initializer: Box::new(initializer),
        }))
    }
}

fn validate_linkage(linkage: CLinkage, origin: &CFileRef) -> Result<(), E> {
    match linkage {
        CLinkage::None => Err(E::InvalidLinkage),
        CLinkage::Internal
            if matches!(
                origin.key().role,
                CFileRole::GeneratedPublicHeader | CFileRole::RuntimePublicHeader
            ) =>
        {
            Err(E::InvalidLinkage)
        }
        CLinkage::External | CLinkage::Internal => Ok(()),
    }
}

fn definition_file(origin: &CFileRef, target: &CFileRef) -> Result<(), E> {
    let valid = match origin.key().role {
        CFileRole::GeneratedPublicHeader => target.key().role == CFileRole::GeneratedSource,
        CFileRole::RuntimePublicHeader => target.key().role == CFileRole::RuntimeSource,
        CFileRole::PrivateHeader => matches!(
            target.key().role,
            CFileRole::GeneratedSource | CFileRole::RuntimeSource
        ),
        CFileRole::GeneratedSource | CFileRole::RuntimeSource | CFileRole::TestSource => {
            origin == target
        }
    };
    if valid { Ok(()) } else { Err(E::WrongFileRole) }
}
