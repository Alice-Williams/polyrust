use super::{CValueBinding, Writer};
use crate::ast::{CBlock, CStatementKind};
use std::fmt::Write;

impl Writer<'_> {
    pub(super) fn block(&self, block: &CBlock, indent: usize, text: &mut String) {
        text.push_str("{\n");
        for statement in block.statements() {
            text.push_str(&"    ".repeat(indent + 1));
            match statement.kind() {
                CStatementKind::Block(block) => self.block(block, indent + 1, text),
                CStatementKind::Declare(local) => {
                    let name =
                        self.names.values[&CValueBinding::Local(local.local().clone())].as_str();
                    writeln!(
                        text,
                        "{} = {};",
                        self.declarator(local.local().ty(), name),
                        self.initializer(local.initializer().expect("checked initialized local"))
                    )
                    .unwrap();
                }
                CStatementKind::Assign { place, value } => {
                    writeln!(text, "{} = {};", self.place(place), self.value(value)).unwrap();
                }
                CStatementKind::Discard(value) => {
                    writeln!(text, "(void) {};", self.value(value)).unwrap();
                }
                CStatementKind::Return(Some(value)) => {
                    writeln!(text, "return {};", self.value(value)).unwrap();
                }
                CStatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    write!(text, "if ({}) ", self.value(condition)).unwrap();
                    self.block(then_block, indent + 1, text);
                    write!(text, "{}else ", "    ".repeat(indent + 1)).unwrap();
                    self.block(else_block, indent + 1, text);
                }
                _ => unreachable!("checked statement profile"),
            }
        }
        writeln!(text, "{}}}", "    ".repeat(indent)).unwrap();
    }
}
