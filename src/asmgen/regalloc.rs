use std::collections::HashMap;

use crate::{asmgen::aarch64::instructions::Register, common::Width, ir::nodes::{self, Address, Ssa}};

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
    pub start: usize,
    pub end: usize,
}

impl Lifetime {
   pub fn intersects(&self, other: &Lifetime) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

pub fn analyze_lifetimes(
    body: &[Ssa],
    parameters: &[String]
) -> HashMap<Address, Lifetime> {
    let mut lifetimes = HashMap::new();

    for (idx, b) in body.iter().enumerate() {
        let alive_here = alive_addresses_in_ssa(b);
        for a in alive_here.iter() {
            let mut need_add = true;
            for p in parameters {
                if let Some(svar)  = a.try_get_source() && svar == p {
                    need_add = false;
                    break;
                }
            }
            if need_add {
                let lifetime = lifetimes.entry(a.clone()).or_insert_with(|| Lifetime { start: idx, end: idx });
                lifetime.end = idx;
            }
        }
    }

    lifetimes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Reg(Register),
    Spill(i64), // stack offset
}

#[derive(Debug)]
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
    pub fn new(mut available_regs: Vec<Register>, precolor: HashMap<Address, Allocation>) -> Self {

        available_regs = available_regs.iter().filter(|r| {
            let was_precolored = precolor.values().find(|all| {
                match all.loc {
                    Location::Reg(inner_r) => {
                        r.align(Width::Long) ==inner_r.align(Width::Long)
                    },
                    Location::Spill(_) => false
                }
            });
            was_precolored.is_none()
        }).copied().collect();

        Self {
            available_regs,
            allocations: precolor,
            next_spill_slot: 0,
        }
    }

    pub fn stack_size(&self) -> usize {
        self.next_spill_slot.abs() as usize
    }

    pub fn used_registers_at(&self, instr_idx: usize) -> Vec<Register> {
        self.allocations.values().filter_map(|all| {
            if all.lifetime.start <= instr_idx && all.lifetime.end >= instr_idx &&
             let Location::Reg(r) = all.loc {
                Some(r)
            } else {
                None
            }
        }).collect()
    }

    fn recheck(&self) {
        for a in &self.allocations {
            for b in &self.allocations {
                if a.0 == b.0 {
                    continue;
                }

                let l1 = a.1.lifetime;
                let l2 = b.1.lifetime;

                let loc1 = a.1.loc;
                let loc2 = b.1.loc;

                if l1.intersects(&l2) {
                    assert_ne!(loc1, loc2)
                }
            }
        }
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

        self.recheck();
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

