use std::str;
use std::mem;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::ast;
use crate::parser::{SourceFiles, collect_tokens, parse_program};
use crate::diagnostics::Diagnostics;

/// Attempts to expand all `.include` directives in a program
///
/// Up to `max_attempts` passes will be made. If all `.include` directives have not been resolved
/// by then, the program is assumed to not have any fixed point and an error will be produced.
///
/// If an error occurs while processing an `.include` or while parsing any of the included files,
/// the errors will be outputted and the program will be returned in its current state.
///
/// If no errors occur, the returned program is guaranteed to not have any remaining `.include`
/// directives in it.
pub fn expand_includes(
    prog: ast::Program,
    source_files: &Arc<RwLock<SourceFiles>>,
    diag: &Diagnostics,
    max_attempts: usize,
) -> ast::Program {
    let ast::Program {mut stmts} = prog;
    // We will have at least the current amount of statements by the end of this
    let mut expanded_stmts = Vec::with_capacity(stmts.len());

    for _ in 0..max_attempts {
        // Avoids a lot of unnecessary copying in exchange for an extra pass over the statements
        let has_includes = stmts.iter().any(|stmt| stmt.is_include());
        if !has_includes {
            return ast::Program {stmts};
        }

        // Use drain() so we can potentially reuse the allocation on the next pass
        for stmt in stmts.drain(..) {
            let ast::Include {path} = match stmt {
                ast::Stmt::Include(include) => include,
                stmt => {
                    expanded_stmts.push(stmt);
                    continue;
                },
            };

            let path_span = path.span;
            let path = match str::from_utf8(&path.value) {
                Ok(path) => path,
                Err(err) => {
                    diag.span_error(path.span, format!("included path was not valid UTF-8: {}", err)).emit();

                    // Finish this pass before stopping in case there are further errors
                    continue;
                },
            };

            // Note that we don't validate the extension of included files since that can be anything

            //TODO: Resolve path relative to currently processed file

            // Need this separate variable so that the lock on source files ends before diag.span_error()
            let included_file = source_files.write().add_file(&*path);
            let included_file = match included_file {
                Ok(file_handle) => file_handle,
                Err(err) => {
                    diag.span_error(path_span, format!("unable to read included source file: `{}`: {}", path, err)).emit();
                    // Finish this pass before stopping in case there are further errors
                    continue;
                },
            };

            let tokens = collect_tokens(source_files.read().source(included_file), diag);
            if diag.emitted_errors() > 0 {
                // Finish this pass before stopping in case there are further errors
                continue;
            }

            let ast::Program {stmts: included_stmts} = parse_program(&tokens, diag);
            if diag.emitted_errors() > 0 {
                // Finish this pass before stopping in case there are further errors
                continue;
            }

            expanded_stmts.extend(included_stmts);
        }

        // For the next pass, iterate over the expanded statements and reuse the stmts allocation
        // for the next set of expanded statements
        mem::swap(&mut stmts, &mut expanded_stmts);

        // Quit if this pass failed
        if diag.emitted_errors() > 0 {
            return ast::Program {stmts};
        }
    }

    diag.error("surpassed maximum include expansion attempts").emit();

    ast::Program {stmts}
}
