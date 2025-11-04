pub mod nodes;
mod ssa;
mod text;
mod blocks;
mod graphviz;

pub use ssa::build_ssa;
pub use text::{IrTextRepr, into_text};
pub use blocks::{BasicBlock, ir_to_basic_blocks, ir_to_basic_blocks_labeled, extract_phi_functions, block_label};
pub use graphviz::graphviz_unit;