use std::collections::HashMap;

use crate::ir::{self, IrTextRepr, block_label, extract_phi_functions, ir_to_basic_blocks, nodes};

pub fn eliminate_phi_block(
    block: &[nodes::Ssa],
    mut added_end: Vec<nodes::Ssa>,
) -> Vec<nodes::Ssa> {
    let mut out = vec![];

    for (idx, b) in block.iter().enumerate() {
        if matches!(b, nodes::Ssa::Phi(_)) {
            continue;
        }

        let mut need_push = true;

        if idx == block.len() - 1 {
            // last block
            match b {
                nodes::Ssa::Return { value: _ }
                | nodes::Ssa::Jump(_)
                | nodes::Ssa::Branch {
                    cond: _,
                    true_target: _,
                    false_target: _,
                } => {
                    added_end.push(b.clone());
                    need_push = false;
                }
                _ => (),
            }
        }
        if need_push {
            out.push(b.clone());
        }
    }
    out.extend(added_end);
    out
}

pub fn eliminate_phi_body(body: &[nodes::Ssa]) -> Vec<ir::nodes::Ssa> {
    let blocks = ir_to_basic_blocks(&body);
    let mut out = vec![];

    let mut added_end_assgn: HashMap<String, Vec<nodes::Ssa>> = HashMap::new();

    for b in blocks.iter() {
        let phis = extract_phi_functions(b);
        for p in phis {
            let dest = p.dest;
            for merge in p.merging {
                added_end_assgn
                    .entry(merge.1.to_ir_string())
                    .or_default()
                    .push(ir::nodes::Ssa::Assignment {
                        dest: dest.clone(),
                        source: merge.0,
                        width: p.width,
                    });
            }
        }
    }

    for b in blocks.iter() {
        let block_name = block_label(b).unwrap();
        if let Some(added) = added_end_assgn.remove(&block_name) {
            let eliminated = eliminate_phi_block(b, added);
            out.extend(eliminated);
        } else {
            let eliminated = eliminate_phi_block(b, vec![]);
            out.extend(eliminated);
        }
    }

    out
}
