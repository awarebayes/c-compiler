use crate::asmgen::aarch64::instructions;
use crate::asmgen::aarch64::instructions::Instruction;
use crate::asmgen::aarch64::instructions::Register;
use crate::asmgen::aarch64::instructions::Symbol;
use crate::asmgen::regalloc;
use crate::asmgen::regalloc::LinearScanRegisterAlloc;
use crate::asmgen::regalloc::analyze_lifetimes;
use crate::asmgen::lookup_table::{SymbolAddress, SymbolLookup};
use crate::common::StorageClass;
use crate::common::Width;
use crate::ir::IrTextRepr;
use crate::ir::nodes;

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

fn load_if_needed(instructions: &mut Vec<Instruction>, loc: regalloc::Location, spill_load_register: Register) -> Register {
    match loc {
        regalloc::Location::Reg(r) => r.align(spill_load_register.width),
        regalloc::Location::Spill(stack_off) => {
            instructions.push(Instruction::Load {
                    width: spill_load_register.width,
                    dest: spill_load_register,
                    operand: instructions::AddressingMode::stack_offset(stack_off),
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

fn alloc_stack_spill(instructions: &mut Vec<Instruction>, reg: Register) {
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Sub,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(32), // FIXME MY MAN
    }));
    instructions.push(Instruction::Store {
        width: Width::Long,
        source: reg.align(Width::Long),
        operand: instructions::AddressingMode::stack_offset(0),
    });
}

fn pop_stack_spill(instructions: &mut Vec<Instruction>, reg: Register) {
    instructions.push(Instruction::Load {
        width: Width::Long,
        dest: reg.align(Width::Long),
        operand: instructions::AddressingMode::stack_offset(0),
    });
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Add,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(32),
    }));
}

fn handle_param(instructions: &mut Vec<Instruction>, allocator: &LinearScanRegisterAlloc, param: &nodes::FunctionParameter, idx: usize, scratch_register_1: Register) -> Option<Register> {
    assert!(!param.is_variadic);
    let mut spill = None;
    let width = param.width;
    let arg_reg = match param.number {
        0 => Register::x0(width),
        1 => Register::x1(width),
        2 => Register::x2(width),
        _ => todo!(),
    };

    let scratch_register_1 = scratch_register_1.align(width);
    let param_loc = allocator.location_of(&param.value, idx).unwrap();
    let param_reg = load_if_needed(instructions, param_loc, scratch_register_1);

    // Maybe check if arg_reg is empty here?
    if arg_reg != param_reg {
        // Something was in arg_reg, store this something to stack
        spill = Some(arg_reg.align(Width::Long));
        alloc_stack_spill(instructions, arg_reg.align(Width::Long));

        instructions.push(Instruction::Mov {
            dest: arg_reg,
            operand: param_reg.rvalue(),
        });
    }

    return spill;
}


fn handle_variadic_params(instructions: &mut Vec<Instruction>, allocator: &LinearScanRegisterAlloc, idx: usize, params: &[&nodes::FunctionParameter], scratch_register_1: Register) -> usize {
    const APPLE_VARARG_SLOT_SIZE: usize = 8;
    let allocated = (APPLE_VARARG_SLOT_SIZE * params.len()).next_multiple_of(16);

    alloc_stack(instructions, allocated);

    for (param_idx, param) in params.iter().enumerate() {
        let param_loc =  allocator.location_of(&param.value, idx).unwrap();
        let width = param.width;
        let scratch_register_1 = scratch_register_1.align(width);
        let param_reg = load_if_needed(instructions, param_loc, scratch_register_1);

        instructions.push(Instruction::Store {
            width: width,
            source: param_reg.align(width),
            operand: instructions::AddressingMode::stack_offset(8 * param_idx as i64)
        });

    }

    allocated
}

fn body_to_asm(
    block: &[nodes::Ssa],
    func_name: &str,
    lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {

    let lifetimes = analyze_lifetimes(block);

    let mut allocator = LinearScanRegisterAlloc::new(vec![
        Register::x0(Width::Long),
        Register::x1(Width::Long),
        Register::x2(Width::Long),
        Register::x3(Width::Long),
        Register::x4(Width::Long),
    ]);

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

                    result.push(Instruction::Mov {
                        dest: scratch_register_1.align(*width),
                        operand: instructions::RValue::Immediate(*num_const),
                    });

                    store_if_needed(&mut result, dest_loc, scratch_register_1.align(*width));
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
                    let source_reg = load_if_needed(&mut result, source_loc, scratch_register_2);

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
                let right_loc = allocator.location_of(quad.right.as_ref().unwrap(), idx).unwrap();
                let dest_loc = allocator.location_of(&quad.dest, idx).unwrap();

                let left_reg = load_if_needed(&mut result, left_loc, scratch_register_1);
                let right_reg = load_if_needed(&mut result, right_loc, scratch_register_2);
                let dest_reg = empty_register(dest_loc, scratch_register_3);

                if quad.op.is_cmp() {
                    let cond_op = instructions::ConditionalCode::try_from_nodes_op(quad.op);
                    result.push(Instruction::Cmp {
                        left: left_reg,
                        right: right_reg.rvalue(),
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
                        right: right_reg.rvalue(),
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
                let cond_loc = allocator.location_of(&cond, idx).unwrap();
                let cond_register = load_if_needed(&mut result, cond_loc, scratch_register_1);

                result.push(Instruction::Cmp {
                    left: cond_register,
                    right: instructions::RValue::Immediate(1),
                });
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
                    let val_loc = allocator.location_of(&val, idx).unwrap();
                    let val_register = load_if_needed(&mut result, val_loc, scratch_register_1);

                    result.push(Instruction::Mov {
                        dest: Register::x0(*width), // dont care about contents at this point
                        operand: val_register.rvalue(),
                    });
                    result.push(Instruction::Branch(instructions::Branch::uncond(
                        instructions::Label(format!("return_{}", func_name)),
                    )))
                }

                result.push(Instruction::Branch(instructions::Branch::uncond(
                    instructions::Label(format!("return_{}", func_name)),
                )))
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

                let mut call_stack_spills: Vec<Register> = vec![];

                let non_variadic_parameters = parameters.iter().filter(|x| !x.is_variadic).collect::<Vec<_>>();
                let variadic_parameters = parameters.iter().filter(|x| x.is_variadic).collect::<Vec<_>>();

                for p in non_variadic_parameters.iter() {
                    if let Some(spill) = handle_param(&mut result, &allocator, p, idx, scratch_register_1) {
                        call_stack_spills.push(spill);
                    }
                }

                let allocated_variadic = if !variadic_parameters.is_empty() {
                    handle_variadic_params(&mut result, &allocator, idx, &variadic_parameters, scratch_register_1)
                } else {
                    0
                };

                match func {
                    nodes::Address::CompilerTemp(_) => {
                        let scratch_register_1 = scratch_register_1.align(Width::Long);

                        let func_loc = allocator.location_of(func, idx).unwrap();
                        let func_reg = load_if_needed(&mut result, func_loc, scratch_register_1);

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

                if allocated_variadic > 0 {
                    pop_stack(&mut result, allocated_variadic);
                }

                if let Some((val, width)) = dest {
                    let scratch_register_1 = scratch_register_1.align(*width);

                    result.push(Instruction::Mov{
                        dest: scratch_register_1,
                        operand: Register::x0(*width).rvalue()
                    });

                    let val_loc = allocator.location_of(val, idx).unwrap();

                    store_if_needed(&mut result, val_loc, scratch_register_1);
                }

                for spilled_reg in call_stack_spills.iter().rev() {
                    pop_stack_spill(&mut result, *spilled_reg);
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

    let asm = body_to_asm(ir, func_name, lookup);
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
    instructions.extend(convert_function_body_ir_to_asm(&fd.body, &fd.name, lookup));

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
