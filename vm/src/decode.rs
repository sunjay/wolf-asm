use wolf_asm::asm::{
    InstrKind,
    layout::{Opcode, Layout, BitPattern},
};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum DecodeError {
    #[error("Invalid instruction: opcode `{0}` is not supported")]
    InvalidOpcode(u16),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instr {
    pub kind: InstrKind,
    pub args: Layout,
}

impl Instr {
    pub fn decode(instr: u64) -> Result<Self, DecodeError> {
        let opcode = Opcode::read(instr, 0);
        let (kind, opcode_offset) = InstrKind::from_opcode(opcode);
        let args = Layout::from_binary(instr, opcode_offset)
            .ok_or_else(|| DecodeError::InvalidOpcode(opcode.into_value()))?;

        Ok(Self {kind, args})
    }
}
