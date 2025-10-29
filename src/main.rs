pub mod asmgen;
pub mod common;
pub mod ir;
pub mod parsing;
pub mod semantic_analysis;

use parsing::{Parser, TreeSitterParser};
use semantic_analysis::SymbolTable;

use std::{error::Error, fs};

use crate::parsing::ast;

fn main() -> Result<(), Box<dyn Error>> {
    let source_path = "input/hello_world.c";
    let source_code = fs::read_to_string(source_path)?;
    let parser = TreeSitterParser::default();

    let unit = parser.parse(&source_code);
    let symbol_table = SymbolTable::from_translation_unit(&unit);

    println!("Parsed tree: {:#?}", unit);
    println!("Parsed symbol table: {:#?}", &symbol_table);

    let tac = ir::build_tac(&unit, symbol_table.clone());
    let tac_text = ir::into_text(&tac);

    println!("--- IR ---");
    println!("{}", tac_text);

    let asm = asmgen::convert_unit_to_asm(&tac);
    let asm_text = asmgen::asm_into_text(&asm);

    println!("--- ASM ---");
    println!("{}", asm_text);

    let out_path = "input/codegen.asm";
    fs::write(out_path, asm_text).unwrap();

    Ok(())
}
