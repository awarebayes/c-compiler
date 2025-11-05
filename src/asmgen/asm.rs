use crate::asmgen::aarch64::instructions;
use crate::asmgen::aarch64::instructions::Instruction;
use crate::asmgen::aarch64::instructions::Register;
use crate::asmgen::aarch64::instructions::Symbol;
use crate::asmgen::regalloc::LinearScanRegisterAlloc;
use crate::asmgen::regalloc::analyze_lifetimes;
use crate::asmgen::lookup_table::{SymbolAddress, SymbolLookup};
use crate::common::StorageClass;
use crate::common::Width;
use crate::ir::ir_to_basic_blocks;
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
        Register::x5(Width::Long),
        Register::x6(Width::Long),
        Register::x7(Width::Long),
    ]);

    let mut result = vec![];
    for (idx, b) in block.iter().enumerate() {
        match b {
            nodes::Ssa::Assignment {
                dest,
                source,
                width,
            } => {
                if let nodes::Address::Constant(nodes::AddressConstant::Numeric(num_const)) = source
                {
                    let (dest_register, spill) = allocator.allocate_for_variable(dest.clone(), &lifetimes[&dest], idx);
                    assert!(spill.is_none());

                    result.push(Instruction::Mov {
                        dest: dest_register,
                        operand: instructions::RValue::Immediate(*num_const),
                    });
                } else if let nodes::Address::Constant(nodes::AddressConstant::StringLiteral(sl)) =
                    source
                {
                    let (dest_register, spill) = allocator.allocate_for_variable(dest.clone(), &lifetimes[&dest], idx);
                    assert!(spill.is_none());

                    let sl_label = lookup
                        .get(&nodes::Address::Constant(
                            nodes::AddressConstant::StringLiteral(sl.clone()),
                        ))
                        .unwrap();
                    let sl_label_str = match sl_label.address {
                        SymbolAddress::StringLiteral(sl_count) => format!("sl{}", sl_count),
                        _ => panic!(),
                    };

                    let x0 = Register::x0(*width);
                    result.push(Instruction::AdressPage {
                        dest: dest_register,
                        symbol: Symbol(sl_label_str.clone()),
                    });
                    result.push(Instruction::Arith(instructions::Arith {
                        op: instructions::ArithOp::Add,
                        dest: dest_register,
                        left: dest_register,
                        right: instructions::RValue::SymbolOffset(Symbol(sl_label_str.clone())),
                    }));

                } else {
                    let (dest_register, spill) = allocator.allocate_for_variable(dest.clone(), &lifetimes[&dest], idx);
                    assert!(spill.is_none());
                    let source_register = allocator.get_register(source).unwrap();

                    result.push(Instruction::Mov {
                        dest: dest_register,
                        operand: source_register.rvalue(),
                    });
                }
            }
            nodes::Ssa::Quadriplet(quad) => {
                let width = quad.width;

                let left_register = allocator.get_register(&quad.left).unwrap().align(width);
                let right_register = allocator.get_register(quad.right.as_ref().unwrap()).unwrap().align(width);

                let (result_register, spill) = allocator.allocate_for_variable(quad.dest.clone(), &lifetimes[&quad.dest], idx);
                let result_register = result_register.align(width);

                assert!(spill.is_none());


                if quad.op.is_cmp() {
                    let cond_op = instructions::ConditionalCode::try_from_nodes_op(quad.op);
                    result.push(Instruction::Cmp {
                        left: left_register,
                        right: right_register.rvalue(),
                    });
                    result.push(Instruction::CondSet {
                        dest: result_register,
                        cond: cond_op,
                    });
                } else {
                    let mod_op = instructions::ArithOp::try_from_nodes_op(quad.op);
                    result.push(Instruction::Arith(instructions::Arith {
                        op: mod_op,
                        dest: result_register,
                        left: left_register,
                        right: right_register.rvalue(),
                    }));
                }

                if allocator.is_spilled(&quad.dest) {
                    result.push(Instruction::Store {
                            width,
                            source: result_register,
                            operand: instructions::AddressingMode::stack_offset(allocator.get_spill_slot(&quad.dest).unwrap()),
                    });
                }

            }
            nodes::Ssa::Label(lab) => {
                result.push(Instruction::Label(lab.to_asm_label(func_name)));
            }
            nodes::Ssa::Branch {
                cond,
                true_target,
                false_target,
            } => {
                let cond_register = allocator.get_register(cond).unwrap();

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
                    let val_register = allocator.get_register(val).unwrap().align(*width);

                    result.push(Instruction::Mov {
                        dest: Register::x0(*width),
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
            nodes::Ssa::Param {
                number,
                value,
                width,
            } => {
                let reg = match *number {
                    0 => Register::x0(*width),
                    1 => Register::x1(*width),
                    2 => Register::x2(*width),
                    _ => todo!(),
                };
                result.push(Instruction::Load {
                    width: *width,
                    dest: reg,
                    operand: instructions::AddressingMode::stack_offset(var.address.offset()),
                });
            }
            nodes::Ssa::Call {
                dest,
                func,
                num_params: _,
            } => {
                match func {
                    nodes::Address::CompilerTemp(_) => {
                        let x0 = Register::x0(Width::Long);
                        let symb = lookup.get(func).unwrap();
                        result.push(Instruction::Load {
                            width: Width::Long,
                            dest: x0,
                            operand: instructions::AddressingMode::stack_offset(
                                symb.address.offset(),
                            ),
                        });
                        result.push(Instruction::Branch(
                            instructions::Branch::branch_link_register(x0),
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
                if let Some((val, width)) = dest {
                    let x0 = Register::x0(*width);
                    let val_info = lookup.get(&val).unwrap();
                    result.push(Instruction::Store {
                        width: *width,
                        source: x0,
                        operand: instructions::AddressingMode::stack_offset(
                            val_info.address.offset(),
                        ),
                    });
                }
            }

            nodes::Ssa::Phi(_) => panic!("Phi functions should be eliminated at this stage!"),
            _ => todo!(),
        }
    }
    result
}

pub fn convert_function_body_ir_to_asm(
    ir: &[nodes::Ssa],
    func_name: &str,
    global_lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {
    let lookup = SymbolLookup::from_fn_body(ir);

    let stack_size = lookup.stack_size();

    let lookup = lookup.extend_with_global(global_lookup.clone());

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

    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Sub,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(stack_size as i64),
    }));

    let asm = body_to_asm(ir, func_name);
    instructions.extend(asm);

    instructions.push(instructions::Instruction::Label(format!(
        "return_{}",
        func_name
    )));
    instructions.push(instructions::Instruction::Arith(instructions::Arith {
        op: instructions::ArithOp::Add,
        dest: Register::stack_pointer(),
        left: Register::stack_pointer(),
        right: instructions::RValue::Immediate(stack_size as i64),
    }));
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
