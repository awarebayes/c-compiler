mod blocks;
mod graphviz;
pub mod nodes;
mod ssa;
mod text;

pub use blocks::{
    BasicBlock, block_label, extract_phi_functions, ir_to_basic_blocks, ir_to_basic_blocks_labeled,
};
pub use graphviz::graphviz_unit;
pub use ssa::build_ssa;
pub use text::{IrTextRepr, into_text};
