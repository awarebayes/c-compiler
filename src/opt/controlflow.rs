use std::collections::{HashMap};

use crate::ir::{IrTextRepr, ir_to_basic_blocks, nodes};


pub struct BasicBlock {
    pub label: String,
    pub ir: Vec<nodes::Ssa>,
    pub used_variables: HashMap<nodes::Address, Vec<usize>>, // vars that are present in other blocks phi functions
}

pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
    block_index: HashMap<String, usize>,
    control_adjacency: HashMap<usize, Vec<usize>>,
}

impl ControlFlowGraph {
    pub fn new(body: &[nodes::Ssa]) -> Self {
        let blocks = ir_to_basic_blocks(body);


        let mut out_blocks: Vec<BasicBlock> = blocks.iter().map(|b| {
            BasicBlock {
            label: {
                let label = &b[0];
                if let nodes::Ssa::Label(lab) = label {
                    lab.to_ir_string()
                } else {
                    panic!("Label not found");
                }
            },
            ir: b.clone(),
            used_variables: HashMap::new()
        }}
    ).collect();

        let mut control_adjacency: HashMap<usize, Vec<usize>> = HashMap::new();

        let mut block_index = HashMap::new();
        for (block_idx, block) in blocks.iter().enumerate() {
            let label = &block[0];
            let mut lab_ir = String::new();
            if let nodes::Ssa::Label(lab) = label {
                lab_ir = lab.to_ir_string();
            } else {
                panic!("Label not found");
            }

            block_index.insert(lab_ir, block_idx);
        }

        let mut prev_block_idx: Option<usize> = None;

        // Control Flow
        for (block_idx, block) in blocks.iter().enumerate() {
            let mut ended_jump = false;
            for i in block.iter().skip(1) {
                match i {
                    nodes::Ssa::Jump(j) => {
                        let j_block_idx = block_index[&j.to_ir_string()];
                        control_adjacency.entry(block_idx).or_default().push(j_block_idx);
                        ended_jump = true;
                    }
                    nodes::Ssa::Branch {
                        cond: _,
                        true_target,
                        false_target,
                        width: _,
                    } => {
                        let true_block_idx = block_index[&true_target.to_ir_string()];
                        let false_block_idx = block_index[&false_target.to_ir_string()];
                        control_adjacency.entry(block_idx).or_default().extend([true_block_idx, false_block_idx]);
                        ended_jump = true;
                    }
                    _ => (),
                }
            }

            if let Some(pb) = prev_block_idx {
                control_adjacency.entry(pb).or_default().push(block_idx);
            }

            if !ended_jump {
                prev_block_idx = Some(block_idx);
            } else {
                prev_block_idx = None;
            }

        }

        // Data Flow from phi vars

        for (block_idx, block) in blocks.iter().enumerate() {
            for i in block {
                if let nodes::Ssa::Phi(phi) = i {
                    for (merge_addr, merge_label) in &phi.merging {
                        let merge_block_idx = block_index[&merge_label.to_ir_string()]; // TODO: groupby label not string
                        out_blocks[merge_block_idx].used_variables.entry(merge_addr.clone()).or_default().push(block_idx);
                    }
                }
            }
        }

        Self { blocks: out_blocks, control_adjacency, block_index }
    }

    pub fn to_dot(&self) -> String {
        let mut dot_str = String::new();
        dot_str.push_str("digraph SSA {\n");
        dot_str.push_str("rankdir=TB;\n");
        dot_str.push_str("node [shape=rectangle, fontname=\"Courier\"];\n");
        let sep = "<BR ALIGN=\"LEFT\"/>    ";

        for (block_idx, block) in self.blocks.iter().enumerate() {
            dot_str.push_str(&format!("block_{}", block_idx));
            dot_str.push_str(" [label=<");

            let body = block.ir
                .iter()
                .map(|x| {
                    html_escape::encode_safe_to_string(&x.to_ir_string(), &mut String::new()).to_owned()
                })
                .collect::<Vec<_>>()
                .join(sep);
            dot_str.push_str(&body);
            dot_str.push_str(sep);
            dot_str.push_str(">];");

            dot_str.push('\n');
        }

        for (parent_idx, children) in self.control_adjacency.iter() {
            for child_idx in children {
                dot_str.push_str(&format!("block_{} -> block_{};\n", parent_idx, child_idx));
            }
        }

        for (parent_idx, parent) in self.blocks.iter().enumerate() {
            for (used_var, child_idxes) in &parent.used_variables {
                for child_idx in child_idxes {
                    dot_str.push_str(&format!("block_{} -> block_{}[label=\"{} -> phi\"];\n", parent_idx, child_idx, format!("Phi uses {}", used_var.to_ir_string())));
                }
            }
        }

        dot_str
    }

    pub fn blocks(&self) -> &Vec<BasicBlock> {
        &self.blocks
    }
}