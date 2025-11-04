pub mod nodes;
mod ssa;
mod text;

pub use ssa::build_ssa;
pub use text::{IrTextRepr, into_text};
