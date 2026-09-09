//! Allocation requests retain authenticated call sites and original byte facts.
#[cfg(test)]
#[path = "../../../tests/allocation_request_sites.rs"]
mod tests;
#[cfg(test)]
use super::NumericFacts;
use crate::ast::{
    CCall, CCallableKind, CFunctionRef, CKnownObject, CObjectType, CScopeRef,
    contextual::flow_graph::Point,
};
use crate::dialect::CKnownCall;
use crate::ownership::{CSafetyError as E, layout::Layouts, numeric_flow::Obligation};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) struct AllocationOrigin {
    function: CFunctionRef,
    point: Point,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ownership) struct AllocationRequest {
    origin: AllocationOrigin,
    scope: CScopeRef,
    bytes: (u64, u64),
    alignment: u64,
}
impl AllocationRequest {
    pub(in crate::ownership) fn join(&self, other: &Self) -> Option<Self> {
        if self.origin != other.origin
            || self.scope != other.scope
            || self.alignment != other.alignment
        {
            return None;
        }
        let mut result = self.clone();
        result.bytes = (
            self.bytes.0.min(other.bytes.0),
            self.bytes.1.max(other.bytes.1),
        );
        result.validate().ok()?;
        Some(result)
    }
    pub(in crate::ownership::numeric_flow) fn actual(
        context: &crate::ownership::context_facts::ContextFacts<'_>,
        graph: &crate::ast::contextual::flow_graph::Graph<'_>,
        point: Point,
        call: &CCall,
        bytes: &crate::ownership::numeric_flow::Number<'_>,
    ) -> Result<Self, E> {
        if !context
            .functions()
            .iter()
            .any(|actual| std::ptr::eq(actual, graph))
            || point.index() >= graph.nodes().len()
        {
            return Err(E::InvalidNumericSite);
        }
        if call.callable().kind() != &CCallableKind::Known(CKnownCall::Allocate) {
            return Err(E::UnprovedAllocation);
        }
        crate::ownership::numeric_flow::sites::check(
            graph.node(point).action(),
            &Obligation::Call {
                call,
                arguments: vec![],
                byte_product: None,
            },
        )?;
        let bounds = bytes.extent_bounds().map_err(|error| match error {
            E::IndexOutOfBounds => E::UnprovedAllocationSize,
            error => error,
        })?;
        let request = Self {
            origin: AllocationOrigin {
                function: graph.function().clone(),
                point,
            },
            scope: graph.node(point).scope().clone(),
            bytes: bounds,
            alignment: Layouts::new(context.registry())
                .object(&CObjectType::known(CKnownObject::MaxAlign))?
                .alignment(),
        };
        request.validate()?;
        Ok(request)
    }
    pub(in crate::ownership) fn admits_object(
        &self,
        allocation: &crate::ast::CAllocationRef,
        registry: &crate::ast::CRegistry,
    ) -> Result<(), E> {
        self.validate()?;
        if allocation.scope().function() != &self.origin.function
            || allocation.allocator() != &crate::ast::CAllocatorSource::Default
        {
            return Err(E::UnprovedAllocation);
        }
        let layout = Layouts::new(registry).object(allocation.object_type())?;
        if layout.size() > self.bytes.0 || layout.alignment() > self.alignment {
            return Err(E::UnprovedAllocationSize);
        }
        Ok(())
    }
    pub(in crate::ownership) fn origin(&self) -> &AllocationOrigin {
        &self.origin
    }
    pub(in crate::ownership) fn validate(&self) -> Result<(), E> {
        if self.scope.function() != &self.origin.function {
            return Err(E::InvalidNumericSite);
        }
        if self.bytes.0 == 0 || self.bytes.0 > self.bytes.1 || !self.alignment.is_power_of_two() {
            return Err(E::UnprovedAllocationSize);
        }
        Ok(())
    }
}
#[cfg(test)]
impl<'ast> NumericFacts<'ast> {
    pub(in crate::ownership) fn allocation_request(
        &self,
        actual: &CCall,
    ) -> Result<AllocationRequest, E> {
        if actual.callable().kind() != &CCallableKind::Known(CKnownCall::Allocate) {
            return Err(E::UnprovedAllocation);
        }
        let mut result: Option<AllocationRequest> = None;
        for entry in &self.analysis.obligations {
            let Obligation::Call {
                call, arguments, ..
            } = &entry.kind
            else {
                continue;
            };
            if !std::ptr::eq(*call, actual) {
                continue;
            }
            let bytes = arguments
                .first()
                .and_then(Option::as_ref)
                .ok_or(E::ExpectedNumericValue)?;
            if !bytes.losses.is_empty() {
                return Err(E::UnprovedSizeArithmetic);
            }
            let (first, last) = bytes
                .domain
                .integer_bounds()
                .ok_or(E::ExpectedNumericValue)?;
            let bounds = (
                u64::try_from(first).map_err(|_| E::UnprovedAllocationSize)?,
                u64::try_from(last).map_err(|_| E::UnprovedAllocationSize)?,
            );
            let graph = self
                .context
                .functions()
                .iter()
                .find(|graph| graph.function() == entry.site.function)
                .ok_or(E::InvalidNumericSite)?;
            let request = AllocationRequest {
                origin: AllocationOrigin {
                    function: entry.site.function.clone(),
                    point: entry.site.point,
                },
                scope: graph.node(entry.site.point).scope().clone(),
                bytes: bounds,
                alignment: Layouts::new(self.context.registry())
                    .object(&CObjectType::known(CKnownObject::MaxAlign))?
                    .alignment(),
            };
            request.validate()?;
            if let Some(old) = &mut result {
                if old.origin != request.origin || old.scope != request.scope {
                    return Err(E::InvalidNumericSite);
                }
                old.bytes = (old.bytes.0.min(bounds.0), old.bytes.1.max(bounds.1));
            } else {
                result = Some(request);
            }
        }
        result.ok_or(E::InvalidNumericSite)
    }
}
