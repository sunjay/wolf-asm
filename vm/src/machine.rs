use thiserror::Error;

use crate::{
    memory::{Memory, OutOfBounds},
    registers::Registers,
    flags::Flags,
    decode::{Instr, DecodeError},
    execute::{Execute, ExecuteError},
};

/// The address used to indicate that the program should quit
const QUIT_ADDR: usize = u64::MAX as usize;

/// Whether the program should continue running
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramStatus {
    Continue,
    Quit,
}

#[derive(Debug, Error, Clone)]
#[error(transparent)]
pub enum ExecutionError {
    OutOfBounds(#[from] OutOfBounds),
    DecodeError(#[from] DecodeError),
    ExecuteError(#[from] ExecuteError),
}

#[derive(Debug, PartialEq)]
pub struct Machine {
    /// Holds the address of the next instruction to execute
    pub program_counter: usize,
    /// The machine memory unit
    pub memory: Memory,
    /// The machine registers
    pub registers: Registers,
    /// The machine flags/status register
    pub flags: Flags,
}

impl Machine {
    /// Decode and run the instruction at the program counter
    pub fn step(&mut self) -> Result<ProgramStatus, ExecutionError> {
        let instr = self.memory.read_u64(self.program_counter)?;
        let instr = Instr::decode(instr)?;
        self.program_counter += instr.size_bytes() as usize;

        instr.execute(self)?;

        if self.program_counter == QUIT_ADDR {
            Ok(ProgramStatus::Quit)
        } else {
            Ok(ProgramStatus::Continue)
        }
    }

    pub fn push_quit_addr(&mut self) -> Result<(), ExecutionError> {
        self.push_immediate(QUIT_ADDR as i128)
    }

    pub fn push_immediate(&mut self, imm: i128) -> Result<(), ExecutionError> {
        todo!()
    }
}
