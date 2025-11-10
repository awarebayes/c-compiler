use std::collections::HashMap;
use std::rc::Rc;

use crate::asmgen::aarch64::instructions;
use crate::asmgen::aarch64::instructions::Instruction;
use crate::asmgen::aarch64::instructions::RValue;
use crate::asmgen::aarch64::instructions::Register;
use crate::asmgen::aarch64::instructions::Symbol;
use crate::asmgen::regalloc;
use crate::asmgen::regalloc::LinearScanRegisterAlloc;
use crate::asmgen::regalloc::Location;
use crate::asmgen::regalloc::analyze_lifetimes;
use crate::asmgen::lookup_table::{SymbolAddress, SymbolLookup};
use crate::common::StorageClass;
use crate::common::Width;
use crate::ir::IrTextRepr;
use crate::ir::nodes;
use crate::ir::nodes::Address;

// fn address_to_asm_str(adress: &nodes::Address, lookup: &SymbolLookup) -> String {
//     match address
// }

impl nodes::Label {
    fn to_asm_label(&self, func_name: &str) -> String {
        match self {
            Self::CompilerTemp(ct) => format!("L_{}_{}", func_name, ct),
            Self::Source(s) => s.as_str().to_owned(),
        }
    }
}

fn load_if_needed(instructions: &mut Vec<Instruction>, loc: regalloc::Location, spill_load_register: Register, dynamic_offset: i64) -> Register {
    match loc {
        regalloc::Location::Reg(r) => r.align(spill_load_register.width),
        regalloc::Location::Spill(stack_off) => {
            instructions.push(Instruction::Load {
                    width: spill_load_register.width,
                    dest: spill_load_register,
                    operand: instructions::AddressingMode::stack_offset(stack_off + dynamic_offset),
            });
            spill_load_register
        }
    }
}

fn empty_register(loc: regalloc::Location, spill_load_register: Register) -> Register {
    match loc {
        regalloc::Location::Reg(r) => r.align(spill_load_register.width),
        regalloc::Location::Spill(_) => {
            spill_load_register
        }
    }
}

fn store_if_needed(instructions: &mut Vec<Instruction>, loc: regalloc::Location, reg: Register) {
    match loc {
        regalloc::Location::Reg(r) => {
            if reg != r.align(reg.width) {
                instructions.push(Instruction::Mov {
                    dest: r.align(reg.width),
                    operand: reg.rvalue(),
                });
            }
        },
        regalloc::Location::Spill(stack_off) => {
            instructions.push(Instruction::Store {
                    width: reg.width,
                    source: reg,
                    operand: instructions::AddressingMode::stack_offset(stack_off),
            });
        }
    }
}

fn alloc_stack(instructions: &mut Vec<Instruction>, bytes: usize) {
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Sub,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(bytes.next_multiple_of(16) as i64),
    }));
}

fn pop_stack(instructions: &mut Vec<Instruction>, bytes: usize) {
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Add,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(bytes.next_multiple_of(16) as i64),
    }));
}

fn alloc_stack_spills(instructions: &mut Vec<Instruction>, regs: &[Register]) -> usize {
    let stack_space = (regs.len() * 8).next_multiple_of(16);
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Sub,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(stack_space as i64), 
    }));

    for (idx, reg) in regs.iter().enumerate() {
        instructions.push(Instruction::Comment(format!("Spilling {} which is in use", reg.to_string())));
        instructions.push(Instruction::Store {
            width: Width::Long,
            source: reg.align(Width::Long),
            operand: instructions::AddressingMode::stack_offset(8 * idx as i64),
        });
    }


    stack_space
}

fn pop_stack_spills(instructions: &mut Vec<Instruction>, regs: &[Register]) {
    let stack_space = (regs.len() * 8).next_multiple_of(16);
    for (idx, reg) in regs.iter().enumerate() {
        instructions.push(Instruction::Comment(format!("Popping {} which was in use", reg.to_string())));
        instructions.push(Instruction::Load {
            width: Width::Long,
            dest: reg.align(Width::Long),
            operand: instructions::AddressingMode::stack_offset(8 * idx as i64),
        });
    }


    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Add,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(stack_space as i64),
    }));
}

fn handle_param(instructions: &mut Vec<Instruction>, allocator: &LinearScanRegisterAlloc, param: &nodes::FunctionParameter, idx: usize, scratch_register_1: Register, dynamic_offset: i64) {
    instructions.push(Instruction::Comment(param.to_ir_string()));
    assert!(!param.is_variadic);
    let width = param.width;
    let arg_reg = match param.number {
        0 => Register::x0(width),
        1 => Register::x1(width),
        2 => Register::x2(width),
        _ => todo!(),
    };

    match param.value {
        Address::Constant(nodes::AddressConstant::Numeric(nc)) => {
            instructions.push(Instruction::Mov {
                dest: arg_reg,
                operand: RValue::Immediate(nc)
            });
        },
        _ => {
            let scratch_register_1 = scratch_register_1.align(width);
            let param_loc = allocator.location_of(&param.value, idx).unwrap();
            let param_reg = load_if_needed(instructions, param_loc, scratch_register_1, dynamic_offset);

            // Maybe check if arg_reg is empty here?
            if arg_reg != param_reg {
                instructions.push(Instruction::Mov {
                    dest: arg_reg,
                    operand: param_reg.rvalue(),
                });
            }
        }
    }

}


fn handle_variadic_params(instructions: &mut Vec<Instruction>, allocator: &LinearScanRegisterAlloc, idx: usize, params: &[&nodes::FunctionParameter], scratch_register_1: Register, dynamic_offset: i64) -> usize {
    const APPLE_VARARG_SLOT_SIZE: usize = 8;
    let allocated = (APPLE_VARARG_SLOT_SIZE * params.len()).next_multiple_of(16);

    alloc_stack(instructions, allocated);

    for (param_idx, param) in params.iter().enumerate() {
        instructions.push(Instruction::Comment(param.to_ir_string()));
        let param_loc =  allocator.location_of(&param.value, idx).unwrap();
        let width = param.width;

        let scratch_register_1 = scratch_register_1.align(width);
        let param_reg = load_if_needed(instructions, param_loc, scratch_register_1, dynamic_offset + allocated as i64);

        if let Address::Constant(nodes::AddressConstant::Numeric(nc)) =  param.value {
            instructions.push(Instruction::Mov {
                dest: param_reg.align(Width::Long),
                operand: RValue::Immediate(nc)
            });
        }

        instructions.push(Instruction::Store {
            width: Width::Long,
            source: param_reg.align(Width::Long),
            operand: instructions::AddressingMode::stack_offset(8 * param_idx as i64)
        });
    }

    allocated
}

fn generate_precolor(parameters: &[(String, Width)], body_len: usize) -> HashMap<Address, regalloc::Allocation> {
    let mut hm = HashMap::new();
    for (p_idx, p) in parameters.iter().enumerate() {
        let loc = match p_idx {
            0 => Location::Reg(Register::x0(p.1)),
            1 => Location::Reg(Register::x1(p.1)),
            2 => Location::Reg(Register::x2(p.1)),
            _ => todo!()
        };
        let alloc = regalloc::Allocation {
            loc,
            lifetime: regalloc::Lifetime {
                start: 0,
                end: body_len
            }
        };
        hm.insert(Address::Source((Rc::new(p.0.clone()), 0)), alloc);
    }
    hm
}

fn body_to_asm(
    block: &[nodes::Ssa],
    func_name: &str,
    parameters: &[(String, Width)],
    lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {

    let parameter_names: Vec<String> = parameters.iter().map(|x| x.0.clone()).collect();

    let lifetimes = analyze_lifetimes(block, &parameter_names);

    let mut allocator = LinearScanRegisterAlloc::new(vec![
        Register::x0(Width::Long),
        Register::x1(Width::Long),
        Register::x2(Width::Long),
        Register::x3(Width::Long),
        Register::x4(Width::Long),
    ], 
        generate_precolor(parameters, block.len())
    );

    allocator.linear_scan(&lifetimes);

    let stack_size = allocator.stack_size();

    let scratch_register_1 = Register::x5(Width::Long);
    let scratch_register_2 = Register::x6(Width::Long);
    let scratch_register_3 = Register::x7(Width::Long);

    let mut result = vec![];

    alloc_stack(&mut result, stack_size);

    for (idx, b) in block.iter().enumerate() {
        result.push( Instruction::Comment( b.to_ir_string() ) );
        match b {
            nodes::Ssa::Assignment {
                dest,
                source,
                width,
            } => {
                if let nodes::Address::Constant(nodes::AddressConstant::Numeric(num_const)) = source
                {
                    let dest_loc = allocator.location_of(dest, idx).unwrap();
                    match dest_loc {
                        regalloc::Location::Reg(r) => {
                            result.push(Instruction::Mov {
                                dest: r.align(*width),
                                operand: instructions::RValue::Immediate(*num_const),
                            });
                        },
                        regalloc::Location::Spill(_) => {
                            result.push(Instruction::Mov {
                                dest: scratch_register_1.align(*width),
                                operand: instructions::RValue::Immediate(*num_const),
                            });

                            store_if_needed(&mut result, dest_loc, scratch_register_1.align(*width));
                        }
                    }
                } else if let nodes::Address::Constant(nodes::AddressConstant::StringLiteral(sl)) =
                    source
                {

                    let scratch_register_1 = scratch_register_1.align(*width);
                    let dest_loc = allocator.location_of(dest, idx).unwrap();
                    let dest_reg = empty_register(dest_loc, scratch_register_1);

                    let sl_label = lookup
                        .get(&nodes::Address::Constant(
                            nodes::AddressConstant::StringLiteral(sl.clone()),
                        ))
                        .unwrap();
                    let sl_label_str = match sl_label.address {
                        SymbolAddress::StringLiteral(sl_count) => format!("sl{}", sl_count),
                        _ => panic!(),
                    };

                    result.push(Instruction::AdressPage {
                        dest: dest_reg,
                        symbol: Symbol(sl_label_str.clone()),
                    });
                    result.push(Instruction::Arith(instructions::Arith {
                        op: instructions::ArithOp::Add,
                        dest: dest_reg,
                        left: dest_reg,
                        right: instructions::RValue::SymbolOffset(Symbol(sl_label_str.clone())),
                    }));
                    store_if_needed(&mut result, dest_loc, dest_reg);

                } else {
                    let scratch_register_1 = scratch_register_1.align(*width);
                    let scratch_register_2 = scratch_register_2.align(*width);
                    let dest_loc = allocator.location_of(dest, idx).unwrap();
                    let source_loc = allocator.location_of(source, idx).unwrap();

                    let dest_reg = empty_register(dest_loc, scratch_register_1);
                    let source_reg = load_if_needed(&mut result, source_loc, scratch_register_2, 0);

                    result.push(Instruction::Mov {
                        dest: dest_reg,
                        operand: source_reg.rvalue(),
                    });

                    store_if_needed(&mut result, dest_loc, dest_reg.align(*width));
                }
            }
            nodes::Ssa::Quadriplet(quad) => {
                let width = quad.width;

                let scratch_register_1 = scratch_register_1.align(width);
                let scratch_register_2 = scratch_register_2.align(width);
                let scratch_register_3 = scratch_register_3.align(width);

                let left_loc = allocator.location_of(&quad.left, idx).unwrap();
                let right_loc = allocator.location_of(quad.right.as_ref().unwrap(), idx);
                let dest_loc = allocator.location_of(&quad.dest, idx).unwrap();

                let rvalue = if let Some(right_loc) =  right_loc {
                    let right_reg = load_if_needed(&mut result, right_loc, scratch_register_2, 0);
                    right_reg.rvalue()
                } else {
                    if let Address::Constant(nodes::AddressConstant::Numeric(nc)) = quad.right.as_ref().unwrap() {
                        RValue::Immediate(*nc)
                    } else {
                        panic!("WTF?")
                    }
                };

                let left_reg = load_if_needed(&mut result, left_loc, scratch_register_1, 0);

                let dest_reg = empty_register(dest_loc, scratch_register_3);

                if quad.op.is_cmp() {
                    let cond_op = instructions::ConditionalCode::try_from_nodes_op(quad.op);
                    result.push(Instruction::Cmp {
                        left: left_reg,
                        right: rvalue,
                    });
                    result.push(Instruction::CondSet {
                        dest: dest_reg,
                        cond: cond_op,
                    });
                } else {
                    let mod_op = instructions::ArithOp::try_from_nodes_op(quad.op);
                    result.push(Instruction::Arith(instructions::Arith {
                        op: mod_op,
                        dest: dest_reg,
                        left: left_reg,
                        right: rvalue,
                    }));
                }

                store_if_needed(&mut result, dest_loc, dest_reg);
            }
            nodes::Ssa::Label(lab) => {
                result.push(Instruction::Label(lab.to_asm_label(func_name)));
            }
            nodes::Ssa::Branch {
                cond,
                true_target,
                false_target,
                width
            } => {
                let scratch_register_1 = scratch_register_1.align(*width);
                if let Address::Constant(nodes::AddressConstant::Numeric(nc)) = cond {
                    result.push(Instruction::Mov { dest: scratch_register_1, operand: instructions::RValue::Immediate(*nc) });
                    result.push(Instruction::Cmp {
                        left: scratch_register_1,
                        right: instructions::RValue::Immediate(1),
                    });
                } else {
                    let cond_loc = allocator.location_of(&cond, idx).unwrap();
                    let cond_register = load_if_needed(&mut result, cond_loc, scratch_register_1, 0);

                    result.push(Instruction::Cmp {
                        left: cond_register,
                        right: instructions::RValue::Immediate(1),
                    });
                }

                result.push(Instruction::Branch(instructions::Branch::cond_eq((
                    true_target.clone(),
                    func_name,
                ))));
                result.push(Instruction::Branch(instructions::Branch::cond_not_eq((
                    false_target.clone(),
                    func_name,
                ))));
            }
            nodes::Ssa::Return { value } => {
                if let Some((val, width)) = value {
                    let scratch_register_1 = scratch_register_1.align(*width);
                    if let nodes::Address::Constant(nodes::AddressConstant::Numeric(nc)) = val {
                        result.push(Instruction::Mov {
                            dest: Register::x0(*width), // dont care about contents at this point
                            operand: instructions::RValue::Immediate(*nc)
                        });
                    } else {
                        let val_loc = allocator.location_of(&val, idx).unwrap();
                        let val_register = load_if_needed(&mut result, val_loc, scratch_register_1, 0);
                        result.push(Instruction::Mov {
                            dest: Register::x0(*width), // dont care about contents at this point
                            operand: val_register.rvalue(),
                        });
                    }

                    result.push(Instruction::Branch(instructions::Branch::uncond(
                        instructions::Label(format!("return_{}", func_name)),
                    )))
                } else {
                    result.push(Instruction::Branch(instructions::Branch::uncond(
                        instructions::Label(format!("return_{}", func_name)),
                    )))
                }

            }
            nodes::Ssa::Jump(target) => {
                result.push(Instruction::Branch(instructions::Branch::uncond((
                    target.clone(),
                    func_name,
                ))));
            }

            nodes::Ssa::Call {
                dest,
                func,
                num_params: _,
                parameters
            } => {

                let non_variadic_parameters = parameters.iter().filter(|x| !x.is_variadic).collect::<Vec<_>>();
                let variadic_parameters = parameters.iter().filter(|x| x.is_variadic).collect::<Vec<_>>();

                let used_registers = allocator.used_registers_at(idx);

                let dynamic_stack_offset = alloc_stack_spills(&mut result, &used_registers) as i64;

                let allocated_variadic = if !variadic_parameters.is_empty() {
                    handle_variadic_params(&mut result, &allocator, idx, &variadic_parameters, scratch_register_1, dynamic_stack_offset)
                } else {
                    0
                };

                for p in non_variadic_parameters.iter() {
                    handle_param(&mut result, &allocator, p, idx, scratch_register_1, dynamic_stack_offset) 
                }

                match func {
                    nodes::Address::CompilerTemp(_) => {
                        let scratch_register_1 = scratch_register_1.align(Width::Long);

                        let func_loc = allocator.location_of(func, idx).unwrap();
                        let func_reg = load_if_needed(&mut result, func_loc, scratch_register_1, dynamic_stack_offset);

                        result.push(Instruction::Branch(
                            instructions::Branch::branch_link_register(func_reg),
                        ));
                    }
                    nodes::Address::Source(source) => {
                        result.push(Instruction::Branch(instructions::Branch::branch_link(
                            instructions::Label(format!("_{}", source.0)),
                        )));
                    }
                    nodes::Address::Constant(_) => {
                        panic!("Cannot call a constant! Unless maybe blr?");
                    }
                }

                if let Some((_, width)) = dest {
                    let scratch_register_1 = scratch_register_1.align(*width);

                    result.push(Instruction::Mov{
                        dest: scratch_register_1,
                        operand: Register::x0(*width).rvalue()
                    });
                }

                if allocated_variadic > 0 {
                    result.push(Instruction::Comment(format!("Variadic parameters pop")));
                    pop_stack(&mut result, allocated_variadic);
                }

                pop_stack_spills(&mut result, &used_registers);

                if let Some((val, width)) = dest {
                    let scratch_register_1 = scratch_register_1.align(*width);
                    let val_loc = allocator.location_of(val, idx).unwrap();
                    store_if_needed(&mut result, val_loc, scratch_register_1);
                }
            }

            nodes::Ssa::Phi(_) => panic!("Phi functions should be eliminated at this stage!"),
            _ => todo!(),
        }
    }

    result.push(instructions::Instruction::Label(format!(
        "return_{}",
        func_name
    )));

    pop_stack(&mut result, stack_size);

    result
}

pub fn convert_function_body_ir_to_asm(
    ir: &[nodes::Ssa],
    func_name: &str,
    parameters: &[(String, Width)],
    global_lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {
    let lookup = global_lookup;

    let mut instructions = vec![];

    instructions.push(instructions::Instruction::StorePair {
        r1: Register::frame_pointer(),
        r2: Register::link_register(),
        addressing: instructions::AddressingMode::pre_indexed(-16),
    });
    instructions.push(instructions::Instruction::Mov {
        dest: Register::frame_pointer(),
        operand: instructions::RValue::Register(Register::stack_pointer()),
    });

    let asm = body_to_asm(ir, func_name, parameters, lookup);
    instructions.extend(asm);

    instructions.push(instructions::Instruction::LoadPair {
        r1: Register::frame_pointer(),
        r2: Register::link_register(),
        addressing: instructions::AddressingMode::post_indexed(16),
    });
    instructions.push(instructions::Instruction::Branch(
        instructions::Branch::Return,
    ));
    instructions
}

pub fn convert_function_to_asm(
    fd: &nodes::FunctionDef,
    lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {
    let mut instructions = vec![];
    instructions.push(instructions::Instruction::Label(
        "_".to_owned() + fd.name.as_str(),
    ));
    instructions.extend(convert_function_body_ir_to_asm(&fd.body, &fd.name, &fd.parameters, lookup));

    instructions
}

pub fn convert_declaration_to_asm(
    dec: &nodes::ToplevelDeclaration,
) -> Vec<instructions::Instruction> {
    let mut instructions = vec![];
    match dec {
        nodes::ToplevelDeclaration::Function {
            storage_class,
            name,
            return_width: _,
            parameters: _,
        } => {
            if matches!(storage_class, StorageClass::Extern) {
                instructions.push(instructions::Instruction::Directive(
                    instructions::Directive::Extern(name.clone()),
                ));
            }
        }
    }
    instructions
}

pub fn convert_unit_to_asm(unit: &[nodes::ToplevelItem]) -> Vec<instructions::Instruction> {
    let lookup = SymbolLookup::global_from_unit(unit);
    let mut instructions = vec![];

    instructions.push(Instruction::Directive(instructions::Directive::Section(
        instructions::Section::Text,
    )));

    for tl in unit {
        match tl {
            nodes::ToplevelItem::Declaration(dec) => {
                instructions.extend(convert_declaration_to_asm(dec))
            }
            nodes::ToplevelItem::Function(f) => {
                // Todo: Static functions should not be exported
                instructions.push(Instruction::Directive(instructions::Directive::Global(
                    f.name.clone(),
                )));
            }
        }
    }

    for tl in unit {
        if let nodes::ToplevelItem::Function(f) = tl {
            instructions.extend(convert_function_to_asm(f, &lookup))
        }
    }

    instructions.push(Instruction::Directive(instructions::Directive::Section(
        instructions::Section::TextCstring,
    )));

    for (counter, strlit) in lookup.string_literals_iter() {
        instructions.push(instructions::Instruction::Label(format!("sl{}", counter)));
        instructions.push(instructions::Instruction::Directive(
            instructions::Directive::AsciiCString(strlit.to_owned()),
        ));
    }

    instructions
}

pub fn asm_into_text(instr: &[instructions::Instruction]) -> String {
    let mut res = String::new();
    for i in instr {
        res.push_str(&i.to_string());
        res.push('\n');
    }
    res
}
