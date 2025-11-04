use crate::ir::nodes;

pub type BasicBlock = Vec<nodes::Ssa>;

pub fn ir_to_basic_blocks(ir: &[nodes::Ssa]) -> Vec<BasicBlock> {
    let mut blocks = vec![];
    let mut current_block = vec![];

    for node in ir {
        match node {
            nodes::Ssa::Label(_) |
            nodes::Ssa::Return { value: _ } => {
                if !current_block.is_empty() {
                    blocks.push(current_block);
                    current_block = vec![];
                }
            }
            _ => (),
        }

        current_block.push(node.clone());
    }

    blocks
}