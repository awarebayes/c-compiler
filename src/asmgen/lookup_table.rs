use std::collections::HashMap;

use crate::common::Width;
use crate::ir::nodes::{self, AddressConstant};

const STACK_ALIGN: usize = 16;

#[derive(Debug, Clone)]
pub enum SymbolAddress {
    VariableOffset(usize),
    StringLiteral(usize),
    SourceFunction(String),
}

impl SymbolAddress {
    pub fn offset(&self) -> i64 {
        match self {
            Self::VariableOffset(vo) => *vo as i64,
            Self::StringLiteral(_) => panic!(),
            Self::SourceFunction(_) => panic!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub address: SymbolAddress,
    pub width: Width,
}

#[derive(Debug, Clone)]
pub struct SymbolLookup {
    lookup: HashMap<nodes::Address, SymbolInfo>
}

impl SymbolLookup {

    pub fn get(&self, key: &nodes::Address) -> Option<&SymbolInfo> {
        self.lookup.get(key)
    }

    pub fn stack_size(&self) -> usize {
        self.lookup.values().map(|x| x.width.to_bytes()).sum::<usize>().next_multiple_of(STACK_ALIGN)
    }

    pub fn from_fn_body(ir: &[nodes::Ssa]) -> Self {
        let mut offset = 0;
        let mut lookup = HashMap::new();
        for n in ir {

            let (addr, width) = match n {
                nodes::Ssa::Assignment {
                    dest,
                    source: _,
                    width,
                } => {
                    (dest, width)
                },
                nodes::Ssa::Quadriplet(quad) => {
                    (&quad.dest, &quad.width)
                },
                nodes::Ssa::Phi(phi ) => {
                    (&phi.dest, &phi.width)
                },

                nodes::Ssa::Call { dest, func: _, num_params: _ } => {
                    if let Some((dest, width)) = dest {
                        (dest, width)
                    } else {
                        continue;
                    }
                }

                _ => continue,
            };


            if !lookup.contains_key(addr) {
                lookup.insert(
                    addr.clone(),
                    SymbolInfo {
                        address: SymbolAddress::VariableOffset(offset),
                        width: *width,
                    },
                );
                offset += width.to_bytes();
            }
        }
        SymbolLookup { lookup }
    }

    pub fn global_from_unit(toplevels: &[nodes::ToplevelItem]) -> Self {
        let mut lookup = HashMap::new();
        let mut string_liter_count: usize = 0;

        for toplevel in toplevels { 
            match toplevel {
                nodes::ToplevelItem::Declaration(decl) => {
                    match decl {
                        nodes::ToplevelDeclaration::Function { storage_class: _, name, return_width: _, parameters: _ } => {
                            let addr = nodes::Address::source_count(name.clone(), 0);
                            let info = SymbolInfo {
                                address: SymbolAddress::SourceFunction(name.clone()),
                                width: Width::Long,
                            };
                            lookup.insert(addr, info);
                        }
                    }
                },
                nodes::ToplevelItem::Function(func) => {
                    let addr = nodes::Address::source_count(func.name.clone(), 0);
                    let info = SymbolInfo {
                        address: SymbolAddress::SourceFunction(func.name.clone()),
                        width: Width::Long,
                    };
                    lookup.insert(addr, info);

                    for b in func.body.iter() {
                        match &b {
                            nodes::Ssa::Assignment { dest: _, source, width: _ } => {
                                match &source {
                                    nodes::Address::CompilerTemp(_) |
                                    nodes::Address::Source(_) |
                                    nodes::Address::Constant(nodes::AddressConstant::Numeric(_)) => {},
                                    nodes::Address::Constant(nodes::AddressConstant::StringLiteral(_)) => {
                                        let info = SymbolInfo {
                                            address: SymbolAddress::StringLiteral(string_liter_count),
                                            width: Width::Long,
                                        };
                                        lookup.insert(source.clone(), info);
                                        string_liter_count += 1;
                                    }
                                }
                            },
                            _ => continue,
                        }
                    }
                },

            }
        }
        SymbolLookup { lookup }
    }

    pub fn extend_with_global(self, global: Self) -> Self {
        let mut joint_lookup = self.lookup;
        joint_lookup.extend(global.lookup);
        SymbolLookup { lookup:  joint_lookup }
    }

    pub fn string_literals_iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.lookup.iter().filter_map(|(addr, symbol_info)|  {
            if let  SymbolAddress::StringLiteral(counter) = symbol_info.address {
                match addr {
                    nodes::Address::Constant(AddressConstant::StringLiteral(sl)) => Some((counter, sl.as_str())),
                    _ => panic!("Not a string literal")
                }
            } else {
                None
            }
        })
    }

}

