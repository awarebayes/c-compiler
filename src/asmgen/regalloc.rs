use std::collections::HashMap;
use std::hash::Hash;

use crate::{asmgen::aarch64::instructions::Register, ir::nodes::{self, Address, Ssa}};

#[derive(Debug)]
pub struct Lifetime {
    start: usize,
    end: usize,
}

pub fn alive_addresses_in_ssa(ssa: &Ssa) -> Vec<Address> {
    match ssa {
        Ssa::Assignment { dest, source, width: _ } => {
            if matches!(source, Address::Constant(_)) {
                vec![dest.clone()]
            } else {
                vec![dest.clone(), source.clone()]
            }
        },
        Ssa::Branch { cond, true_target: _, false_target: _ } => {
            vec![cond.clone()]
        },
        Ssa::Call { dest, func, num_params: _ } => {
            let mut res = vec![];
            if let Some(d) = dest {
                res.push(d.0.clone());
            }
            if matches!(func, Address::CompilerTemp(_)) {
                res.push(func.clone());
            }
            res
        },
        Ssa::Jump(_) => vec![],
        Ssa::Label(_) => vec![],
        Ssa::Phi(_) => panic!("Phis should be eliminated at this point..."),
        Ssa::Param { number: _, value, width: _ } => vec![value.clone()],
        Ssa::Quadriplet(q) => {
            let mut res = vec![];
            res.push(q.dest.clone());
            res.push(q.left.clone());
            if let Some(qr) = &q.right && !matches!(qr, Address::Constant(_)){
                res.push(qr.clone());
            }
            res
        },
        Ssa::Return { value } => {
            if let Some(v) = value && !matches!(v.0, Address::Constant(_)) {
                vec![v.0.clone()]
            } else {
                vec![]
            }
        }
    }
}

pub fn analyze_lifetimes(
    body: &[Ssa]
) -> HashMap<Address, Lifetime> {
    let mut lifetimes = HashMap::new();

    for (idx, b) in body.iter().enumerate() {
        let alive_here = alive_addresses_in_ssa(b);
        for a in alive_here.iter() {
            let lifetime = lifetimes.entry(a.clone()).or_insert_with(|| Lifetime { start: idx, end: idx });
            lifetime.end = idx;
        }
    }

    lifetimes
}



pub struct LinearScanRegisterAlloc {
    available_regs: Vec<Register>,
    allocations: HashMap<Address, Register>,
    active: Vec<(Address, Register, usize)>, // (temp, reg, end)
    spill_slots: HashMap<Address, i64>,
    next_spill_offset: i64,

    pre_colored: HashMap<Address, Register>,
}

impl LinearScanRegisterAlloc {
    pub fn new(regs: Vec<Register>) -> Self {
        Self {
            available_regs: regs,
            allocations: HashMap::new(),
            spill_slots: HashMap::new(),
            active: vec![],
            next_spill_offset: 0,
        }
    }

    pub fn allocate_for_variable(&mut self,
        var: Address, 
        lifetime: &Lifetime,
        position: usize
    ) -> (Register, Option<(Address, i64)>) {
        self.expire_old_lifetimes(position);
        if let Some(reg) = self.available_regs.pop() {
            self.allocations.insert(var.clone(), reg);
            self.active.push((var, reg, lifetime.end));
            (reg, None)
        } else {
            self.spill_one(var, lifetime)
        }
    }

    fn spill_one(&mut self, new_var: Address, new_lifetime: &Lifetime) -> (Register, Option<(Address, i64)>) {
        // spill the first active variable
        if let Some((spill_var, spill_reg, _)) = self.active.pop() {
            let spill_offset = self.next_spill_offset;
            self.next_spill_offset += 8;  // 8 bytes per spill slot
            self.spill_slots.insert(spill_var.clone(), spill_offset);
            
            // Use this register for the new variable
            self.allocations.insert(new_var.clone(), spill_reg);
            self.active.push((new_var, spill_reg, new_lifetime.end));
            (spill_reg, Some((spill_var, spill_offset)))
        } else {
            panic!("Cant spill!")
        }
    }

    pub fn expire_old_lifetimes(&mut self, position: usize) {
        self.active.retain(|(var, reg, end)| {
            if position > *end {
                self.available_regs.push(*reg);
                self.allocations.remove(var);
                false
            } else {
                true
            }
        });
    }

    pub fn get_register(&self, var: &Address) -> Option<Register> {
        self.allocations.get(var).copied()
    }
    
    pub fn is_spilled(&self, var: &Address) -> bool {
        self.spill_slots.contains_key(var)
    }

    pub fn get_spill_slot(&self, var: &Address) -> Option<i64> {
        self.spill_slots.get(var).copied()
    }
}

