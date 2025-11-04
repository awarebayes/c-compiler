use std::collections::HashMap;

use crate::ir::{IrTextRepr, nodes};

pub type BasicBlock = Vec<nodes::Ssa>;

pub fn ir_to_basic_blocks(ir: &[nodes::Ssa]) -> Vec<BasicBlock> {
    let mut blocks = vec![];
    let mut current_block = vec![];

    for node in ir {
        match node {
            nodes::Ssa::Label(_) =>
            {
                if !current_block.is_empty() {
                    blocks.push(current_block);
                    current_block = vec![];
                }
            },
            nodes::Ssa::Return { value: _ } =>  {
                current_block.push(node.clone());
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

pub fn ir_to_basic_blocks_labeled(ir: &[nodes::Ssa]) -> HashMap<String, BasicBlock> {
    let mut blocks = HashMap::new();
    let mut current_block = vec![];
    let mut prev_label = String::new();

    for node in ir {
        match node {
            nodes::Ssa::Label(l) =>
            {
                if !current_block.is_empty() {
                    blocks.insert(prev_label.clone(), current_block);
                    current_block = vec![];
                }
                prev_label = l.to_ir_string();
            },
            nodes::Ssa::Return { value: _ } =>  {
                current_block.push(node.clone());
                if !current_block.is_empty() {
                    blocks.insert(prev_label.clone(), current_block);
                    current_block = vec![];
                }
            }
            _ => (),
        }

        current_block.push(node.clone());
    }

    blocks
}

pub fn extract_phi_functions(body: &[nodes::Ssa]) -> Vec<nodes::PhiFunction> {
    body.iter().filter_map(|x| if let nodes::Ssa::Phi(phi) = x {
        Some(phi.clone())
    } else {
        None
    }).collect()
}

pub fn block_label(body: &[nodes::Ssa]) -> Option<String> {
    body.iter().find_map(|x| if let nodes::Ssa::Label(l) = x {
        Some(l.to_ir_string())
    } else {
        None
    }) 
}