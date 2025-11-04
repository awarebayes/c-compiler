use crate::asmgen::aarch64::instructions;
use crate::asmgen::aarch64::instructions::Instruction;
use crate::asmgen::aarch64::instructions::Register;
use crate::asmgen::aarch64::instructions::Symbol;
use crate::asmgen::lookup_table::{SymbolAddress, SymbolLookup};
use crate::common::StorageClass;
use crate::common::Width;
use crate::ir::nodes;

type BasicBlock = Vec<nodes::Ssa>;

// fn address_to_asm_str(adress: &nodes::Address, lookup: &SymbolLookup) -> String {
//     match address 
// }

impl nodes::Label {
    fn to_asm_label(&self) -> String {
        match self {
            Self::CompilerTemp(ct) => format!("L{}", ct),
            Self::Source(s) => s.clone(),
        }
    }
}

fn basic_block_to_asm(
    block: &[nodes::Ssa],
    func_name: &str,
    lookup: &SymbolLookup,
) -> Vec<instructions::Instruction> {
    let mut result = vec![];
    for b in block {
        match b {
            nodes::Ssa::Assignment {
                dest,
                source,
                width,
            } => {
                if let nodes::Address::Constant(nodes::AddressConstant::Numeric(num_const)) = source
                {
                    let result_var = lookup.get(dest).unwrap();
                    result.push(Instruction::Mov {
                        dest: Register::x0(*width),
                        operand: instructions::RValue::Immediate(*num_const),
                    });
                    result.push(Instruction::Store {
                        width: *width,
                        source: Register::x0(*width),
                        operand: instructions::AddressingMode::stack_offset( result_var.address.offset() ),
                    });
                } else if let nodes::Address::Constant(nodes::AddressConstant::StringLiteral(sl)) = source {
                    let result_var = lookup.get(dest).unwrap();
                    let sl_label = lookup.get(&nodes::Address::Constant(nodes::AddressConstant::StringLiteral(sl.clone()))).unwrap();
                    let sl_label_str = match sl_label.address {
                        SymbolAddress::StringLiteral(sl_count) => format!("sl{}", sl_count),
                        _ => panic!(),
                    };

                    let x0 = Register::x0(*width);
                    result.push(Instruction::AdressPage { dest: x0, symbol: Symbol(sl_label_str.clone())});
                    result.push(Instruction::Arith( instructions::Arith { op: instructions::ArithOp::Add, dest: x0, left: x0, right:  instructions::RValue::SymbolOffset(Symbol(sl_label_str.clone())) } ));
                    result.push(Instruction::Store {
                        width: *width,
                        source: Register::x0(*width),
                        operand: instructions::AddressingMode::stack_offset( result_var.address.offset() ),
                    });
                } else {
                    let dest_var = lookup.get(dest).unwrap();
                    let source_var = lookup.get(source).unwrap();
                    result.push(Instruction::Load { width: *width, dest: Register::x0(*width), operand: instructions::AddressingMode::stack_offset(source_var.address.offset()) });
                    result.push(Instruction::Store {
                        width: *width,
                        source: Register::x0(*width),
                        operand: instructions::AddressingMode::stack_offset( dest_var.address.offset() ),
                    });
                }
            },
            nodes::Ssa::Quadriplet(quad) => {
                let width = quad.width;
                let left_var = lookup.get(&quad.left).unwrap();
                let right_var = lookup.get(quad.right.as_ref().unwrap()).unwrap();
                let dest_var = lookup.get(&quad.dest).unwrap();

                let x0 = Register::x0(width);
                let x1 = Register::x1(width);

                result.push(Instruction::Load { width, dest: x0, operand: instructions::AddressingMode::stack_offset(left_var.address.offset()) });
                result.push(Instruction::Load { width, dest: x1, operand: instructions::AddressingMode::stack_offset(right_var.address.offset()) });

                if quad.op.is_cmp() {
                    let cond_op = instructions::ConditionalCode::try_from_nodes_op(quad.op);
                    result.push(Instruction::Cmp { left: x0, right: x1.rvalue() });
                    result.push(Instruction::CondSet { dest: x0, cond: cond_op});
                } else {
                    let mod_op = instructions::ArithOp::try_from_nodes_op(quad.op);
                    result.push(Instruction::Arith( instructions::Arith { op: mod_op, dest: x0, left: x0, right: x1.rvalue() } ));
                }

                result.push(Instruction::Store { width, source: x0, operand: instructions::AddressingMode::stack_offset(dest_var.address.offset()) });
            },
            nodes::Ssa::Label(lab) => {
                result.push(Instruction::Label(lab.to_asm_label()));
            },
            nodes::Ssa::Branch { cond, true_target, false_target } => {
                let cond_var = lookup.get(&cond).unwrap();
                let width = cond_var.width;
                let x0 = Register::x0(width);
                result.push(Instruction::Load { width, dest: x0, operand: instructions::AddressingMode::stack_offset(cond_var.address.offset()) });
                result.push(Instruction::Cmp { left: x0, right: instructions::RValue::Immediate(1) });
                result.push(Instruction::Branch(instructions::Branch::cond_eq(true_target.clone())));
                result.push(Instruction::Branch(instructions::Branch::cond_not_eq(false_target.clone())));
            },
            nodes::Ssa::Return { value } => {
                if let Some((val, width)) = value {
                    let x0 = Register::x0(*width);
                    let val_info = lookup.get(&val).unwrap();
                    result.push(Instruction::Load { width: *width, dest: x0, operand: instructions::AddressingMode::stack_offset(val_info.address.offset()) });
                    result.push(Instruction::Branch(instructions::Branch::uncond(instructions::Label(format!("return_{}", func_name)))))
                } 

                result.push(Instruction::Branch(instructions::Branch::uncond(instructions::Label(format!("return_{}", func_name)))))
            },
            nodes::Ssa::Jump(target) => {
                result.push(Instruction::Branch(instructions::Branch::uncond(target.clone())));
            },
            nodes::Ssa::Param { number, value, width } => {
                let reg = match *number {
                    0 => Register::x0(*width),
                    1 => Register::x1(*width),
                    2 => Register::x2(*width),
                    _ => todo!(),
                };
                let var = lookup.get(value).unwrap();
                result.push(Instruction::Load { width: *width, dest: reg, operand: instructions::AddressingMode::stack_offset(var.address.offset()) });
            },
            nodes::Ssa::Call { dest, func, num_params: _ } => {
                match func {
                    nodes::Address::CompilerTemp(_) => {
                        let x0 = Register::x0(Width::Long);
                        let symb = lookup.get(func).unwrap();
                        result.push(Instruction::Load { width: Width::Long, dest: x0, operand: instructions::AddressingMode::stack_offset(symb.address.offset()) });
                        result.push(Instruction::Branch(instructions::Branch::branch_link_register(x0)));
                    }, 
                    nodes::Address::Source(source) => {
                        result.push(Instruction::Branch(instructions::Branch::branch_link(instructions::Label("_".to_string() + source.as_str()))));
                    },
                    nodes::Address::Constant(_)  => {
                        panic!("Cannot call a constant! Unless maybe blr?");
                    }
                }
                if let Some((val, width)) = dest {
                    let x0 = Register::x0(*width);
                    let val_info = lookup.get(&val).unwrap();
                    result.push(Instruction::Store { width: *width, source: x0, operand: instructions::AddressingMode::stack_offset(val_info.address.offset()) });
                }
            }

            _ => todo!(),
        }
    }
    result
}

fn ir_to_basic_blocks(ir: &[nodes::Ssa]) -> Vec<BasicBlock> {
    let mut blocks = vec![];
    let mut current_block = vec![];

    for node in ir {
        match node {
            nodes::Ssa::Label(_) |
            nodes::Ssa::Return { value: _ } => {
                if !current_block.is_empty() {
                    blocks.push(current_block);
                    current_block = vec![];
                }
            }
            _ => (),
        }

        current_block.push(node.clone());
    }

    blocks
}

pub fn convert_function_body_ir_to_asm(ir: &[nodes::Ssa], func_name: &str, global_lookup: &SymbolLookup) -> Vec<instructions::Instruction> {
    let lookup = SymbolLookup::from_fn_body(ir);

    let stack_size = lookup.stack_size();

    let lookup = lookup.extend_with_global(global_lookup.clone());

    let blocks = ir_to_basic_blocks(ir);
    let mut instructions = vec![];

    instructions.push(instructions::Instruction::StorePair { r1: Register::frame_pointer(), r2: Register::link_register(), addressing: instructions::AddressingMode::pre_indexed(-16) });
    instructions.push(instructions::Instruction::Mov { dest: Register::frame_pointer(), operand: instructions::RValue::Register(Register::stack_pointer()) });

    instructions.push(instructions::Instruction::Arith( instructions::Arith { op: instructions::ArithOp::Sub, dest: Register::stack_pointer(), left: Register::stack_pointer(), right: instructions::RValue::Immediate(stack_size as i64) } ));

    for block in blocks.iter() {
        let asm = basic_block_to_asm(block, func_name, &lookup);
        instructions.extend(asm);
    }

    instructions.push(instructions::Instruction::Label( format!("return_{}", func_name)));
    instructions.push(instructions::Instruction::Arith( instructions::Arith { op: instructions::ArithOp::Add, dest: Register::stack_pointer(), left: Register::stack_pointer(), right: instructions::RValue::Immediate(stack_size as i64) } ));
    instructions.push(instructions::Instruction::LoadPair { r1: Register::frame_pointer(), r2: Register::link_register(), addressing: instructions::AddressingMode::post_indexed(16) });
    instructions.push(instructions::Instruction::Branch(instructions::Branch::Return));
    instructions
}

pub fn convert_function_to_asm(fd: &nodes::FunctionDef,  lookup: &SymbolLookup) -> Vec<instructions::Instruction> {
    let mut instructions = vec![];
    instructions.push(instructions::Instruction::Label( "_".to_owned() + fd.name.as_str() ));
    instructions.extend(convert_function_body_ir_to_asm(&fd.body, &fd.name, lookup));

    instructions
}

pub fn convert_declaration_to_asm(dec: &nodes::ToplevelDeclaration) -> Vec<instructions::Instruction> {
    let mut instructions = vec![];
    match dec {
        nodes::ToplevelDeclaration::Function { storage_class, name, return_width: _, parameters: _ } => {
            if matches!(storage_class, StorageClass::Extern) {
                instructions.push(instructions::Instruction::Directive( instructions::Directive::Extern( name.clone() ) ));
            }
        },
    }
    instructions
}

pub fn convert_unit_to_asm(unit: &[nodes::ToplevelItem]) -> Vec<instructions::Instruction> {
    let lookup = SymbolLookup::global_from_unit(unit);
    let mut instructions = vec![];

    instructions.push(Instruction::Directive(instructions::Directive::Section(instructions::Section::Text)));

    for tl in unit {
        match tl {
            nodes::ToplevelItem::Declaration(dec) => {
                instructions.extend(convert_declaration_to_asm(dec))
            },
            nodes::ToplevelItem::Function(f) => {
                // Todo: Static functions should not be exported
                instructions.push(Instruction::Directive(instructions::Directive::Global(f.name.clone())));
            }
        } 
    }

    for tl in unit {
        if let nodes::ToplevelItem::Function(f) = tl {
            instructions.extend(convert_function_to_asm(f, &lookup))
        }
    }

    instructions.push(Instruction::Directive(instructions::Directive::Section(instructions::Section::TextCstring)));

    for (counter, strlit) in lookup.string_literals_iter() {
        instructions.push(instructions::Instruction::Label(format!("sl{}", counter)));
        instructions.push(instructions::Instruction::Directive(
            instructions::Directive::AsciiCString(strlit.to_owned())
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