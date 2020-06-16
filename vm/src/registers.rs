use std::fmt;

use wolf_asm::asm::{self, layout::Reg};

use crate::reinterpret::Reinterpret;

const REGISTERS: usize = asm::REGISTERS as usize;

#[derive(Clone)]
pub struct Registers {
    registers: [u64; REGISTERS],
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            registers: [0; REGISTERS],
        }
    }
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let Self {registers} = self;
        f.debug_list().entries(&registers[..]).finish()
    }
}

impl PartialEq for Registers {
    fn eq(&self, other: &Self) -> bool {
        let Self {registers} = self;
        let Self {registers: other_registers} = other;

        if registers.len() != other_registers.len() {
            unreachable!("bug: should always have the same number of registers");
        } else {
            registers.iter().zip(&other_registers[..]).all(|(x, y)| x.eq(y))
        }
    }
}

impl Registers {
    /// Creates a new set of registers with the stack pointer and frame pointer
    /// initialized to the given value
    pub fn new(stack_end_addr: usize) -> Self {
        let mut regs = Self::default();

        let stack_pointer = stack_end_addr as u64;
        regs.store_sp(stack_pointer);
        // The initial stack frame starts at the end of the stack
        regs.store_fp(stack_pointer);

        regs
    }

    /// Loads the given register value
    pub fn load<R: Reinterpret<u64>>(&self, reg: Reg) -> R {
        let index = reg.into_value() as usize;
        // Safety: `Reg` is guaranteed to contain a value between 0 and 63
        let value = unsafe { *self.registers.get_unchecked(index) };
        R::reinterpret(value)
    }

    /// Stores the given value into the given register
    pub fn store<R>(&mut self, reg: Reg, new_value: R)
        where u64: Reinterpret<R>,
    {
        let index = reg.into_value() as usize;
        // Safety: `Reg` is guaranteed to contain a value between 0 and 63
        let value = unsafe { self.registers.get_unchecked_mut(index) };
        *value = u64::reinterpret(new_value);
    }

    /// Loads the value of the stack pointer
    pub fn load_sp<R: Reinterpret<u64>>(&self) -> R {
        self.load(asm::RegisterKind::StackPointer.into())
    }

    /// Stores the given value into the stack pointer
    pub fn store_sp<R>(&mut self, new_value: R)
        where u64: Reinterpret<R>,
    {
        self.store(asm::RegisterKind::StackPointer.into(), new_value)
    }

    /// Loads the value of the frame pointer
    pub fn load_fp<R: Reinterpret<u64>>(&self) -> R {
        self.load(asm::RegisterKind::FramePointer.into())
    }

    /// Stores the given value into the frame pointer
    pub fn store_fp<R>(&mut self, new_value: R)
        where u64: Reinterpret<R>,
    {
        self.store(asm::RegisterKind::FramePointer.into(), new_value)
    }
}
