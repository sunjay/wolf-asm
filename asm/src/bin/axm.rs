//! axm - the ax assembly language assembler
//!
//! Takes assembly code and generates machine code for the ax virtual machine

use std::process;
use std::sync::Arc;
use std::path::Path;

use parking_lot::RwLock;
use termcolor::ColorChoice;

use asm::{
    parser::{self, SourceFiles},
    diagnostics::Diagnostics,
};

macro_rules! quit {
    ($diag:expr, $($args:tt)*) => {
        {
            $diag.error(format!($($args)*)).emit();
            process::exit(1);
        }
    };
}

macro_rules! check_errors {
    ($diag:expr) => {
        let diag = $diag;
        match diag.emitted_errors() {
            0 => {},
            1 => quit!(diag, "aborting due to 1 previous error"),
            errors => quit!(diag, "aborting due to {} previous errors", errors),
        }
    };
}

fn main() {
    //TODO: Parse command line arguments
    let program_path = Path::new("asm/tests/ui/syntax.ax");
    let color = ColorChoice::Auto;

    let source_files = Arc::new(RwLock::new(SourceFiles::default()));
    let diag = Diagnostics::new(source_files.clone(), color.into());

    let root_file = source_files.write().add_file(program_path)
        .unwrap_or_else(|err| quit!(&diag, "Could not read source file `{}`: {}", program_path.display(), err));
    let program = {
        // New scope because we want to drop this lock guard as soon as possible
        let files = source_files.read();
        let tokens = parser::collect_tokens(files.source(root_file), &diag);
        check_errors!(&diag);
        parser::parse_program(&tokens, &diag)
    };
    println!("{:#?}", program);
    check_errors!(&diag);
}
