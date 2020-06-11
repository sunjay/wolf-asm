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

fn write_static_data<W: Write>(data: &asm::StaticData, mut writer: W) -> io::Result<()> {
    use asm::StaticData::*;
    match data {
        StaticBytes(bytes) => {
            // starting at 1 because 1 is more visible in a binary file than 0
            write_tag(1, asm::STATIC_DATA_TAG_BYTES, &mut writer)?;
            write_static_bytes(bytes, writer)
        },
        StaticZero(zero) => {
            write_tag(2, asm::STATIC_DATA_TAG_BYTES, &mut writer)?;
            todo!()
        },
        StaticUninit(uninit) => {
            write_tag(3, asm::STATIC_DATA_TAG_BYTES, &mut writer)?;
            todo!()
        },
        StaticByteStr(byte_str) => {
            write_tag(4, asm::STATIC_DATA_TAG_BYTES, &mut writer)?;
            todo!()
        },
    }
}

fn write_static_bytes<W: Write>(bytes: &asm::StaticBytes, mut writer: W) -> io::Result<()> {
    todo!()
}

// Writes a tag so we can tell which variant of an enum is in the executable
fn write_tag<W: Write>(tag: u8, expected_bytes: usize, mut writer: W) -> io::Result<()> {
    let tag = tag.to_le_bytes();
    debug_assert!(tag.len() == expected_bytes,
        "bug: expected {} bytes for tag, got {} bytes", expected_bytes, tag.len());
    writer.write_all(&tag)
}

fn write_instr<W: Write>(instr: &InstrLayout, mut writer: W) -> io::Result<()> {
    let instr_binary = instr.to_binary();
    let bytes = instr_binary.to_le_bytes();

    writer.write_all(&bytes)
}
