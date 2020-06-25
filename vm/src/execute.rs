use std::io;

use thiserror::Error;

use crate::reinterpret::Reinterpret;
use crate::machine::Machine;
use crate::memory::OutOfBounds;
use crate::flags::{Flags, CF, ZF, SF, OF};
use crate::operands::{StoreDestination, Operand};
use crate::decode::*;

/// The address used to indicate that the program should quit
pub const QUIT_ADDR: u64 = u64::MAX;
/// The address used for stdout
pub const STDOUT_ADDR: u64 = 0xffff_000c;
/// The address used for stdin
pub const STDIN_ADDR: u64 = 0xffff_0004;
/// The byte used to indicate EOF
pub const EOF_BYTE: u8 = b'\0';

fn size_bytes_of<T>() -> u64 {
    std::mem::size_of::<T>() as u64
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error(transparent)]
    OutOfBounds(#[from] OutOfBounds),
    #[error("Divided a number by zero")]
    DivideByZero,
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
        let lhs: u64 = dest.into_value(vm);
        let rhs: u64 = source.into_value(vm);

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

        vm.store_dest(dest, result);
        vm.flags = Flags {carry, zero, sign, overflow};

        Ok(())
    }
}

impl Execute for Sub {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Sub {dest, source} = self;
        let lhs: u64 = dest.into_value(vm);
        let rhs: u64 = source.into_value(vm);

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

        vm.store_dest(dest, result);
        vm.flags = Flags {carry, zero, sign, overflow};

        Ok(())
    }
}

impl Execute for Mul {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mul {dest, source} = self;

        // mul loads values as signed and then widens them so we can detect
        // overflow
        let lhs: i64 = dest.into_value(vm);
        let rhs: i64 = source.into_value(vm);
        let lhs = i128::reinterpret(lhs);
        let rhs = i128::reinterpret(rhs);

        let (result, overflow) = lhs.overflowing_mul(rhs);

        // Check if the higher bits are set (i.e. result cannot fit in i64)
        let overflow = overflow || result != i64::reinterpret(result) as i128;

        vm.store_dest(dest, result);
        // Update carry and overflow only, all other flags are undefined
        vm.flags = Flags {
            carry: if overflow { CF::Carry } else { CF::NoCarry },
            overflow: if overflow { OF::Overflow } else { OF::NoOverflow },
            ..vm.flags
        };

        Ok(())
    }
}

impl Execute for Mull {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mull {dest_hi, dest, source} = self;

        // mull loads values as signed and then widens them so we can detect
        // overflow
        let lhs: i64 = dest.into_value(vm);
        let rhs: i64 = source.into_value(vm);
        let lhs = i128::reinterpret(lhs);
        let rhs = i128::reinterpret(rhs);

        let (result, overflow) = lhs.overflowing_mul(rhs);
        // Cast to u128 so we can freely manipulate the bits (`>>` has different
        // behaviour for i128)
        let result = u128::reinterpret(result);

        vm.store_dest(dest, result);
        vm.store_dest(dest_hi, result >> 64);
        // Update carry and overflow only, all other flags are undefined
        vm.flags = Flags {
            carry: if overflow { CF::Carry } else { CF::NoCarry },
            overflow: if overflow { OF::Overflow } else { OF::NoOverflow },
            ..vm.flags
        };

        Ok(())
    }
}

impl Execute for Mulu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mulu {dest, source} = self;

        // mulu loads values as unsigned and then widens them so we can detect
        // overflow
        let lhs: u64 = dest.into_value(vm);
        let rhs: u64 = source.into_value(vm);
        let lhs = u128::reinterpret(lhs);
        let rhs = u128::reinterpret(rhs);

        let (result, overflow) = lhs.overflowing_mul(rhs);

        // Check if the higher bits are set (i.e. result cannot fit in u64)
        let overflow = overflow || result != u64::reinterpret(result) as u128;

        vm.store_dest(dest, result);
        // Update carry and overflow only, all other flags are undefined
        vm.flags = Flags {
            carry: if overflow { CF::Carry } else { CF::NoCarry },
            overflow: if overflow { OF::Overflow } else { OF::NoOverflow },
            ..vm.flags
        };

        Ok(())
    }
}

impl Execute for Mullu {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mullu {dest_hi, dest, source} = self;

        // mullu loads values as unsigned and then widens them so we can detect
        // overflow
        let lhs: u64 = dest.into_value(vm);
        let rhs: u64 = source.into_value(vm);
        let lhs = u128::reinterpret(lhs);
        let rhs = u128::reinterpret(rhs);

        let (result, overflow) = lhs.overflowing_mul(rhs);

        vm.store_dest(dest, result);
        vm.store_dest(dest_hi, result >> 64);
        // Update carry and overflow only, all other flags are undefined
        vm.flags = Flags {
            carry: if overflow { CF::Carry } else { CF::NoCarry },
            overflow: if overflow { OF::Overflow } else { OF::NoOverflow },
            ..vm.flags
        };

        Ok(())
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
        let lhs: u64 = dest.into_value(vm);
        let rhs: u64 = source.into_value(vm);

        let quotient = lhs.checked_div_euclid(rhs);
        let remainder = lhs.checked_rem_euclid(rhs);
        let (quotient, remainder) = match (quotient, remainder) {
            (Some(quotient), Some(remainder)) => (quotient, remainder),
            (Some(_), None) |
            (None, Some(_)) |
            (None, None) => return Err(ExecuteError::DivideByZero),
        };

        vm.store_dest(dest, quotient);
        vm.store_dest(dest_rem, remainder);

        Ok(())
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

impl Execute for Not {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Not {dest} = self;
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
        let lhs: u64 = source1.into_value(vm);
        let rhs: u64 = source2.into_value(vm);

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

        vm.flags = Flags {carry, zero, sign, overflow};

        Ok(())
    }
}

impl Execute for Mov {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Mov {dest, source} = self;

        let value: u64 = source.into_value(vm);
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Load1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load1 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // load1 loads only 1 byte
        let value = if addr == STDIN_ADDR {
            u8::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.get(addr)?
        };
        // load (unlike loadu) must sign-extend (hence i8)
        let value = i8::reinterpret(value);
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Loadu1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu1 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // loadu1 loads only 1 byte
        let value = if addr == STDIN_ADDR {
            u8::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.get(addr)?
        };
        // loadu (unlike load) must NOT sign-extend (hence u8 is fine)
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Load2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load2 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // load2 loads 2 bytes
        let value = if addr == STDIN_ADDR {
            u16::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.read_u16(addr)?
        };
        // load (unlike loadu) must sign-extend (hence i16)
        let value = i16::reinterpret(value);
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Loadu2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu2 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // load2 loads 2 bytes
        let value = if addr == STDIN_ADDR {
            u16::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.read_u16(addr)?
        };
        // loadu (unlike load) must NOT sign-extend (hence u16 is fine)
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Load4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Load4 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // load4 loads 4 bytes
        let value = if addr == STDIN_ADDR {
            u32::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.read_u32(addr)?
        };
        // load (unlike loadu) must sign-extend (hence i32)
        let value = i32::reinterpret(value);
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Loadu4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Loadu4 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        // load4 loads 4 bytes
        let value = if addr == STDIN_ADDR {
            u32::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            vm.memory.read_u32(addr)?
        };
        // loadu (unlike load) must NOT sign-extend (hence u32 is fine)
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Load8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        // Note: load8 and loadu8 have the same behaviour
        let Load8 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        let value = if addr == STDIN_ADDR {
            u64::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            // Since the value is already 8 bytes, we don't need to worry about
            // sign-extension
            vm.memory.read_u64(addr)?
        };
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Loadu8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        // Note: load8 and loadu8 have the same behaviour
        let Loadu8 {dest, loc} = self;

        let addr: u64 = loc.into_value(vm);
        let value = if addr == STDIN_ADDR {
            u64::reinterpret(vm.io.read_byte()?.unwrap_or(EOF_BYTE))
        } else {
            // Since the value is already 8 bytes, we don't need to worry about
            // zero-extension
            vm.memory.read_u64(addr)?
        };
        vm.store_dest(dest, value);

        Ok(())
    }
}

impl Execute for Store1 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store1 {loc, source} = self;
        let addr: u64 = loc.into_value(vm);

        let value: u8 = source.into_value(vm);

        if addr == STDOUT_ADDR {
            vm.io.write_bytes(u32::reinterpret(value))?;
        } else {
            vm.memory.set(addr, value)?;
        }

        Ok(())
    }
}

impl Execute for Store2 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store2 {loc, source} = self;
        let addr: u64 = loc.into_value(vm);

        let value: u16 = source.into_value(vm);

        if addr == STDOUT_ADDR {
            vm.io.write_bytes(u32::reinterpret(value))?;
        } else {
            vm.memory.write_u16(addr, value)?;
        }

        Ok(())
    }
}

impl Execute for Store4 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store4 {loc, source} = self;
        let addr: u64 = loc.into_value(vm);

        let value: u32 = source.into_value(vm);

        if addr == STDOUT_ADDR {
            vm.io.write_bytes(u32::reinterpret(value))?;
        } else {
            vm.memory.write_u32(addr, value)?;
        }

        Ok(())
    }
}

impl Execute for Store8 {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Store8 {loc, source} = self;
        let addr: u64 = loc.into_value(vm);

        let value: u64 = source.into_value(vm);

        if addr == STDOUT_ADDR {
            vm.io.write_bytes(u32::reinterpret(value))?;
        } else {
            vm.memory.write_u64(addr, value)?;
        }

        Ok(())
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
        let value: u64 = source.into_value(vm);
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
        vm.store_dest(dest, value);

        // Increment the stack pointer
        let sp = stack_top + size_bytes_of::<u64>();
        vm.registers.store_sp(sp);

        Ok(())
    }
}

impl Execute for Jmp {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jmp {loc} = self;
        let addr: u64 = loc.into_value(vm);
        vm.program_counter = addr;
        Ok(())
    }
}

impl Execute for Je {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Je {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Equal if ZF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Equal
        if vm.flags.zero == ZF::Zero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jne {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jne {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Not equal if ZF = 0
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Not_Equal
        if vm.flags.zero == ZF::NonZero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jg {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jg {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Greater
        let flags = &vm.flags;
        match (flags.sign, flags.overflow, flags.zero) {
            // Greater if SF = OF and ZF = 0
            (SF::NegativeSign, OF::Overflow, ZF::NonZero) |
            (SF::PositiveSign, OF::NoOverflow, ZF::NonZero) => {
                vm.program_counter = addr;
            },

            // Not greater than
            _ => {},
        }

        Ok(())
    }
}

impl Execute for Jge {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jge {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Greater_or_Equal
        let flags = &vm.flags;
        match (flags.sign, flags.overflow, flags.zero) {
            // Greater than or equal if SF = OF or ZF = 1
            (_, _, ZF::Zero) |
            (SF::NegativeSign, OF::Overflow, _) |
            (SF::PositiveSign, OF::NoOverflow, _) => {
                vm.program_counter = addr;
            },

            // Not greater than or equal
            _ => {},
        }

        Ok(())
    }
}

impl Execute for Ja {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ja {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Above if CF = 0 and ZF = 0
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Above_(unsigned_comparison)
        let flags = &vm.flags;
        if flags.carry == CF::NoCarry && flags.zero == ZF::NonZero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jae {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jae {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Above or equal if CF = 0 or ZF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Above_or_Equal_(unsigned_comparison)
        let flags = &vm.flags;
        if flags.carry == CF::NoCarry || flags.zero == ZF::Zero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jl {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jl {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Lesser
        let flags = &vm.flags;
        match (flags.sign, flags.overflow) {
            // Lesser if SF != OF
            (SF::NegativeSign, OF::NoOverflow) |
            (SF::PositiveSign, OF::Overflow) => {
                vm.program_counter = addr;
            },

            // Not greater than
            _ => {},
        }

        Ok(())
    }
}

impl Execute for Jle {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jle {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Less_or_Equal
        let flags = &vm.flags;
        match (flags.sign, flags.overflow, flags.zero) {
            // Less than or equal if SF != OF or ZF = 1
            (_, _, ZF::Zero) |
            (SF::NegativeSign, OF::NoOverflow, _) |
            (SF::PositiveSign, OF::Overflow, _) => {
                vm.program_counter = addr;
            },

            // Not less than or equal
            _ => {},
        }

        Ok(())
    }
}

impl Execute for Jb {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jb {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Below if CF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Below_(unsigned_comparison)
        if vm.flags.carry == CF::Carry {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jbe {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jbe {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Below or equal if CF = 1 or ZF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Below_or_Equal_(unsigned_comparison)
        let flags = &vm.flags;
        if flags.carry == CF::Carry || flags.zero == ZF::Zero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jo {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jo {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Overflow if OF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Overflow
        if vm.flags.overflow == OF::Overflow {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jno {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jno {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // No overflow if OF = 0
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Not_Overflow
        if vm.flags.overflow == OF::NoOverflow {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jz {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Zero if ZF = 1
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Zero
        if vm.flags.zero == ZF::Zero {
            vm.program_counter = addr;
        }

        Ok(())
    }
}

impl Execute for Jnz {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Jnz {loc} = self;
        let addr: u64 = loc.into_value(vm);

        // Not zero if ZF = 0
        // See: https://en.wikibooks.org/wiki/X86_Assembly/Control_Flow#Jump_if_Not_Zero
        if vm.flags.zero == ZF::NonZero {
            vm.program_counter = addr;
        }

        Ok(())
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

        // Decrement the stack pointer
        let sp: u64 = vm.registers.load_sp();
        let stack_top = sp - size_bytes_of::<u64>();
        vm.registers.store_sp(stack_top);

        // Store the program counter at the top of the stack
        vm.memory.write_u64(stack_top, vm.program_counter)?;

        // Jump to the given location
        let addr: u64 = loc.into_value(vm);
        vm.program_counter = addr;

        Ok(())
    }
}

impl Execute for Ret {
    fn execute(self, vm: &mut Machine) -> Result<(), ExecuteError> {
        let Ret {} = self;

        // Load the top of the stack into the program counter
        let stack_top: u64 = vm.registers.load_sp();
        let value = vm.memory.read_u64(stack_top)?;
        vm.program_counter = value;

        // Increment the stack pointer
        let sp = stack_top + size_bytes_of::<u64>();
        vm.registers.store_sp(sp);

        Ok(())
    }
}
