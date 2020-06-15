use wolf_asm::executable as exec;

use crate::memory::{Memory, OutOfBounds};

/// Writes a value into memory at the given address and then returns the address
/// of the next byte after the data that was just written
pub trait WriteMemory {
    fn write_into(&self, mem: &mut Memory, addr: usize) -> Result<usize, OutOfBounds>;
}

impl WriteMemory for exec::Executable {
    fn write_into(&self, mem: &mut Memory, addr: usize) -> Result<usize, OutOfBounds> {
        todo!()
    }
}
