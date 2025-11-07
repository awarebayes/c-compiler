use std::collections::HashMap;

use crate::{asmgen::aarch64::instructions::Register, ir::nodes::{self, Address, Ssa}};

pub fn alive_addresses_in_ssa(ssa: &Ssa) -> Vec<Address> {
    match ssa {
        Ssa::Assignment { dest, source, width: _ } => {
            if matches!(source, Address::Constant(_)) {
                vec![dest.clone()]
            } else {
                vec![dest.clone(), source.clone()]
            }
        },
        Ssa::Branch { cond, true_target: _, false_target: _ , width: _} => {
            vec![cond.clone()]
        },
        Ssa::Call { dest, func, num_params: _, parameters } => {
            let mut res = vec![];
            if let Some(d) = dest {
                res.push(d.0.clone());
            }
            if matches!(func, Address::CompilerTemp(_)) {
                res.push(func.clone());
            }
            for p in parameters {
                res.push(p.value.clone());
            }
            res
        },
        Ssa::Jump(_) => vec![],
        Ssa::Label(_) => vec![],
        Ssa::Phi(_) => panic!("Phis should be eliminated at this point..."),
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

#[derive(Debug, Clone, Copy)]
pub struct Lifetime {
    start: usize,
    end: usize,
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

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Reg(Register),
    Spill(i64), // stack offset
}

pub struct Allocation {
    pub loc: Location,
    pub lifetime: Lifetime,
}

pub struct LinearScanRegisterAlloc {
    available_regs: Vec<Register>,
    allocations: HashMap<Address, Allocation>,
    next_spill_slot: i64,
}

impl LinearScanRegisterAlloc {
    pub fn new(available_regs: Vec<Register>) -> Self {
        Self {
            available_regs,
            allocations: HashMap::new(),
            next_spill_slot: 0,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.next_spill_slot.abs() as usize
    }

    pub fn linear_scan(&mut self, lifetimes: &HashMap<Address, Lifetime>) {
        // Sort intervals by start time
        let mut intervals: Vec<_> = lifetimes.iter().collect();
        intervals.sort_by_key(|(_, lt)| lt.start);

        let mut active: Vec<(Address, Lifetime, Register)> = Vec::new();

        for (addr, lifetime) in intervals {
            // expire old intervals
            active.retain(|(_, lt, _)| lt.end >= lifetime.start);

            // try allocate register
            if let Some(&reg) = self.available_regs
                .iter()
                .find(|r| !active.iter().any(|(_, _, ar)| ar == *r))
            {
                self.allocations.insert(addr.clone(), Allocation {
                    loc: Location::Reg(reg),
                    lifetime: *lifetime,
                });
                active.push((addr.clone(), *lifetime, reg));
            } else {
                // Spill this variable
                let spill_off = self.next_spill_slot;
                self.next_spill_slot += 8; // assuming 8-byte slots
                self.allocations.insert(addr.clone(), Allocation {
                    loc: Location::Spill(spill_off),
                    lifetime: *lifetime,
                });
            }
        }
    }


    pub fn location_of(&self, addr: &Address, instr_index: usize) -> Option<Location> {
        self.allocations.get(addr).and_then(|a| {
            if instr_index >= a.lifetime.start && instr_index <= a.lifetime.end {
                Some(a.loc)
            } else {
                None
            }
        })
    }

}

