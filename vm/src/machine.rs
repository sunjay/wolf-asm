use wolf_asm::asm::{
    InstrKind,
    layout::Layout,
};
use thiserror::Error;

use crate::{
    memory::{Memory, OutOfBounds},
    registers::Registers,
    flags::Flags,
    decode::{Instr, DecodeError},
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
        self.program_counter += instr.kind.size_bytes();

        self.execute(instr)?;

        if self.program_counter == QUIT_ADDR {
            Ok(ProgramStatus::Quit)
        } else {
            Ok(ProgramStatus::Continue)
        }
    }

    pub fn execute(&mut self, instr: Instr) -> Result<(), ExecutionError> {
        let Instr {kind, args} = instr;

        match kind {
            InstrKind::Nop => self.execute_nop(args),
            InstrKind::Add => self.execute_add(args),
            InstrKind::Sub => self.execute_sub(args),
            InstrKind::Mul => self.execute_mul(args),
            InstrKind::Mull => self.execute_mull(args),
            InstrKind::Mulu => self.execute_mulu(args),
            InstrKind::Mullu => self.execute_mullu(args),
            InstrKind::Div => self.execute_div(args),
            InstrKind::Divr => self.execute_divr(args),
            InstrKind::Divu => self.execute_divu(args),
            InstrKind::Divru => self.execute_divru(args),
            InstrKind::Rem => self.execute_rem(args),
            InstrKind::Remu => self.execute_remu(args),
            InstrKind::And => self.execute_and(args),
            InstrKind::Or => self.execute_or(args),
            InstrKind::Xor => self.execute_xor(args),
            InstrKind::Test => self.execute_test(args),
            InstrKind::Cmp => self.execute_cmp(args),
            InstrKind::Mov => self.execute_mov(args),
            InstrKind::Load1 => self.execute_load1(args),
            InstrKind::Loadu1 => self.execute_loadu1(args),
            InstrKind::Load2 => self.execute_load2(args),
            InstrKind::Loadu2 => self.execute_loadu2(args),
            InstrKind::Load4 => self.execute_load4(args),
            InstrKind::Loadu4 => self.execute_loadu4(args),
            InstrKind::Load8 => self.execute_load8(args),
            InstrKind::Loadu8 => self.execute_loadu8(args),
            InstrKind::Store1 => self.execute_store1(args),
            InstrKind::Store2 => self.execute_store2(args),
            InstrKind::Store4 => self.execute_store4(args),
            InstrKind::Store8 => self.execute_store8(args),
            InstrKind::Push => self.execute_push(args),
            InstrKind::Pop => self.execute_pop(args),
            InstrKind::Jmp => self.execute_jmp(args),
            InstrKind::Je => self.execute_je(args),
            InstrKind::Jne => self.execute_jne(args),
            InstrKind::Jg => self.execute_jg(args),
            InstrKind::Jge => self.execute_jge(args),
            InstrKind::Ja => self.execute_ja(args),
            InstrKind::Jae => self.execute_jae(args),
            InstrKind::Jl => self.execute_jl(args),
            InstrKind::Jle => self.execute_jle(args),
            InstrKind::Jb => self.execute_jb(args),
            InstrKind::Jbe => self.execute_jbe(args),
            InstrKind::Jo => self.execute_jo(args),
            InstrKind::Jno => self.execute_jno(args),
            InstrKind::Jz => self.execute_jz(args),
            InstrKind::Jnz => self.execute_jnz(args),
            InstrKind::Js => self.execute_js(args),
            InstrKind::Jns => self.execute_jns(args),
            InstrKind::Call => self.execute_call(args),
            InstrKind::Ret => self.execute_ret(args),
        }
    }

    fn execute_nop(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_add(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_sub(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_mul(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_mull(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_mulu(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_mullu(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_div(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_divr(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_divu(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_divru(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_rem(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_remu(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_and(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_or(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_xor(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_test(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_cmp(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_mov(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_load1(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_loadu1(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_load2(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_loadu2(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_load4(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_loadu4(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_load8(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_loadu8(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_store1(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_store2(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_store4(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_store8(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_push(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_pop(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jmp(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_je(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jne(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jg(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jge(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_ja(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jae(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jl(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jle(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jb(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jbe(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jo(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jno(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jz(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jnz(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_js(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_jns(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_call(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    fn execute_ret(&mut self, args: Layout) -> Result<(), ExecutionError> {
        todo!();
    }

    pub fn push_quit_addr(&mut self) -> Result<(), ExecutionError> {
        self.push_immediate(QUIT_ADDR as i128)
    }

    pub fn push_immediate(&mut self, imm: i128) -> Result<(), ExecutionError> {
        todo!()
    }
}
