use crate::semantic_analysis::{SymbolKind, SymbolType};

#[derive(Debug, Clone, Copy)]
pub enum StorageClass {
    Auto,
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Word,
    Long,
    Short,
    Byte,
}

impl Width {
    pub fn from_type(symbol: &SymbolType) -> Self {
        match symbol {
            SymbolType::Int => Self::Word,
            SymbolType::Char => Self::Byte,
            SymbolType::Pointer(_) => Self::Long,
        }
    }

    pub fn to_bytes(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Short => 2,
            Self::Word => 4,
            Self::Long => 8,
        }
    }
}
