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
    let mut code_section: Option<asm::Section> = None;
    let mut static_section: Option<asm::Section> = None;
    let mut stmts = None;
    let mut labels = Vec::new();
    for stmt in prog.stmts {
        let validated_stmt = match stmt {
            ast::Stmt::Label(label) => {
                labels.push(label);
                continue;
            },

            ast::Stmt::Section(section) => match section.kind {
                ast::SectionKind::Code => {
                    if static_section.is_some() {
                        diag.span_error(section.span, "the `.code` section must occur before the `.static` section").emit();
                    }

                    match &code_section {
                        Some(prev) => diag.span_error(section.span, "duplicate `.code` section")
                            .span_note(prev.section_header_span, "previously declared here").emit(),
                        None => code_section = Some(asm::Section {
                            section_header_span: section.span,
                            stmts: Vec::new(),
                        }),
                    }

                    stmts = Some(&mut code_section.as_mut().unwrap().stmts);
                    continue;
                },
                ast::SectionKind::Static => {
                    match &static_section {
                        Some(prev) => diag.span_error(section.span, "duplicate `.static` section")
                            .span_note(prev.section_header_span, "previously declared here").emit(),
                        None => static_section = Some(asm::Section {
                            section_header_span: section.span,
                            stmts: Vec::new(),
                        }),
                    }

                    stmts = Some(&mut static_section.as_mut().unwrap().stmts);
                    continue;
                },
            },

            ast::Stmt::Include(_) => unreachable!("bug: all includes should be resolved by now"),

            // Already handled above
            ast::Stmt::Const(_) => continue,

            ast::Stmt::StaticData(static_data) => {
                let static_data = consts.subst_static_data(static_data);
                validate_static_data(static_data, diag).map(asm::StmtKind::StaticData)
            },

            ast::Stmt::Instr(instr) => {
                let instr = consts.subst_instr(instr);
                validate_instr(instr, diag).map(asm::StmtKind::Instr)
            },
        };

        // Error recovery: No quitting early if errors were produced above because we want to
        // get through as many statements as possible before exiting.

        if let Some(kind) = validated_stmt {
            match &mut stmts {
                Some(stmts) => {
                    stmts.push(asm::Stmt {labels, kind});
                    labels = Vec::new();
                },
                None => diag.span_error(kind.span(), "all assembly statements must occur within a section, e.g. `section .code`").emit(),
            }
        }
    }

    asm::Program {code_section, static_section}
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

/// Validates a static data directive to ensure that it is valid assembly language
fn validate_static_data(stmt: ast::StaticData, diag: &Diagnostics) -> Option<asm::StaticData> {
    todo!()
}

/// Validates an instruction to ensure that it is valid assembly language
fn validate_instr(stmt: ast::Instr, diag: &Diagnostics) -> Option<asm::Instr> {
    todo!()
}
