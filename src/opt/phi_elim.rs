use std::collections::HashMap;

use crate::{ir::{self, IrTextRepr, block_label, extract_phi_functions, ir_to_basic_blocks, nodes}};


fn zero_source_variables(mut ssa: nodes::Ssa) -> nodes::Ssa {
    match &mut ssa {
        nodes::Ssa::Assignment { dest, source, width: _ } => {
            if let nodes::Address::Source(s) = dest {
                s.1 = 0;
            }

            if let nodes::Address::Source(s) = source {
                s.1 = 0;
            }
        },
        nodes::Ssa::Call { parameters, dest, func: _, num_params: _ } => {
            if let Some(dest) = dest {
                if let nodes::Address::Source(s) = &mut dest.0 {
                    s.1 = 0;
                }
            }
            for p in parameters {
                if let nodes::Address::Source(s) = &mut p.value {
                    s.1 = 0;
                }
            }
        }
        nodes::Ssa::Branch { width: _, cond, true_target: _, false_target: _ } => {
            if let nodes::Address::Source(s) = cond {
                s.1 = 0;
            }
        },
        nodes::Ssa::Return { value } => {
            if let Some(value) = value {
                if let nodes::Address::Source(s) = &mut value.0 {
                    s.1 = 0;
                }
            }
        },
        nodes::Ssa::Quadriplet(quad) => {
            if let Some(dest) = &mut quad.right {
                if let nodes::Address::Source(s) = dest {
                    s.1 = 0;
                }
            }
            
            if let nodes::Address::Source(s) = &mut quad.left {
                s.1 = 0;
            }

            if let nodes::Address::Source(s) = &mut quad.dest {
                s.1 = 0;
            }
        }
        _ => {}
    }

    ssa
}

// fn eliminate_phi_block(
//     block: &[nodes::Ssa],
//     mut added_end: Vec<nodes::Ssa>,
// ) -> Vec<nodes::Ssa> {
//     let mut out = vec![];

//     for (idx, b) in block.iter().enumerate() {
//         if matches!(b, nodes::Ssa::Phi(_)) {
//             continue;
//         }

//         let mut need_push = true;

//         if idx == block.len() - 1 {
//             // last block
//             match b {
//                 nodes::Ssa::Return { value: _ }
//                 | nodes::Ssa::Jump(_)
//                 | nodes::Ssa::Branch {
//                     cond: _,
//                     true_target: _,
//                     false_target: _,
//                     width: _,
//                 } => {
//                     added_end.push(b.clone());
//                     need_push = false;
//                 }
//                 _ => (),
//             }
//         }
//         if need_push {
//             out.push(b.clone());
//         }
//     }
//     out.extend(added_end);
//     out
// }

pub fn eliminate_phi_body(body: &[nodes::Ssa]) -> Vec<ir::nodes::Ssa> {
    let mut out = vec![];

    for b in body {
        if matches!(b, nodes::Ssa::Phi(_)) {
            continue;
        }
        out.push(zero_source_variables(b.clone()));
    }

    out
}