pub mod nodes;
mod ssa;
mod text;
mod blocks;
mod graphviz;

pub use ssa::build_ssa;
pub use text::{IrTextRepr, into_text};
pub use blocks::{BasicBlock, ir_to_basic_blocks};
pub use graphviz::graphviz_unit;