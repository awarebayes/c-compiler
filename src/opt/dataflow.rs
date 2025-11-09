use std::collections::{HashMap, HashSet};

use crate::ir::{IrTextRepr, nodes::{self, Address}};

pub struct DataFlowGraph {
    ir: Vec<nodes::Ssa>,
    adjacency: HashMap<usize, Vec<usize>>,
    address_assignment: HashMap<Address, usize>,
    do_not_optimize: Vec<usize>,
}

impl DataFlowGraph {
    pub fn from_basic_block(block: &[nodes::Ssa]) -> Self 
    {
        let mut address_assignment: HashMap<Address, usize> = HashMap::new();
        let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut do_not_optimize = Vec::new();

        for (idx, b) in block.iter().enumerate() {
            match b {
                nodes::Ssa::Phi(phi) => {
                    address_assignment.insert(phi.dest.clone(), idx);
                },
                nodes::Ssa::Assignment { dest, source, width: _ } => {
                    address_assignment.insert(dest.clone(), idx);
                    match source {
                        Address::Constant(_) => {},
                        Address::CompilerTemp(_) | Address::Source(_) => {
                            if let Some(source_def) = address_assignment.get(source) {
                                adjacency.entry(*source_def).or_default().push(idx);
                            }
                        }
                    }
                },
                nodes::Ssa::Quadriplet(quad) => {
                    address_assignment.insert(quad.dest.clone(), idx);
                    if let Some(&left) = address_assignment.get(&quad.left) {
                        adjacency.entry(left).or_default().push(idx);
                    }
                    if let Some(&right) = address_assignment.get(quad.right.as_ref().unwrap()) {
                        adjacency.entry(right).or_default().push(idx);
                    }
                },
                nodes::Ssa::Call { parameters, dest, func: _, num_params: _ } => {
                    if let Some((dest, _)) = dest {
                        address_assignment.insert(dest.clone(), idx);
                    }
                    for p in parameters {
                        let parameter_def = address_assignment[&p.value];
                        adjacency.entry(parameter_def).or_default().push(idx);
                    }
                },
                nodes::Ssa::Return { value } => {
                    if let Some((source, _)) = value &&
                       let Some(source_def) = address_assignment.get(&source) {
                        adjacency.entry(*source_def).or_default().push(idx);
                    }
                },
                _ => {
                    do_not_optimize.push(idx);
                }
            }
        }

        Self { ir: block.to_vec(), adjacency, address_assignment, do_not_optimize }
    }

    pub fn do_not_optimize(&self) -> &Vec<usize> {
        &self.do_not_optimize
    }

    pub fn adjacency(&self) -> &HashMap<usize, Vec<usize>> {
        &self.adjacency
    }

    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();
        
        for &node in self.adjacency.keys() {
            if !visited.contains(&node) {
                let mut component = Vec::new();
                self.dfs(node, &mut visited, &mut component);
                components.push(component);
            }
        }
        
        components
    }

    fn dfs(&self, node: usize, visited: &mut HashSet<usize>, component: &mut Vec<usize>) {
        visited.insert(node);
        component.push(node);
        
        if let Some(neighbors) = self.adjacency.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    self.dfs(neighbor, visited, component);
                }
            }
        }
    }

    pub fn to_dot(&self) -> String {
        let mut dot_str = String::new();
        dot_str.push_str("digraph SSA_Dataflow {\n");
        dot_str.push_str("rankdir=TB;\n");
        dot_str.push_str("node [shape=box, fontname=\"Courier\"];\n");
        dot_str.push_str("edge [fontname=\"Courier\"];\n");

        for (idx, b) in self.ir.iter().enumerate() {
            if self.do_not_optimize.contains(&idx) {
                continue;
            }
            dot_str.push_str(&format!("block_{} [label=\"{}\"];\n", idx, b.to_ir_string().replace("\t", "") ));
        }

        for (parent, children) in self.adjacency.iter() {
            for child in children {
                dot_str.push_str(&format!("block_{} -> block_{};\n", parent, child));
            }
        }

        dot_str.push_str("}\n");
        dot_str
    }
}

