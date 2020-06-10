use std::io::{self, Write};

use crate::asm::{self, layout::InstrLayout};

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableHeader {
    /// The total number of bytes in the code section
    pub code_section_bytes: u32,
    /// The total number of bytes in the static section
    pub static_section_bytes: u32,
}

impl ExecutableHeader {
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let Self {code_section_bytes, static_section_bytes} = *self;

        let code_section_bytes = code_section_bytes.to_le_bytes();
        let static_section_bytes = static_section_bytes.to_le_bytes();

        writer.write_all(&code_section_bytes)?;
        writer.write_all(&static_section_bytes)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    StaticData(asm::StaticData),
    Instr(InstrLayout),
}

impl Stmt {
    pub fn write<W: Write>(&self, writer: W) -> io::Result<()> {
        use Stmt::*;
        match self {
            StaticData(data) => write_static_data(data, writer),
            Instr(instr) => todo!(),
        }
    }
}

fn write_static_data<W: Write>(data: &asm::StaticData, writer: W) -> io::Result<()> {
    use asm::StaticData::*;
    match data {
        StaticBytes(bytes) => todo!(),
        StaticZero(zero) => todo!(),
        StaticUninit(uninit) => todo!(),
        StaticByteStr(byte_str) => todo!(),
    }
}
