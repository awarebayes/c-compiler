use std::fs;

use crate::ir::{IrTextRepr, ir_to_basic_blocks, nodes};

pub fn graphviz_function(declaration: &nodes::FunctionDef) -> String {
    let blocks = ir_to_basic_blocks(&declaration.body);
    let mut res = String::new();
    res.push_str("digraph SSA {\n");
    res.push_str("rankdir=TB;\n");
    res.push_str("node [shape=rectangle, fontname=\"Courier\"];\n");
    let sep = "<BR ALIGN=\"LEFT\"/>    ";
    for block in blocks.iter() {
        let label = &block[0];
        if let nodes::Ssa::Label(lab) = label {
            res.push_str(&lab.to_ir_string());
            res.push(' ');
            res.push('[');
            res.push_str(&format!("label=<{}", lab.to_ir_string()));
            res.push_str(sep);
        }

        let body = block
            .iter()
            .skip(1)
            .map(|x| {
                html_escape::encode_safe_to_string(&x.to_ir_string(), &mut String::new()).to_owned()
            })
            .collect::<Vec<_>>()
            .join(sep);
        res.push_str(&body);
        res.push_str(sep);
        res.push('>');
        res.push(']');
        res.push(';');

        res.push('\n');
    }

    // Control Flow
    let mut prev_block = None;
    for block in blocks.iter() {
        let label = &block[0];
        let mut lab_ir = String::new();
        if let nodes::Ssa::Label(lab) = label {
            lab_ir = lab.to_ir_string();
        }
        let mut ended_jump = false;
        for i in block.iter().skip(1) {
            match i {
                nodes::Ssa::Jump(j) => {
                    res.push_str(&format!("{} -> {};\n", lab_ir, j.to_ir_string()));
                    ended_jump = true;
                }
                nodes::Ssa::Branch {
                    cond: _,
                    true_target,
                    false_target,
                } => {
                    res.push_str(&format!(
                        "{} -> {}[label=\"true\"];\n",
                        lab_ir,
                        true_target.to_ir_string()
                    ));
                    res.push_str(&format!(
                        "{} -> {}[label=\"false\"];\n",
                        lab_ir,
                        false_target.to_ir_string()
                    ));
                    ended_jump = true;
                }
                _ => (),
            }
        }

        if let Some(pb) = &prev_block {
            res.push_str(&format!("{} -> {};\n", pb, lab_ir));
        }
        if !ended_jump {
            prev_block = Some(lab_ir);
        } else {
            prev_block = None;
        }
    }

    // Data flow (phi)
    res.push_str("edge [color=red, style=dashed, constraint=false];\n");

    for block in blocks.iter() {
        let label = &block[0];
        let mut lab_ir = String::new();
        if let nodes::Ssa::Label(lab) = label {
            lab_ir = lab.to_ir_string();
        }
        for i in block.iter().skip(1) {
            match i {
                nodes::Ssa::Phi(phi) => {
                    for (addr, lab) in &phi.merging {
                        res.push_str(&format!(
                            "{} -> {}[label=\"{} -> phi\"];\n",
                            lab.to_ir_string(),
                            lab_ir,
                            addr.to_ir_string()
                        ));
                    }
                }
                _ => (),
            }
        }
    }

    res.push_str("}");
    res
}

pub fn graphviz_unit(unit: &[nodes::ToplevelItem], dir: &str) {
    for i in unit {
        match i {
            nodes::ToplevelItem::Function(f) => {
                let u = &graphviz_function(f);
                fs::write(format!("{}/{}.dot", dir, f.name), u).unwrap();
            }
            _ => (),
        }
    }
}
