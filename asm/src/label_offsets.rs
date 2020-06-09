use std::collections::HashMap;

use crate::asm;
use crate::diagnostics::Diagnostics;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOffsets {
    offsets: HashMap<asm::Ident, i128>,
}

impl LabelOffsets {
    pub fn new(prog: &asm::Program) -> Self {
        let mut offsets = HashMap::new();
        let mut current_offset = 0;

        for stmt in prog.iter_all_stmts() {
            for label in &stmt.labels {
                offsets.insert(label.clone(), current_offset);
            }

            current_offset += stmt.size_bytes() as i128;
        }

        Self {offsets}
    }

    /// Looks up a label name and returns the immediate value of its offset
    pub fn lookup(&self, name: &asm::Ident, diag: &Diagnostics) -> asm::Immediate {
        //TODO: Use the span of `name` in the returned immediate
        todo!()
    }
}
