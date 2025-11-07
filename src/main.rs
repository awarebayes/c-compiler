pub mod asmgen;
pub mod common;
pub mod ir;
pub mod opt;
pub mod parsing;
pub mod semantic_analysis;

use parsing::{Parser, TreeSitterParser};
use semantic_analysis::SymbolTable;
use clap::Parser as ClapParser;
use std::path::PathBuf;

use std::{error::Error, fs};

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input C source file
    #[arg(short, long)]
    input: PathBuf,

    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Dump abstract syntax tree
    #[arg(long)]
    dump_ast: bool,

    /// Dump IR during compilation
    #[arg(long)]
    dump_ir: bool,

    /// Do a graphviz visualization of functions
    #[arg(long)]
    graphviz: bool,

    /// Enable verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Type of output to generate
    #[arg(long, value_enum, default_value_t = EmitType::Asm)]
    emit: EmitType,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq)]
enum EmitType {
    Ir,
    Asm,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let source_path = args.input;

    let source_code = fs::read_to_string(source_path).expect("Could not read input file");
    let parser = TreeSitterParser::default();

    let unit = parser.parse(&source_code);
    let symbol_table = SymbolTable::from_translation_unit(&unit);

    if args.dump_ast {
        println!("--- AST ---");
        println!("Parsed tree: {:#?}", unit);
    }


    let ssa = ir::build_ssa(&unit, symbol_table.clone());

    if args.dump_ir {
        let ssa_text = ir::into_text(&ssa);
        println!("--- IR ---");
        println!("{}", ssa_text);
    }

    let opt_ssa = opt::o1(&ssa);

    if args.graphviz {
        ir::graphviz_unit(&opt_ssa, "./graphviz");
    }

    if args.dump_ir {
        println!("--- IR OPT ---");
        let ssa_text = ir::into_text(&opt_ssa);
        println!("{}", ssa_text);
    }

    if args.emit == EmitType::Ir {
        let ssa_text = ir::into_text(&opt_ssa);
        println!("{}", ssa_text);
        if let Some(out_path) = args.output {
            fs::write(out_path, ssa_text).unwrap();
        } else {
            println!("{}", ssa_text);
        }
        return Ok(())
    }

    let asm = asmgen::convert_unit_to_asm(&opt_ssa);
    let asm_text = asmgen::asm_into_text(&asm);

    if let Some(out_path) = args.output {
        fs::write(out_path, asm_text).unwrap();
    } else {
        println!("{}", asm_text);
    }

    Ok(())
}
