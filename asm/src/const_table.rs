use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

use crate::ast;
use crate::diagnostics::Diagnostics;

#[derive(Debug, Clone)]
struct ConstEntry(ast::Const);

impl PartialEq for ConstEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.name.eq(&other.0.name)
    }
}

impl Eq for ConstEntry {}

impl Hash for ConstEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name.hash(state);
    }
}

impl Borrow<ast::Ident> for ConstEntry {
    fn borrow(&self) -> &ast::Ident {
        &self.0.name
    }
}

#[derive(Debug)]
pub struct ConstTable {
    const_values: HashSet<ConstEntry>,
}

impl ConstTable {
    pub fn new(prog: &ast::Program, diag: &Diagnostics, labels: &HashSet<ast::Ident>) -> Self {
        let mut const_values = HashSet::new();

        for stmt in &prog.stmts {
            let const_stmt = match stmt {
                ast::Stmt::Const(const_stmt) => const_stmt,
                _ => continue,
            };

            if let Some(label) = labels.get(&const_stmt.name) {
                diag.span_error(const_stmt.span, format!("constant name `{}` conflicts with a label name", const_stmt.name))
                    .span_note(label.span, "the conflicting label")
                    .emit();
            }

            if let Some(ConstEntry(prev_const)) = const_values.get(&const_stmt.name) {
                if prev_const.value != const_stmt.value {
                    diag.span_warning(const_stmt.span, format!("constant named `{}` was redefined", const_stmt.name))
                        .span_note(prev_const.span, "the previous declaration of this constant")
                        .emit();
                }
            }

            // Insert or overwrite the constant to update the span
            const_values.replace(ConstEntry(const_stmt.clone()));
        }

        Self {const_values}
    }

    /// Replaces all constant names with the immediate values that they map to
    pub fn subst_instr(&self, stmt: ast::Instr) -> ast::Instr {
        //TODO: Preserve the Span of the replaced value so error messages point to the right
        // part of the code
        todo!()
    }
}
