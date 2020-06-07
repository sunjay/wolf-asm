use std::str;
use std::sync::Arc;
use std::path::Path;
use std::borrow::Cow;

use parking_lot::RwLock;

use crate::ast;
use crate::parser::{SourceFiles, collect_tokens, parse_program};
use crate::diagnostics::Diagnostics;

/// Attempts to expand all `.include` directives in a program
///
/// Recurses up to `depth` times, after which an error will be produced if all `.include`
/// directives have not been resolved.
///
/// If an error occurs while reading an included path or while parsing any of the included files,
/// the errors will be outputted and the program will be returned in its current state.
///
/// If no errors occur, the returned program is guaranteed to not have any remaining `.include`
/// directives in it.
pub fn expand_includes(
    prog_path: &Path,
    prog: ast::Program,
    source_files: &Arc<RwLock<SourceFiles>>,
    diag: &Diagnostics,
    depth: usize,
) -> ast::Program {
    // This avoids a lot of unnecessary copying in exchange for an extra pass over the statements
    let has_includes = prog.stmts.iter().any(|stmt| stmt.is_include());
    if !has_includes {
        return prog;
    }

    if depth == 0 {
        diag.error("surpassed maximum include expansion attempts").emit();
        return prog;
    }

    let ast::Program {stmts} = prog;
    // We will have at least the current amount of statements by the end of this
    let mut expanded_stmts = Vec::with_capacity(stmts.len());

    for stmt in stmts {
        let ast::Include {path: included_path} = match stmt {
            ast::Stmt::Include(include) => include,
            stmt => {
                expanded_stmts.push(stmt);
                continue;
            },
        };

        let path_span = included_path.span;
        let included_path = match str::from_utf8(&included_path.value) {
            Ok(path) => Path::new(path),
            Err(err) => {
                diag.span_error(included_path.span, format!("included path was not valid UTF-8: {}", err)).emit();

                // Finish this pass before stopping in case there are further errors
                continue;
            },
        };

        // Note that we don't validate the extension of included files since that can be anything

        // Included paths are resolved relative to the file they are included in
        let included_path = if included_path.is_relative() {
            // Even `Path::new("foo.ax").parent()` will return `Some(Path::new(""))`
            let parent_dir = prog_path.parent()
                .expect("bug: if a source file has been read, it must have a parent directory");
            Cow::Owned(parent_dir.join(included_path))
        } else {
            Cow::Borrowed(included_path)
        };

        // Need this separate variable so that the lock on source files ends before diag.span_error()
        let included_file = source_files.write().add_file(&included_path);
        let included_file = match included_file {
            Ok(file_handle) => file_handle,
            Err(err) => {
                diag.span_error(path_span, format!("unable to read included source file: `{}`: {}", included_path.display(), err)).emit();
                // Finish this pass before stopping in case there are further errors
                continue;
            },
        };

        let tokens = collect_tokens(source_files.read().source(included_file), diag);
        if diag.emitted_errors() > 0 {
            // Finish this pass before stopping in case there are further errors
            continue;
        }

        let included_prog = parse_program(&tokens, diag);
        if diag.emitted_errors() > 0 {
            // Finish this pass before stopping in case there are further errors
            continue;
        }

        // Recurse and expand the included program
        let ast::Program {stmts: included_stmts} = expand_includes(
            &included_path,
            included_prog,
            source_files,
            diag,
            depth-1,
        );
        // Even if this expansion ends with errors, we still want to include as much in the final
        // result as we can, that's why we aren't checking `diag.emitted_errors()` here.

        expanded_stmts.extend(included_stmts);
    }

    ast::Program {stmts: expanded_stmts}
}