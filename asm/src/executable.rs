use crate::label_offsets::LabelOffsets;
use crate::asm::{self, layout::InstrLayout};
use crate::diagnostics::Diagnostics;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    StaticData(asm::StaticData),
    Instr(InstrLayout),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Executable {
    code_section: Vec<Stmt>,
    static_section: Vec<Stmt>,
}

impl Executable {
    pub fn layout_executable(prog: asm::Program, diag: &Diagnostics, labels: &LabelOffsets) -> Self {
        let asm::Program {code_section, static_section} = prog;

        let code_section = code_section.map(|section| layout_section(section, diag, labels)).unwrap_or_default();
        let static_section = static_section.map(|section| layout_section(section, diag, labels)).unwrap_or_default();

        Self {code_section, static_section}
    }
}

fn layout_section(section: asm::Section, diag: &Diagnostics, labels: &LabelOffsets) -> Vec<Stmt> {
    let asm::Section {section_header_span: _, stmts} = section;
    stmts.into_iter().map(|stmt| match stmt.kind {
        asm::StmtKind::StaticData(data) => Stmt::StaticData(data),
        asm::StmtKind::Instr(instr) => Stmt::Instr(instr.layout(diag, labels)),
    }).collect()
}
