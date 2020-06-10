mod binary_format;

use std::io::{self, Write};
use std::convert::TryInto;

use crate::asm;
use crate::label_offsets::LabelOffsets;
use crate::diagnostics::Diagnostics;

use binary_format::{ExecutableHeader, Stmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Executable {
    header: ExecutableHeader,
    code_section: Vec<Stmt>,
    static_section: Vec<Stmt>,
}

impl Executable {
    pub fn layout_executable(prog: asm::Program, diag: &Diagnostics, labels: &LabelOffsets) -> Self {
        let asm::Program {code_section, static_section} = prog;

        let code_section_bytes = code_section.as_ref()
            .map(|section| section.stmts.iter().map(|stmt| stmt.size_bytes()).sum())
            .unwrap_or(0);
        let static_section_bytes = static_section.as_ref()
            .map(|section| section.stmts.iter().map(|stmt| stmt.size_bytes()).sum())
            .unwrap_or(0);

        let header = ExecutableHeader {
            code_section_bytes: code_section_bytes.try_into()
                .expect("bug: more than `u32::max_value()` bytes are not supported yet"),
            static_section_bytes: static_section_bytes.try_into()
                .expect("bug: more than `u32::max_value()` bytes are not supported yet"),
        };

        let code_section = code_section.map(|section| layout_section(section, diag, labels)).unwrap_or_default();
        let static_section = static_section.map(|section| layout_section(section, diag, labels)).unwrap_or_default();

        Self {header, code_section, static_section}
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.header.write(&mut writer)?;

        for stmt in &self.code_section {
            stmt.write(&mut writer)?;
        }
        for stmt in &self.static_section {
            stmt.write(&mut writer)?;
        }

        Ok(())
    }
}

fn layout_section(section: asm::Section, diag: &Diagnostics, labels: &LabelOffsets) -> Vec<Stmt> {
    let asm::Section {section_header_span: _, stmts} = section;
    stmts.into_iter().map(|stmt| match stmt.kind {
        asm::StmtKind::StaticData(data) => Stmt::StaticData(data),
        asm::StmtKind::Instr(instr) => Stmt::Instr(instr.layout(diag, labels)),
    }).collect()
}
