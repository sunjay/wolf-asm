use std::mem;

use wolf_asm::{executable as exec, asm::layout::InstrLayout};

use crate::memory::{Memory, OutOfBounds};

/// Writes a value into memory at the given address and then returns the address
/// of the next byte after the data that was just written
pub trait WriteMemory {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds>;
}

impl<T: WriteMemory> WriteMemory for [T] {
    fn write_into(&self, mem: &mut Memory, mut addr: u64) -> Result<u64, OutOfBounds> {
        for value in self {
            addr = value.write_into(mem, addr)?;
        }
        Ok(addr)
    }
}

impl WriteMemory for u8 {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        mem.set(addr, *self)?;
        Ok(addr + mem::size_of::<Self>() as u64)
    }
}

impl WriteMemory for u64 {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        mem.write_u64(addr, *self)?;
        Ok(addr + mem::size_of::<Self>() as u64)
    }
}

impl WriteMemory for exec::Executable {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        let exec::Executable {code_section, static_section} = self;

        let addr = code_section.write_into(mem, addr)?;
        static_section.write_into(mem, addr)
    }
}

impl WriteMemory for exec::Stmt {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        use exec::Stmt::*;
        match self {
            StaticData(data) => data.write_into(mem, addr),
            Instr(instr) => instr.write_into(mem, addr),
        }
    }
}

impl WriteMemory for exec::StaticData {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        use exec::StaticData::*;
        match self {
            StaticBytes(data) => data.write_into(mem, addr),
            StaticZero(data) => data.write_into(mem, addr),
            StaticUninit(data) => data.write_into(mem, addr),
            StaticByteStr(data) => data.write_into(mem, addr),
        }
    }
}

impl WriteMemory for InstrLayout {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        let instr = self.to_binary();
        instr.write_into(mem, addr)
    }
}

impl WriteMemory for exec::StaticBytes {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        use exec::StaticBytes::*;
        match self {
            B1(bytes) => bytes.write_into(mem, addr),
            B2(bytes) => bytes.write_into(mem, addr),
            B4(bytes) => bytes.write_into(mem, addr),
            B8(bytes) => bytes.write_into(mem, addr),
        }
    }
}

impl WriteMemory for exec::StaticZero {
    fn write_into(&self, mem: &mut Memory, mut addr: u64) -> Result<u64, OutOfBounds> {
        let &Self {nbytes} = self;
        for _ in 0..nbytes {
            addr = 0u8.write_into(mem, addr)?;
        }
        Ok(addr)
    }
}

impl WriteMemory for exec::StaticUninit {
    fn write_into(&self, _mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        let &Self {nbytes} = self;
        Ok(addr + mem::size_of::<u8>() as u64 * nbytes)
    }
}

impl WriteMemory for exec::StaticByteStr {
    fn write_into(&self, mem: &mut Memory, addr: u64) -> Result<u64, OutOfBounds> {
        let Self {bytes} = self;
        bytes.write_into(mem, addr)
    }
}
