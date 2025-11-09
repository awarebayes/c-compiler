use std::collections::{HashMap, HashSet};

use crate::{ir::nodes::{self, Address}, opt::{controlflow::{BasicBlock, ControlFlowGraph}, dataflow::DataFlowGraph}};

fn eliminate_copy(ssa: &nodes::Ssa, from: nodes::Address, to: nodes::Address) -> nodes::Ssa {
    match ssa {
        nodes::Ssa::Assignment { dest, source, width } => {
            assert_eq!(&from, source);
            nodes::Ssa::Assignment { dest: dest.clone(), source: to, width: *width }
        },
        nodes::Ssa::Quadriplet(quad) => {
            if quad.left == from {
                nodes::Ssa::Quadriplet(nodes::Quadriplet { width: quad.width, dest: quad.dest.clone(), op: quad.op, left: to, right: quad.right.clone() })

            } else if quad.right.as_ref().unwrap() == &from {
                nodes::Ssa::Quadriplet(nodes::Quadriplet { width: quad.width, dest: quad.dest.clone(), op: quad.op, left: quad.left.clone(), right: Some(to) })
            } else {
                panic!("replace what?");
            }
        },
        nodes::Ssa::Branch { width, cond, true_target, false_target } => {
            assert_eq!(cond, &from);
            nodes::Ssa::Branch { width: *width, cond: to, true_target: true_target.clone(), false_target: false_target.clone() }
        },
        nodes::Ssa::Return { value } => {
            assert_eq!(value.as_ref().unwrap().0, from);
            nodes::Ssa::Return { value: Some((to, value.as_ref().unwrap().1)) }
        },
        nodes::Ssa::Call { parameters, dest, func, num_params } => {
            let mut copy_parameters = parameters.clone();
            let changed = parameters.iter().position(|x| x.value == from).expect("Change what");
            copy_parameters[changed].value = to;

            nodes::Ssa::Call { parameters: copy_parameters, dest: dest.clone(), func: func.clone(), num_params: *num_params }
        },
        _ => panic!("Cannot eliminate copy here?")
    }
}

fn eliminate_quadriplets(block: &BasicBlock) -> Vec<nodes::Ssa> {
    let mut out = vec![];
    let dataflow = DataFlowGraph::from_basic_block(&block.ir);

    // instruction index, Vec<(From, To)>
    let mut ignore = HashSet::new();

    for (instr_idx, instr) in block.ir.iter().enumerate() {
        let mut instr_to_push = instr.clone();

        if let nodes::Ssa::Quadriplet(quad) = &mut instr_to_push {
            let out_children = &dataflow.adjacency().get(&instr_idx);
            if let Some(out_children) = out_children && out_children.len() == 1 {
                let out_child = &block.ir[out_children[0]];
                if let nodes::Ssa::Assignment { dest: ass_dist, source: _, width: _ } = out_child {
                    quad.dest = ass_dist.clone();
                    ignore.insert(out_children[0]);
                }
            }
        }

        if let nodes::Ssa::Call { parameters: _, dest, func: _, num_params: _  } = &mut instr_to_push {
            if let Some(dest) = dest {
                let out_children = &dataflow.adjacency().get(&instr_idx);
                if let Some(out_children) = out_children && out_children.len() == 1 {
                    let out_child = &block.ir[out_children[0]];
                    if let nodes::Ssa::Assignment { dest: ass_dist, source: _, width: _ } = out_child {
                        dest.0 = ass_dist.clone();
                        ignore.insert(out_children[0]);
                    }
                }
            }
        }

        if !ignore.contains(&instr_idx) {
            out.push(instr_to_push);
        }
    }

    out
}


fn eliminate_forward_assignments(block: &BasicBlock) -> Vec<nodes::Ssa> {
    let mut out = vec![];
    let dataflow = DataFlowGraph::from_basic_block(&block.ir);

    // instruction index, Vec<(From, To)>
    let mut replacements: HashMap<usize, Vec<(Address, Address)>> = HashMap::new();

    for (instr_idx, instr) in block.ir.iter().enumerate() {
        let mut instr_to_push = instr.clone();
        if let nodes::Ssa::Assignment { dest, source, width: _ } = instr {
            let out_children = &dataflow.adjacency().get(&instr_idx);
            if let Some(out_children) = out_children && out_children.len() == 1 && !matches!(source, Address::Constant(nodes::AddressConstant::StringLiteral(_))) {
                let other_use = out_children[0];

                replacements.entry(other_use).or_default().push((dest.clone(),  source.clone()));
                continue;
            }
        }

        if let Some(current_repl) = replacements.get(&instr_idx) {
            for (from, to) in current_repl {
                instr_to_push = eliminate_copy(&instr_to_push, from.clone(), to.clone());
            }
        } 

        out.push(instr_to_push);
    }

    out
}


pub fn copy_eliminate_forward_assignments(body: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    let control_flow = ControlFlowGraph::new(body);
    let mut out = vec![];
    for b in control_flow.blocks() {
        out.extend(eliminate_forward_assignments(b))
    }

    out
}


pub fn copy_eliminate_quadriplets(body: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    let control_flow = ControlFlowGraph::new(body);
    let mut out = vec![];
    for b in control_flow.blocks() {
        out.extend(eliminate_quadriplets(b))
    }

    out
}

pub fn copy_eliminate(body: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    copy_eliminate_forward_assignments(&copy_eliminate_quadriplets(body))
}