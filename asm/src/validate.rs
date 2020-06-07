use std::collections::HashSet;

use crate::ast;
use crate::asm;
use crate::diagnostics::Diagnostics;
use crate::const_table::ConstTable;

/// Validates the program to ensure that it is valid assembly
///
/// Constant names will be subsituted for their immediate values. The only remaining identifiers
/// in the body of a statement will be a label name. The remaining label names will still need to
/// be checked later to make sure that they are defined somewhere in the program.
pub fn validate_program(prog: ast::Program, diag: &Diagnostics) -> asm::Program {
    let labels = unique_labels(&prog, &diag);
    // Error recovery: No checking if the unique labels generated errors because we can still
    // continue processing the program even if errors occurred during that process.
    let consts = ConstTable::new(&prog, diag, &labels);
    // Error recovery: No checking if the constant table generated errors because we still want to
    // continue and potentially find more errors if we can during the validation process. This may
    // result in some false negatives, but is still a better user experience overall in many cases.

    //TODO: Sort the statements into `code_section` and `static_section` variables
    let mut stmts = Vec::new();
    for stmt in prog.stmts {
        let stmt = consts.subst(stmt);
        stmts.extend(validate_stmt(stmt, diag));
        // Error recovery: No checking if validation failed because we want to produce as many
        // errors as possible by going through all the statements we can.
    }

    todo!()
}

/// Attempts to ensure that all label names are unique
///
/// Returns the set of all label names in the program, including, in the case of an error, label
/// names that may have been defined more than once.
fn unique_labels(prog: &ast::Program, diag: &Diagnostics) -> HashSet<ast::Ident> {
    let mut labels: HashSet<ast::Ident> = HashSet::new();

    for stmt in &prog.stmts {
        let label = match stmt {
            ast::Stmt::Label(label) => label,
            _ => continue,
        };

        match labels.get(label) {
            Some(other_label) => {
                diag.span_error(label.span, format!("duplicate label name `{}`", label))
                    .span_note(other_label.span, "originally defined here")
                    .emit();
                continue;
            },
            None => {
                debug_assert!(!labels.insert(label.clone()), "bug: label should not be present");
            },
        }
    }

    labels
}

/// Validates a statement to ensure that it is valid assembly language
fn validate_stmt(stmt: ast::Stmt, diag: &Diagnostics) -> Option<asm::Stmt> {
    todo!()
}
