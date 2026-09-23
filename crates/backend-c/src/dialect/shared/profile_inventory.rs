//! Source order and syntactic use obligations for the strict native contract.
use super::Node;
use crate::ast::{
    CAggregateRef, CCallableKind, CDeclarationKind, CDefinitionKind, CFileItem, CFunctionRef,
    CLinkage, CLocalRef, CObjectTypeKind, CParameterRef, CPlaceKind, CStatementKind, CStructRef,
    CValueKind,
};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Inventory {
    imports: BTreeSet<CFunctionRef>,
    records: BTreeSet<CStructRef>,
    prototypes: BTreeSet<CFunctionRef>,
    parameters: BTreeSet<CParameterRef>,
    locals: BTreeSet<CLocalRef>,
    used_parameters: BTreeSet<CParameterRef>,
    used_locals: BTreeSet<CLocalRef>,
    internal_functions: BTreeSet<CFunctionRef>,
    used_functions: BTreeSet<CFunctionRef>,
}

impl Inventory {
    pub(super) fn with_registry(registry: Option<&crate::ast::CRegistry>) -> Result<Self, String> {
        let mut result = Self::default();
        if let Some(registry) = registry {
            for (function, _) in registry.imported_functions() {
                registry
                    .imported_function(function)
                    .map_err(|error| error.to_string())?;
                result.imports.insert(function.clone());
            }
        }
        Ok(result)
    }
    pub(super) fn visit(&mut self, node: Node<'_>) -> Result<(), String> {
        match node {
            Node::Item(CFileItem::Declaration(declaration)) => match declaration.kind() {
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(record),
                    ..
                } => {
                    self.records.insert(record.clone());
                }
                CDeclarationKind::FunctionPrototype { function, .. } => {
                    self.prototype(function)?;
                }
                _ => {}
            },
            Node::Item(CFileItem::Definition(definition)) => {
                if let CDefinitionKind::Function {
                    function,
                    parameters,
                    linkage,
                    ..
                } = definition.kind()
                {
                    if !self.prototypes.contains(function) {
                        return Err(
                            "strict C profile requires a prototype before each definition".into(),
                        );
                    }
                    self.parameters.extend(parameters.iter().cloned());
                    if *linkage == CLinkage::Internal {
                        self.internal_functions.insert(function.clone());
                    }
                }
            }
            Node::Statement(statement) => {
                if let CStatementKind::Declare(local) = statement.kind() {
                    self.locals.insert(local.local().clone());
                }
            }
            Node::Place(place) => match place.kind() {
                CPlaceKind::Local(local) => {
                    self.used_locals.insert(local.clone());
                }
                CPlaceKind::Parameter(parameter) => {
                    self.used_parameters.insert(parameter.clone());
                }
                _ => {}
            },
            Node::Value(value) => {
                if let CValueKind::Call(call) = value.kind() {
                    self.call(call)?;
                }
            }
            Node::Effect(effect) => self.call(effect.call())?,
            Node::Type(ty) => {
                if let CObjectTypeKind::Struct(record) = ty.kind()
                    && !self.records.contains(record)
                {
                    return Err("strict C profile requires record declarations before use".into());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn call(&mut self, call: &crate::ast::CCall) -> Result<(), String> {
        if let CCallableKind::Direct(function) = call.callable().kind() {
            if !self.prototypes.contains(function.as_ref())
                && !self.imports.contains(function.as_ref())
            {
                return Err("strict C profile requires a prototype before each call".into());
            }
            self.used_functions.insert(function.as_ref().clone());
        }
        Ok(())
    }

    fn prototype(&mut self, function: &CFunctionRef) -> Result<(), String> {
        let result = match function.signature().return_type() {
            crate::ast::CReturnType::Void => None,
            crate::ast::CReturnType::Value(value) => Some(value.declared_type()),
        };
        for ty in result.into_iter().chain(
            function
                .signature()
                .parameters()
                .iter()
                .map(|parameter| parameter.declared_type()),
        ) {
            if let CObjectTypeKind::Struct(record) = ty.kind()
                && !self.records.contains(record)
            {
                return Err(
                    "strict C profile requires record declarations before signatures".into(),
                );
            }
        }
        if self.imports.contains(function) {
            return Err("C dependency prototypes belong to their certified public header".into());
        }
        if self.prototypes.insert(function.clone()) {
            Ok(())
        } else {
            Err("strict C profile requires exactly one primary prototype".into())
        }
    }

    pub(super) fn finish(self) -> Result<(), String> {
        if !self.internal_functions.is_subset(&self.used_functions) {
            return Err("strict C profile requires syntactic use of internal functions".into());
        }
        if !self.parameters.is_subset(&self.used_parameters)
            || !self.locals.is_subset(&self.used_locals)
        {
            return Err(
                "strict C profile requires explicit use or Discard for unused bindings".into(),
            );
        }
        Ok(())
    }
}
