use super::{Result, RustDocumentationError};

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub declarations: usize,
    pub ancestry_nodes: usize,
    pub ancestry_depth: usize,
    pub export_modules: usize,
    pub export_bindings: usize,
    pub attributes: usize,
    pub text_bytes: usize,
}

impl Limits {
    pub const PRODUCTION: Self = Self {
        declarations: 100_000,
        ancestry_nodes: 100_000,
        ancestry_depth: 128,
        export_modules: 100_000,
        export_bindings: 100_000,
        attributes: 100_000,
        text_bytes: 16 * 1024 * 1024,
    };
}

pub(super) struct Budget {
    limits: Limits,
    declarations: usize,
    ancestry_nodes: usize,
    export_modules: usize,
    export_bindings: usize,
    attributes: usize,
    text_bytes: usize,
}

impl Budget {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            declarations: 0,
            ancestry_nodes: 0,
            export_modules: 0,
            export_bindings: 0,
            attributes: 0,
            text_bytes: 0,
        }
    }

    pub fn declaration(&mut self) -> Result<()> {
        charge(
            &mut self.declarations,
            1,
            self.limits.declarations,
            "declarations",
        )
    }

    pub fn ancestry(&mut self, count: usize) -> Result<()> {
        if count > self.limits.ancestry_depth {
            return Err(RustDocumentationError::Budget("ancestry depth"));
        }
        charge(
            &mut self.ancestry_nodes,
            count,
            self.limits.ancestry_nodes,
            "ancestry nodes",
        )
    }

    pub fn modules(&mut self, count: usize) -> Result<()> {
        charge(
            &mut self.export_modules,
            count,
            self.limits.export_modules,
            "export modules",
        )
    }

    pub fn binding(&mut self, name: &str) -> Result<()> {
        charge(
            &mut self.export_bindings,
            1,
            self.limits.export_bindings,
            "export bindings",
        )?;
        self.text(name.len())
    }

    pub fn text(&mut self, count: usize) -> Result<()> {
        charge(
            &mut self.text_bytes,
            count,
            self.limits.text_bytes,
            "metadata text",
        )
    }

    pub fn attributes(&mut self, attributes: &[String]) -> Result<()> {
        charge(
            &mut self.attributes,
            attributes.len(),
            self.limits.attributes,
            "attributes",
        )?;
        for attribute in attributes {
            self.text(attribute.len())?;
        }
        Ok(())
    }
}

fn charge(total: &mut usize, amount: usize, maximum: usize, name: &'static str) -> Result<()> {
    *total = total
        .checked_add(amount)
        .ok_or(RustDocumentationError::Budget(name))?;
    if *total > maximum {
        return Err(RustDocumentationError::Budget(name));
    }
    Ok(())
}
