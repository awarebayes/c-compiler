use crate::{ir::nodes, opt::{constant_folding, dead_code_elimination, phi_elim}};

pub type OptimisationPassFn = fn(&[nodes::Ssa]) -> Vec<nodes::Ssa>;

pub struct OptimisationLevel<const N: usize> {
    passes: [OptimisationPassFn; N],
}

impl<const N: usize> OptimisationLevel<N> {
    pub fn optimize(&self, ir: &[nodes::Ssa]) -> Vec<nodes::Ssa> {
        let mut current = ir.to_vec();
        for &pass in &self.passes {
            current = pass(&current);
        }
        current
    }
}

pub const O1: OptimisationLevel<3> = OptimisationLevel {
    passes: [
        constant_folding::fold_constants,
        dead_code_elimination::eliminate_dead_code,

        // Make sure it is always at the end!
        phi_elim::eliminate_phi_body,
    ],
};