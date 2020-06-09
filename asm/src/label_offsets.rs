use std::collections::HashMap;

use crate::asm;
use crate::diagnostics::Diagnostics;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOffsets {
    offsets: HashMap<asm::Ident, u64>,
}

impl LabelOffsets {
    pub fn new(prog: &asm::Program) -> Self {
        let mut offsets = HashMap::new();
        let mut current_offset = 0;

        for stmt in prog.iter_all_stmts() {
            for label in &stmt.labels {
                offsets.insert(label.clone(), current_offset);
            }

            current_offset += stmt.size_bytes();
        }

        Self {offsets}
    }

    /// Looks up a label name and returns the immediate value of its offset
    pub fn lookup(&self, name: &asm::Ident, diag: &Diagnostics) -> asm::Immediate {
        let value = match self.offsets.get(name).copied() {
            Some(value) => value,
            None => {
                diag.span_error(name.span, format!("unknown label `{}`", name)).emit();

                // Error Recovery: default to zero if the label isn't found so we can keep checking
                // for more errors
                0
            },
        };

        asm::Integer {
            // Enforcing the invariant that this value must be <= u64::max() by converting from
            // a value of type u64
            value: value as i128,
            // Preserve the span of the replaced value so error messages point to the right
            // place
            span: name.span,
        }
    }
}
