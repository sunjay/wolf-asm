use std::fs;
use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use rayon::prelude::*;
use tempfile::{NamedTempFile, TempPath};

const VM_EXEC_PATH: &str = env!("CARGO_BIN_EXE_wolf-vm");

#[test]
fn run_fail() {
    // Pass the environment variable TESTVM=overwrite to overwrite the stdout and stderr files
    let overwrite_expected_output = env::var("TESTVM")
        .map(|val| val == "overwrite")
        .unwrap_or(false);

    let tests_dir = Path::new("../tests/run-fail");

    let test_files = tests_dir.read_dir()
        .unwrap_or_else(|err| panic!("Failed to read test files directory '{}': {}", tests_dir.display(), err));
    test_files.par_bridge().panic_fuse().for_each(|entry| {
        let entry = entry.unwrap_or_else(|err| panic!("Failed to read directory entry in '{}': {}", tests_dir.display(), err));
        let entry_path = entry.path();
        if entry_path.is_dir() || entry_path.extension() != Some(OsStr::new("wa")) {
            return;
        }

        println!("[run-fail] Running assembler on {}", entry_path.display());
        let exec_path = run_assembler(&entry_path);

        let input_path = entry_path.with_extension("stdin");
        let stdin = if input_path.exists() {
            Some(fs::File::open(&input_path)
                .unwrap_or_else(|err| panic!("Failed to read '{}': {}", input_path.display(), err)))
        } else {
            None
        };

        println!("[run-fail] Running VM on {} ({})", entry_path.display(), exec_path.display());
        match run_vm(&exec_path, stdin) {
            Ok(_) => {
                panic!("VM should have failed to run '{}'", entry_path.display());
            },
            Err((stdout, stderr)) => {
                // Check the stdout and stderr output against what's expected
                // The stdout file is optional
                let stdout_file = entry_path.with_extension("stdout");
                let stderr_file = entry_path.with_extension("stderr");

                if overwrite_expected_output {
                    if !stdout.is_empty() {
                        fs::write(&stdout_file, &stdout)
                            .unwrap_or_else(|err| panic!("Failed to write expected output to '{}': {}", stdout_file.display(), err));
                    }
                    fs::write(&stderr_file, &stderr)
                        .unwrap_or_else(|err| panic!("Failed to write expected error to '{}': {}", stderr_file.display(), err));
                    return;
                }

                if stdout_file.exists() {
                    let expected_stdout = fs::read_to_string(&stdout_file)
                        .unwrap_or_else(|err| panic!("Failed to open '{}': {}", stdout_file.display(), err));
                    if stdout != expected_stdout {
                        panic!("Output for '{}' did not match '{}'", entry_path.display(), stdout_file.display());
                    }
                } else {
                    if !stdout.is_empty() {
                        panic!("Expected no output for for '{}'", entry_path.display());
                    }
                }

                let expected_stderr = fs::read_to_string(&stderr_file)
                    .unwrap_or_else(|err| panic!("Failed to open '{}': {}", stderr_file.display(), err));
                if stderr != expected_stderr {
                    panic!("Error for '{}' did not match '{}'", entry_path.display(), stderr_file.display());
                }

                println!("[run-fail] Finished running VM on {}", entry_path.display());
            },
        }
    });
}

#[test]
fn run_pass() {
    // Pass the environment variable TESTVM=overwrite to overwrite the stdout and stderr files
    let overwrite_expected_output = env::var("TESTVM")
        .map(|val| val == "overwrite")
        .unwrap_or(false);

    let tests_dir = Path::new("../tests/run-pass");

    let test_files = tests_dir.read_dir()
        .unwrap_or_else(|err| panic!("Failed to read test files directory '{}': {}", tests_dir.display(), err));
    test_files.par_bridge().panic_fuse().for_each(|entry| {
        let entry = entry.unwrap_or_else(|err| panic!("Failed to read directory entry in '{}': {}", tests_dir.display(), err));
        let entry_path = entry.path();
        if entry_path.is_dir() || entry_path.extension() != Some(OsStr::new("wa")) {
            return;
        }

        println!("[run-pass] Running assembler on {}", entry_path.display());
        let exec_path = run_assembler(&entry_path);

        let input_path = entry_path.with_extension("stdin");
        let stdin = if input_path.exists() {
            Some(fs::File::open(&input_path)
                .unwrap_or_else(|err| panic!("Failed to read '{}': {}", input_path.display(), err)))
        } else {
            None
        };

        println!("[run-pass] Running VM on {} ({})", entry_path.display(), exec_path.display());
        match run_vm(&exec_path, stdin) {
            Ok((stdout, stderr)) => {
                // Check the stdout and stderr output against what's expected
                let stdout_file = entry_path.with_extension("stdout");

                if overwrite_expected_output {
                    fs::write(&stdout_file, &stdout)
                        .unwrap_or_else(|err| panic!("Failed to write expected output to '{}': {}", stdout_file.display(), err));
                    return;
                }

                let expected_stdout = fs::read_to_string(&stdout_file)
                    .unwrap_or_else(|err| panic!("Failed to open '{}': {}", stdout_file.display(), err));

                if stdout != expected_stdout {
                    panic!("Output for '{}' did not match '{}'", entry_path.display(), stdout_file.display());
                }
                if !stderr.is_empty() {
                    panic!("stderr for '{}' was not empty", entry_path.display());
                }

                println!("[run-pass] Finished running VM on {}", entry_path.display());
            },
            Err(_) => {
                panic!("VM failed to run '{}'", entry_path.display());
            },
        }
    });
}

/// Runs the assembler on a single file, returning the path to the generated
/// executable or panicking if an error occurs.
fn run_assembler(source_path: &Path) -> TempPath {
    // The path to the executable that will be generated
    // Using temp file so this is reliably cleaned up
    let executable = NamedTempFile::new()
        .unwrap_or_else(|err| panic!("Failed to created temporary file: {}", err));

    let asm_exec_path = Path::new(VM_EXEC_PATH).parent().unwrap().join("wolf-asm");
    let status = Command::new(asm_exec_path)
        .arg(source_path)
        .arg("-o")
        .arg(executable.path())
        .status()
        .unwrap_or_else(|err| panic!("Failed to run assembler: {}", err));

    // Check if assembler failed
    if !status.success() {
        panic!("Assembler failed for '{}'", source_path.display());
    }

    executable.into_temp_path()
}

/// Runs the given executable using the virtual machine
///
/// Returns (stdout, stderr)
fn run_vm(exec_path: &Path, stdin: Option<fs::File>) -> Result<(String, String), (String, String)> {
    let stdin = stdin.map(Stdio::from).unwrap_or_else(Stdio::null);

    let output = Command::new(VM_EXEC_PATH)
        .arg(exec_path)
        .stdin(stdin)
        .output()
        .unwrap_or_else(|err| panic!("Failed to spawn VM process: {}", err));

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("VM stdout was not valid UTF-8: {}", err));
    let stderr = String::from_utf8(output.stderr)
        .unwrap_or_else(|err| panic!("VM stderr was not valid UTF-8: {}", err));

    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err((stdout, stderr))
    }
}
