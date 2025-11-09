use crate::{ir::nodes};
mod phi_elim;
mod optimisation;
mod constant_folding;
mod dead_code_elimination;
mod dataflow;
mod controlflow;

pub fn run_o1(unit: &[nodes::ToplevelItem]) -> Vec<nodes::ToplevelItem> {
    unit.iter()
        .map(|u| match u {
            nodes::ToplevelItem::Function(f) => {
                let body = &f.body;
                let o1 = optimisation::O1;
                nodes::ToplevelItem::Function(nodes::FunctionDef {
                    name: f.name.clone(),
                    return_width: f.return_width.clone(),
                    parameters: f.parameters.clone(),
                    body: o1.optimize(&body),
                })
            }
            nodes::ToplevelItem::Declaration(_) => u.clone(),
        })
        .collect()
}
