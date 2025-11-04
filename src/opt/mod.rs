use crate::{ir::nodes, opt::phi_elim::eliminate_phi_body};

mod phi_elim;

pub fn o1(unit: &[nodes::ToplevelItem]) -> Vec<nodes::ToplevelItem> {
    unit.iter()
        .map(|u| match u {
            nodes::ToplevelItem::Function(f) => {
                let optimized_body = &f.body;
                let elim_phi = eliminate_phi_body(&optimized_body);
                nodes::ToplevelItem::Function(nodes::FunctionDef {
                    name: f.name.clone(),
                    return_width: f.return_width.clone(),
                    parameters: f.parameters.clone(),
                    body: elim_phi,
                })
            }
            nodes::ToplevelItem::Declaration(_) => u.clone(),
        })
        .collect()
}
