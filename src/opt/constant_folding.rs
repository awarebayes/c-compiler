use std::collections::HashMap;

use crate::ir::{ir_to_basic_blocks, nodes::{self, Address}};

fn fold_constants_block(block: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    let mut constants: HashMap<nodes::Address, i64> = HashMap::new();
    let mut out = vec![];

    for i in block.iter() {
        match i {
            nodes::Ssa::Assignment { dest, source, width } => {
                if let Address::Constant(nodes::AddressConstant::Numeric(num)) = source {
                    constants.insert(dest.clone(), *num);
                    out.push(i.clone());
                } else {
                    if let Some(c) = constants.get(source).copied() {
                        constants.insert(dest.clone(), c);
                        out.push(nodes::Ssa::Assignment { dest: dest.clone(), source: nodes::Address::constant(nodes::AddressConstant::Numeric(c)), width: *width });
                    } else {
                        out.push(i.clone());
                    }
                }
            },
            nodes::Ssa::Quadriplet(quad) => {
                let left_c = constants.get(&quad.left);
                let right_c = constants.get(quad.right.as_ref().unwrap());
                if let Some(left) = left_c && let Some(right) = right_c {
                    let const_res = quad.op.apply_constant(*left, *right);
                    constants.insert(quad.dest.clone(), const_res);
                    out.push(nodes::Ssa::Assignment { dest: quad.dest.clone(), source: nodes::Address::constant(nodes::AddressConstant::Numeric(const_res)), width: quad.width });
                } else {
                    out.push(i.clone());
                }
            },
            nodes::Ssa::Return { value } => {
                if let Some((addr, width)) = value &&
                    let Some(const_val) = constants.get(addr) {
                    out.push(nodes::Ssa::Return { value: Some((Address::constant(nodes::AddressConstant::Numeric(*const_val)), *width)) });                       

                } else {
                    out.push(i.clone());
                }
            },
            nodes::Ssa::Branch { width, cond, true_target, false_target } => {
                if let Some(const_cond) = constants.get(cond) {
                    out.push(nodes::Ssa::Branch { width: *width, cond: nodes::Address::constant_i64(*const_cond), true_target: true_target.clone(), false_target: false_target.clone() });
                } else {
                    out.push(i.clone());
                }
            }
            _ => {
                out.push(i.clone());
            }
        }
    }


    out
}

pub fn fold_constants(body: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
    let blocks = ir_to_basic_blocks(&body);
    let mut out = vec![];

    for b in blocks.iter() {
        out.extend(fold_constants_block(b))
    }

    out
}