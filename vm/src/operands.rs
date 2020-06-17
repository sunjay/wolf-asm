use wolf_asm::asm::layout::{Reg, Imm, Offset as LayoutOffset};

use crate::reinterpret::Reinterpret;
use crate::machine::Machine;

pub type Immediate = i128;
pub type Offset = i16;

pub trait StoreDestination {
    fn store_dest<R>(&mut self, dest: Destination, value: R)
        where u64: Reinterpret<R>;
}

impl StoreDestination for Machine {
    fn store_dest<R>(&mut self, dest: Destination, value: R)
        where u64: Reinterpret<R>
    {
        match dest {
            Destination::Register(reg) => self.registers.store(reg, value),
        }
    }
}

pub trait Operand {
    fn into_value<R: Reinterpret<u64>>(self, vm: &Machine) -> R;
}

/// Represents an argument for an instruction that may be used as a source operand
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Source {
    Register(Reg),
    Immediate(Immediate),
}

impl From<Reg> for Source {
    fn from(reg: Reg) -> Self {
        Source::Register(reg)
    }
}

impl<S> From<Imm<S>> for Source {
    fn from(imm: Imm<S>) -> Self {
        Source::Immediate(imm.into_value())
    }
}

impl From<Immediate> for Source {
    fn from(imm: Immediate) -> Self {
        Source::Immediate(imm)
    }
}

impl From<u64> for Source {
    fn from(imm: u64) -> Self {
        Source::Immediate(imm as i128)
    }
}

impl From<i64> for Source {
    fn from(imm: i64) -> Self {
        Source::Immediate(imm as i128)
    }
}

impl Operand for Source {
    fn into_value<R: Reinterpret<u64>>(self, vm: &Machine) -> R {
        match self {
            Source::Register(reg) => vm.registers.load(reg),
            Source::Immediate(imm) => {
                let imm = u64::reinterpret(imm);
                R::reinterpret(imm)
            },
        }
    }
}

/// Represents an argument for an instruction that may be used as a destination operand
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Destination {
    Register(Reg),
}

impl From<Reg> for Destination {
    fn from(reg: Reg) -> Self {
        Destination::Register(reg)
    }
}

impl Operand for Destination {
    fn into_value<R: Reinterpret<u64>>(self, vm: &Machine) -> R {
        match self {
            Destination::Register(reg) => vm.registers.load(reg),
        }
    }
}

/// Represents an argument for an instruction that may be used as a location (address) operand
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Location {
    Register(Reg, Option<Offset>),
    Immediate(Immediate),
}

impl From<Reg> for Location {
    fn from(reg: Reg) -> Self {
        Location::Register(reg, None)
    }
}

impl From<(Reg, LayoutOffset)> for Location {
    fn from((reg, offset): (Reg, LayoutOffset)) -> Self {
        Location::Register(reg, Some(offset.into_value()))
    }
}

impl<S> From<Imm<S>> for Location {
    fn from(imm: Imm<S>) -> Self {
        Location::Immediate(imm.into_value())
    }
}

impl From<Immediate> for Location {
    fn from(imm: Immediate) -> Self {
        Location::Immediate(imm)
    }
}

impl Operand for Location {
    fn into_value<R: Reinterpret<u64>>(self, vm: &Machine) -> R {
        match self {
            Location::Register(reg, offset) => {
                let value = vm.registers.load(reg);
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
