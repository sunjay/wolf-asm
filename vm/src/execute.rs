use thiserror::Error;

use crate::reinterpret::Reinterpret;
use crate::machine::Machine;
use crate::registers::Registers;
use crate::memory::OutOfBounds;
use crate::flags::{Flags, CF, ZF, SF, OF};
use crate::decode::*;

/// The address used to indicate that the program should quit
pub const QUIT_ADDR: u64 = u64::MAX;

fn size_bytes_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}

pub trait StoreDestination {
    fn store_dest<R>(&mut self, dest: Destination, value: R)
        where u64: Reinterpret<R>;
}

impl StoreDestination for Registers {
    fn store_dest<R>(&mut self, dest: Destination, value: R)
        where u64: Reinterpret<R>
    {
        match dest {
            Destination::Register(reg) => self.store(reg, value),
        }
    }
}

pub trait Operand {
    fn into_value<R: Reinterpret<u64>>(self, regs: &Registers) -> R;
}

impl Operand for Source {
    fn into_value<R: Reinterpret<u64>>(self, regs: &Registers) -> R {
        match self {
            Source::Register(reg) => regs.load(reg),
            Source::Immediate(imm) => {
                let imm = u64::reinterpret(imm);
                R::reinterpret(imm)
            },
        }
    }
}

impl Operand for Destination {
    fn into_value<R: Reinterpret<u64>>(self, regs: &Registers) -> R {
        match self {
            Destination::Register(reg) => regs.load(reg),
        }
    }
}

impl Operand for Location {
    fn into_value<R: Reinterpret<u64>>(self, regs: &Registers) -> R {
        match self {
            Location::Register(reg, offset) => {
                let value = regs.load(reg);
                R::reinterpret(match offset {
                    Some(offset) => value + u64::reinterpret(offset),
                    None => value,
                })
            },
            Location::Immediate(imm) => {
                let imm = u64::reinterpret(imm);
                R::reinterpret(imm)
            },
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum ExecuteError {
    #[error(transparent)]
    OutOfBounds(#[from] OutOfBounds),
}

pub trait Execute {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError>;
}

impl Execute for Nop {
    fn execute(self, _vm: &mut Machine) -> Result<(), ExecuteError> {
        let Nop {} = self;
        Ok(())
    }
}

impl Execute for Add {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Add {dest, source} = self;
        let lhs: u64 = dest.into_value(&vm.registers);
        let rhs: u64 = source.into_value(&vm.registers);

        let carry = if lhs.checked_add(rhs).is_none() {
            CF::Carry
        } else {
            CF::NoCarry
        };

        let signed_lhs = i64::from_le_bytes(lhs.to_le_bytes());
        let signed_rhs = i64::from_le_bytes(rhs.to_le_bytes());

        let overflow = if signed_lhs.checked_add(signed_rhs).is_none() {
            OF::Overflow
        } else {
            OF::NoOverflow
        };

        let result = lhs.wrapping_add(rhs);

        let zero = if result == 0 {
            ZF::Zero
        } else {
            ZF::NonZero
        };

        let sign = if (1u64 << 63) & result > 0 {
            SF::NegativeSign
        } else {
            SF::PositiveSign
        };

        vm.registers.store_dest(dest, result);
        vm.flags = Flags {carry, zero, sign, overflow};

        Ok(())
    }
}

impl Execute for Sub {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Sub {dest, source} = self;
        let lhs: u64 = dest.into_value(&vm.registers);
        let rhs: u64 = source.into_value(&vm.registers);

        let carry = if lhs.checked_sub(rhs).is_none() {
            CF::Carry
        } else {
            CF::NoCarry
        };

        let signed_lhs = i64::from_le_bytes(lhs.to_le_bytes());
        let signed_rhs = i64::from_le_bytes(rhs.to_le_bytes());

        let overflow = if signed_lhs.checked_sub(signed_rhs).is_none() {
            OF::Overflow
        } else {
            OF::NoOverflow
        };

        let result = lhs.wrapping_sub(rhs);

        let zero = if result == 0 {
            ZF::Zero
        } else {
            ZF::NonZero
        };

        let sign = if (1u64 << 63) & result > 0 {
            SF::NegativeSign
        } else {
            SF::PositiveSign
        };

        vm.registers.store_dest(dest, result);
        vm.flags = Flags {carry, zero, sign, overflow};

        Ok(())
    }
}

impl Execute for Mul {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mul {dest, source} = self;
        todo!()
    }
}

impl Execute for Mull {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mull {dest_hi, dest, source} = self;
        todo!()
    }
}

impl Execute for Mulu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mulu {dest, source} = self;
        todo!()
    }
}

impl Execute for Mullu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mullu {dest_hi, dest, source} = self;
        todo!()
    }
}

impl Execute for Div {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Div {dest, source} = self;
        todo!()
    }
}

impl Execute for Divr {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divr {dest_rem, dest, source} = self;
        todo!()
    }
}

impl Execute for Divu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divu {dest, source} = self;
        todo!()
    }
}

impl Execute for Divru {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Divru {dest_rem, dest, source} = self;
        todo!()
    }
}

impl Execute for Rem {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Rem {dest, source} = self;
        todo!()
    }
}

impl Execute for Remu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Remu {dest, source} = self;
        todo!()
    }
}

impl Execute for And {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let And {dest, source} = self;
        todo!()
    }
}

impl Execute for Or {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Or {dest, source} = self;
        todo!()
    }
}

impl Execute for Xor {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Xor {dest, source} = self;
        todo!()
    }
}

impl Execute for Test {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Test {source1, source2} = self;
        todo!()
    }
}

impl Execute for Cmp {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Cmp {source1, source2} = self;
        todo!()
    }
}

impl Execute for Mov {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mov {dest, source} = self;

        let value: u64 = source.into_value(&vm.registers);
        vm.registers.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Load1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load1 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Loadu1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu1 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Load2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load2 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Loadu2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu2 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Load4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load4 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Loadu4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu4 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Load8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load8 {dest, loc} = self;

        let addr: u64 = loc.into_value(&vm.registers);
        let value = vm.memory.read_u64(addr)?;
        vm.registers.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Loadu8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu8 {dest, loc} = self;
        todo!()
    }
}

impl Execute for Store1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store1 {loc, source} = self;
        todo!()
    }
}

impl Execute for Store2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store2 {loc, source} = self;
        todo!()
    }
}

impl Execute for Store4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store4 {loc, source} = self;
        todo!()
    }
}

impl Execute for Store8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store8 {loc, source} = self;
        todo!()
    }
}

impl Execute for Push {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Push {source} = self;

        // Decrement the stack pointer
        let sp: u64 = vm.registers.load_sp();
        let stack_top = sp - size_bytes_of::<u64>();
        vm.registers.store_sp(stack_top);

        // Store the value at the top of the stack
        let value: u64 = source.into_value(&mut vm.registers);
        vm.memory.write_u64(stack_top, value)?;

        Ok(())
    }
}

impl Execute for Pop {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Pop {dest} = self;

        // Load the top of the stack into the destination
        let stack_top: u64 = vm.registers.load_sp();
        let value = vm.memory.read_u64(stack_top)?;
        vm.registers.store_dest(dest, value);

        // Increment the stack pointer
        let sp = stack_top + size_bytes_of::<u64>();
        vm.registers.store_sp(sp);

        Ok(())
    }
}

impl Execute for Jmp {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jmp {loc} = self;
        todo!()
    }
}

impl Execute for Je {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Je {loc} = self;
        todo!()
    }
}

impl Execute for Jne {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jne {loc} = self;
        todo!()
    }
}

impl Execute for Jg {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jg {loc} = self;
        todo!()
    }
}

impl Execute for Jge {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jge {loc} = self;
        todo!()
    }
}

impl Execute for Ja {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ja {loc} = self;
        todo!()
    }
}

impl Execute for Jae {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jae {loc} = self;
        todo!()
    }
}

impl Execute for Jl {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jl {loc} = self;
        todo!()
    }
}

impl Execute for Jle {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jle {loc} = self;
        todo!()
    }
}

impl Execute for Jb {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jb {loc} = self;
        todo!()
    }
}

impl Execute for Jbe {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jbe {loc} = self;
        todo!()
    }
}

impl Execute for Jo {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jo {loc} = self;
        todo!()
    }
}

impl Execute for Jno {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jno {loc} = self;
        todo!()
    }
}

impl Execute for Jz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jz {loc} = self;
        todo!()
    }
}

impl Execute for Jnz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jnz {loc} = self;
        todo!()
    }
}

impl Execute for Js {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Js {loc} = self;
        todo!()
    }
}

impl Execute for Jns {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jns {loc} = self;
        todo!()
    }
}

impl Execute for Call {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Call {loc} = self;
        todo!()
    }
}

impl Execute for Ret {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ret {} = self;
        todo!()
    }
}
