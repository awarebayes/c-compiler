use crate::{ir::nodes::{self, Address}, opt::{controlflow::{BasicBlock, ControlFlowGraph}, dataflow::DataFlowGraph}};


fn eliminate_dead_code_inside_block(block: &BasicBlock) -> Vec<nodes::Ssa> {
    let mut out = vec![];
    let dataflow = DataFlowGraph::from_basic_block(&block.ir);

    for (instr_idx, instr) in block.ir.iter().enumerate() {
        if let nodes::Ssa::Assignment { dest, source: _, width: _ } = instr {
            let out_children = &dataflow.adjacency().get(&instr_idx);
            if let Address::CompilerTemp(_) = dest && out_children.is_none() {
                continue;
            }
            if let Address::Source(_) = dest && !block.used_variables.contains_key(dest) && out_children.is_none() {
                continue;
            }
        }
        out.push(instr.clone());
    }

    out
}

pub fn eliminate_dead_code(body: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    let control_flow = ControlFlowGraph::new(body);

    let mut out = vec![];

    for b in control_flow.blocks() {
        out.extend(eliminate_dead_code_inside_block(b))
    }

    out
}