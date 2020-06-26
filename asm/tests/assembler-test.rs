use std::fs;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use rayon::prelude::*;
use tempfile::{NamedTempFile, TempPath};

const EXEC_PATH: &str = env!("CARGO_BIN_EXE_wolf-asm");

#[test]
fn ui() {
    // Pass the environment variable TESTASSEMBLER=overwrite to overwrite the stderr files
    let overwrite_expected_output = env::var("TESTASSEMBLER")
        .map(|val| val == "overwrite")
        .unwrap_or(false);

    let tests_dir = Path::new("tests/ui");
    // Relative paths make the output easier to read
    let tests_dir = tests_dir.strip_prefix(env::current_dir().unwrap()).unwrap_or(&tests_dir);

    let test_files = tests_dir.read_dir()
        .unwrap_or_else(|err| panic!("Failed to read test files directory '{}': {}", tests_dir.display(), err));
    test_files.par_bridge().panic_fuse().for_each(|entry| {
        let entry = entry.unwrap_or_else(|err| panic!("Failed to read directory entry in '{}': {}", tests_dir.display(), err));
        let entry_path = entry.path();
        if entry_path.is_dir() || entry_path.extension() != Some(OsStr::new("wa")) {
            return;
        }

        println!("[ui] Running assembler on {}", entry_path.display());
        match run_assembler(&entry_path) {
            Ok(_) => {
                panic!("Assembler should have failed for '{}'", entry.path().display());
            },
            Err(stderr) => {
                // Check the stderr output against what's expected
                let stderr_file = entry_path.with_extension("stderr");

                if overwrite_expected_output {
                    fs::write(&stderr_file, &stderr)
                        .unwrap_or_else(|err| panic!("Failed to write expected output to '{}': {}", stderr_file.display(), err));
                    return;
                }

                let expected_stderr = fs::read_to_string(&stderr_file)
                    .unwrap_or_else(|err| panic!("Failed to open '{}': {}", stderr_file.display(), err));

                if stderr != expected_stderr {
                    panic!("Error for '{}' did not match '{}'", entry_path.display(), stderr_file.display());
                }

                println!("[ui] Finished running assembler on {}", entry_path.display());
            },
        }
    });
}

#[test]
fn run_pass() {
    let tests_dir = Path::new("tests/run-pass");

    let test_files = tests_dir.read_dir()
        .unwrap_or_else(|err| panic!("Failed to read test files directory '{}': {}", tests_dir.display(), err));
    test_files.par_bridge().panic_fuse().for_each(|entry| {
        let entry = entry.unwrap_or_else(|err| panic!("Failed to read directory entry in '{}': {}", tests_dir.display(), err));
        let entry_path = entry.path();
        if entry_path.is_dir() || entry_path.extension() != Some(OsStr::new("wa")) {
            return;
        }

        println!("[run-pass] Running assembler on {}", entry_path.display());
        match run_assembler(&entry_path) {
            Ok((exec_path, stdout)) => {
                // The assembler currently doesn't generate output on success.
                // If this changes later we should probably save that expected
                // output and incorporate it into these tests.
                assert!(stdout.is_empty(), "Assembler generated output despite success for '{}'", entry_path.display());

                let exec_meta = fs::metadata(&exec_path)
                    .unwrap_or_else(|err| panic!("Failed to read metadata for '{}': {}", exec_path.display(), err));
                assert!(exec_meta.len() > 0, "Generated executable for '{}' should be non-empty", entry_path.display());

                println!("[run-pass] Assembler succeeded for {}", entry_path.display());
            },
            Err(err) => panic!("Assembler failed for '{}'\n--- ERROR MESSAGE START --\n{}--- ERROR MESSAGE END ---\n", entry_path.display(), err),
        }
    });
}

/// Runs the assembler on a single file, returning (path to the generated
/// executable, stdout contents) if the assembler succeeded. Returns the
/// assembler error message if the assembler failed.
fn run_assembler(source_path: &Path) -> Result<(TempPath, String), String> {
    // The path to the executable that will be generated
    // Using temp file so this is reliably cleaned up
    let executable = NamedTempFile::new()
        .unwrap_or_else(|err| panic!("Failed to created temporary file: {}", err));

    let output = Command::new(EXEC_PATH)
        .arg(source_path)
        .arg("--color=never")
        .arg("-o")
        .arg(executable.path())
        .output()
        .unwrap_or_else(|err| panic!("Failed to run assembler: {}", err));

    // Check if assembler failed
    if !output.status.success() {
        return Err(String::from_utf8(output.stderr)
            .unwrap_or_else(|err| panic!("Assembler stderr for '{}' was not valid UTF-8: {}", executable.path().display(), err)));
    }

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("Assembler stdout for '{}' was not valid UTF-8: {}", executable.path().display(), err));

    Ok((executable.into_temp_path(), stdout))
}
