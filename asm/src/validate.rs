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

    let mut code_section: Option<asm::Section> = None;
    let mut static_section: Option<asm::Section> = None;
    let mut stmts = None;
    let mut labels = Vec::new();
    for stmt in prog.stmts {
        let kind = match stmt {
            ast::Stmt::Label(label) => {
                labels.push(label);
                continue;
            },

            ast::Stmt::Section(section) => match section.kind {
                ast::SectionKind::Code(_) => {
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

                ast::SectionKind::Static(_) => {
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
                asm::StmtKind::StaticData(validate_static_data(static_data, diag))
            },

            ast::Stmt::Instr(instr) => {
                let instr = consts.subst_instr(instr);
                asm::StmtKind::Instr(asm::Instr::validate(instr, diag))
            },
        };

        // Error recovery: No quitting early if errors were produced above because we want to
        // get through as many statements as possible before exiting.

        match &mut stmts {
            Some(stmts) => {
                stmts.push(asm::Stmt {labels, kind});
                labels = Vec::new();
            },
            None => diag.span_error(kind.span(), "all assembly statements must occur within a section, e.g. `section .code`").emit(),
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
                assert!(labels.insert(label.clone()),
                    "bug: expected to be inserting label for the first time");
            },
        }
    }

    labels
}

/// Validates a static data directive to ensure that it is valid assembly language
fn validate_static_data(stmt: ast::StaticData, diag: &Diagnostics) -> asm::StaticData {
    match stmt {
        ast::StaticData::StaticBytes(static_bytes) => {
            asm::StaticData::StaticBytes(validate_static_bytes(static_bytes, diag))
        },

        ast::StaticData::StaticZero(static_zero) => {
            asm::StaticData::StaticZero(validate_static_zero(static_zero, diag))
        },

        ast::StaticData::StaticUninit(static_uninit) => {
            asm::StaticData::StaticUninit(validate_static_uninit(static_uninit, diag))
        },

        ast::StaticData::StaticByteStr(ast::StaticByteStr {bytes, span}) => {
            asm::StaticData::StaticByteStr(asm::StaticByteStr {bytes, span})
        },
    }
}

fn validate_static_bytes(static_bytes: ast::StaticBytes, diag: &Diagnostics) -> asm::StaticBytes {
    let ast::StaticBytes {size, value, span} = static_bytes;
    let ast::Integer {value, span: value_span} = value;

    match size {
        1 => {
            if value < 0 || value > u8::max_value() as i128 {
                diag.span_error(span, format!("value `{}` for `.b1` must be in the range `0` to `{}`", value, u8::max_value())).emit();
            }

            asm::StaticBytes {
                // Error recovery: if an error is produced above, we'll just end up with the result
                // of `as` when casting with overflow
                value: asm::StaticBytesValue::B1((value as u8).to_le_bytes(), value_span),
                span,
            }
        },

        2 => {
            if value < 0 || value > u16::max_value() as i128 {
                diag.span_error(span, format!("value `{}` for `.b2` must be in the range `0` to `{}`", value, u16::max_value())).emit();
            }

            asm::StaticBytes {
                // Error recovery: if an error is produced above, we'll just end up with the result
                // of `as` when casting with overflow
                value: asm::StaticBytesValue::B2((value as u16).to_le_bytes(), value_span),
                span,
            }
        },

        4 => {
            if value < 0 || value > u32::max_value() as i128 {
                diag.span_error(span, format!("value `{}` for `.b4` must be in the range `0` to `{}`", value, u32::max_value())).emit();
            }

            asm::StaticBytes {
                // Error recovery: if an error is produced above, we'll just end up with the result
                // of `as` when casting with overflow
                value: asm::StaticBytesValue::B4((value as u32).to_le_bytes(), value_span),
                span,
            }
        },

        8 => {
            if value < 0 || value > u64::max_value() as i128 {
                diag.span_error(span, format!("value `{}` for `.b8` must be in the range `0` to `{}`", value, u64::max_value())).emit();
            }

            asm::StaticBytes {
                // Error recovery: if an error is produced above, we'll just end up with the result
                // of `as` when casting with overflow
                value: asm::StaticBytesValue::B8((value as u64).to_le_bytes(), value_span),
                span,
            }
        },

        _ => unreachable!("bug: unexpected size of static bytes: `{}`", size),
    }
}

fn validate_static_zero(static_zero: ast::StaticZero, diag: &Diagnostics) -> asm::StaticZero {
    let ast::StaticZero {nbytes, span} = static_zero;

    asm::StaticZero {
        nbytes: validate_size(nbytes, diag),
        span,
    }
}

fn validate_static_uninit(static_uninit: ast::StaticUninit, diag: &Diagnostics) -> asm::StaticUninit {
    let ast::StaticUninit {nbytes, span} = static_uninit;

    asm::StaticUninit {
        nbytes: validate_size(nbytes, diag),
        span,
    }
}

fn validate_size(size: ast::Integer, diag: &Diagnostics) -> asm::Size {
    // The value of an `Integer` is already guaranteed to be <= u64::max() so we just have to
    // ensure that it is non-negative
    let value = if size.value >= 0 {
        size.value as u64
    } else {
        diag.span_error(size.span, "number of bytes must be non-negative").emit();

        // Error recovery: just pretend the size is zero
        0
    };

    asm::Size {
        value,
        span: size.span,
    }
}
