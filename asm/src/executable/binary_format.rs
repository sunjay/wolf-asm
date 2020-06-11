//! A version the statements without any spans or other source-related info

use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::asm::{self, layout::InstrLayout};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    StaticData(StaticData),
    Instr(InstrLayout),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StaticData {
    StaticBytes(StaticBytes),
    StaticZero(StaticZero),
    StaticUninit(StaticUninit),
    StaticByteStr(StaticByteStr),
}

impl From<asm::StaticData> for StaticData {
    fn from(data: asm::StaticData) -> Self {
        use asm::StaticData::*;
        match data {
            StaticBytes(data) => StaticData::StaticBytes(data.into()),
            StaticZero(data) => StaticData::StaticZero(data.into()),
            StaticUninit(data) => StaticData::StaticUninit(data.into()),
            StaticByteStr(data) => StaticData::StaticByteStr(data.into()),
        }
    }
}

/// The `.b1`, `.b2`, `.b4`, or `.b8` static data directive
///
/// Note that each value is in **little-endian** byte order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StaticBytes {
    B1([u8; 1]),
    B2([u8; 2]),
    B4([u8; 4]),
    B8([u8; 8]),
}

impl From<asm::StaticBytes> for StaticBytes {
    fn from(data: asm::StaticBytes) -> Self {
        use asm::StaticBytesValue::*;
        match data.value {
            B1(data, _) => StaticBytes::B1(data),
            B2(data, _) => StaticBytes::B2(data),
            B4(data, _) => StaticBytes::B4(data),
            B8(data, _) => StaticBytes::B8(data),
        }
    }
}

/// The `.zero` directive
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticZero {
    pub nbytes: Size,
}

impl From<asm::StaticZero> for StaticZero {
    fn from(data: asm::StaticZero) -> Self {
        let asm::StaticZero {nbytes, span: _} = data;
        Self {
            nbytes: nbytes.value,
        }
    }
}

/// The `.uninit` directive
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticUninit {
    pub nbytes: Size,
}

impl From<asm::StaticUninit> for StaticUninit {
    fn from(data: asm::StaticUninit) -> Self {
        let asm::StaticUninit {nbytes, span: _} = data;
        Self {
            nbytes: nbytes.value,
        }
    }
}

/// The `.bytes` directive
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticByteStr {
    pub bytes: Bytes,
}

impl From<asm::StaticByteStr> for StaticByteStr {
    fn from(data: asm::StaticByteStr) -> Self {
        let asm::StaticByteStr {bytes, span: _} = data;
        Self {
            bytes: bytes.value,
        }
    }
}

pub type Bytes = Arc<[u8]>;
pub type Size = u64;
