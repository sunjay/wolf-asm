//! wolf-asm - The Wolf Assembly Language assembler
//!
//! Takes assembly code and generates machine code for the wolf virtual machine

#![deny(unused_must_use)]

use std::env;
use std::process;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs::File;

use parking_lot::RwLock;
use termcolor::ColorChoice;
use structopt::StructOpt;

use wolf_asm::{
    diagnostics::Diagnostics,
    parser::{self, SourceFiles},
    include_expansion::expand_includes,
    validate::validate_program,
    label_offsets::LabelOffsets,
    executable::Executable,
};

/// The maximum number of times we are allowed to recurse when expanding `.include` directives
const MAX_INCLUDE_DEPTH: usize = 50;

/// A command line argument that configures the coloring of the output
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorArg(pub ColorChoice);

impl Default for ColorArg {
    fn default() -> Self {
        ColorArg(ColorChoice::Auto)
    }
}

impl ColorArg {
    /// Allowed values the argument
    pub const VARIANTS: &'static [&'static str] = &["auto", "always", "ansi", "never"];
}

impl FromStr for ColorArg {
    type Err = &'static str;

    fn from_str(src: &str) -> Result<ColorArg, &'static str> {
        match src {
            _ if src.eq_ignore_ascii_case("auto") => Ok(ColorArg(ColorChoice::Auto)),
            _ if src.eq_ignore_ascii_case("always") => Ok(ColorArg(ColorChoice::Always)),
            _ if src.eq_ignore_ascii_case("ansi") => Ok(ColorArg(ColorChoice::AlwaysAnsi)),
            _ if src.eq_ignore_ascii_case("never") => Ok(ColorArg(ColorChoice::Never)),
            _ => Err("valid values: auto, always, ansi, never"),
        }
    }
}

impl Into<ColorChoice> for ColorArg {
    fn into(self) -> ColorChoice {
        self.0
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wolf-asm", about)]
struct AssemblerOptions {
    /// The assembly language file (`.wa`) to generate an executable for
    #[structopt(name = "input", parse(from_os_str))]
    program_path: PathBuf,
    /// Write output to <file>
    #[structopt(short = "o", name = "file")]
    output_path: Option<PathBuf>,
    /// Configure coloring of output
    #[structopt(long = "color", parse(try_from_str), default_value = "auto",
        possible_values = ColorArg::VARIANTS, case_insensitive = true)]
    pub color: ColorArg,
}

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
    let AssemblerOptions {program_path, output_path, color} = AssemblerOptions::from_args();

    let source_files = Arc::new(RwLock::new(SourceFiles::default()));
    let diag = Diagnostics::new(source_files.clone(), color.into());

    // Check that the path and stem are valid
    let program_stem = match (program_path.file_stem(), program_path.extension()) {
        (Some(stem), Some(ext)) if !stem.is_empty() && ext == "wa" => stem,
        _ => quit!(&diag, "Invalid input path. Must use extension `wa`"),
    };

    // Default output path is the input path without its extension
    let output_path = output_path.as_ref().map(|p| p.as_path())
        .unwrap_or_else(|| Path::new(program_stem));
    // Append the current directory to the output path if necessary
    let output_path = if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        let current_dir = env::current_dir()
            .unwrap_or_else(|err| quit!(&diag, "Could not access current directory: {}", err));
        current_dir.join(output_path)
    };

    // Need this separate statement so we don't hold the write() lock in the
    // error case and end up with a deadlock
    let root_file = source_files.write().add_file(&program_path);
    let root_file = root_file.unwrap_or_else(|err| {
        quit!(&diag, "Could not read source file `{}`: {}", program_path.display(), err)
    });
    let program = {
        // New scope because we want to drop this lock guard as soon as possible
        let files = source_files.read();
        let tokens = parser::collect_tokens(files.source(root_file), &diag);
        check_errors!(&diag);
        parser::parse_program(&tokens, &diag)
    };
    check_errors!(&diag);

    let expanded_program = expand_includes(&program_path, program, &source_files, &diag, MAX_INCLUDE_DEPTH);
    check_errors!(&diag);

    let validated_program = validate_program(expanded_program, &diag);
    check_errors!(&diag);

    let label_offsets = LabelOffsets::new(&validated_program);
    let exec = Executable::layout_executable(validated_program, &diag, &label_offsets);
    check_errors!(&diag);

    let output_file = File::create(&output_path)
        .unwrap_or_else(|err| quit!(&diag, "Could not open output path `{}`: {}", output_path.display(), err));
    bincode::serialize_into(output_file, &exec)
        .unwrap_or_else(|err| quit!(&diag, "Unable to write executable `{}`: {}", output_path.display(), err));
}
