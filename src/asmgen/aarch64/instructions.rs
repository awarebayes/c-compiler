use crate::{common::Width, ir::nodes};

#[derive(Debug, Clone, Copy)]
pub enum FunctionArgumentRegister {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
}

#[derive(Debug, Clone, Copy)]
pub enum CorruptibleRegister {
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
}

#[derive(Debug, Clone, Copy)]
pub enum CalleeSavedRegister {
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterKind {
    FunctionArgument(FunctionArgumentRegister), // x0 - x7
    IndirectResult,                             // x8
    Corruptuble(CorruptibleRegister),           // x9-x15
    IP0,                                        // x16
    IP1,                                        // x17
    PR,                                         // x18
    CalleeSaved(CalleeSavedRegister),           // x19-x28
    FramePointer,                               // x29
    LinkRegister,                               //x30
    StackPointer,                               // sp
}

impl FunctionArgumentRegister {
    fn to_gp_num(&self) -> usize {
        match self {
            Self::X0 => 0,
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X3 => 3,
            Self::X4 => 4,
            Self::X5 => 5,
            Self::X6 => 6,
            Self::X7 => 7,
        }
    }
}

impl RegisterKind {
    fn to_gp_num(&self) -> Option<usize> {
        match self {
            Self::FunctionArgument(fa) => Some(fa.to_gp_num()),
            Self::StackPointer => None,
            Self::FramePointer => Some(29),
            Self::LinkRegister => Some(30),
            _ => todo!(),
        }
    }
}

impl From<FunctionArgumentRegister> for RegisterKind {
    fn from(value: FunctionArgumentRegister) -> Self {
        Self::FunctionArgument(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    pub kind: RegisterKind,
    pub width: Width,
}

impl Register {

    pub fn align(&self, width: Width) -> Self {
        Register { kind: self.kind, width }
    }

    pub fn x0(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X0),
            width,
        }
    }

    pub fn x1(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X1),
            width,
        }
    }

    pub fn x2(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X2),
            width,
        }
    }

    pub fn x3(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X3),
            width,
        }
    }

    pub fn x4(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X4),
            width,
        }
    }

    pub fn x5(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X5),
            width,
        }
    }

    pub fn x6(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X6),
            width,
        }
    }

    pub fn x7(width: Width) -> Self {
        Register {
            kind: RegisterKind::FunctionArgument(FunctionArgumentRegister::X7),
            width,
        }
    }

    pub fn frame_pointer() -> Self {
        Register {
            kind: RegisterKind::FramePointer,
            width: Width::Long,
        }
    }

    pub fn link_register() -> Self {
        Register {
            kind: RegisterKind::LinkRegister,
            width: Width::Long,
        }
    }

    pub fn stack_pointer() -> Self {
        Register {
            kind: RegisterKind::StackPointer,
            width: Width::Long,
        }
    }

    pub fn addressing_mode(self) -> AddressingMode {
        AddressingMode::BaseRegister(self)
    }

    pub fn rvalue(self) -> RValue {
        RValue::Register(self)
    }
}

pub enum AddressingMode {
    BaseRegister(Register),
    Offset((Register, i64)),
    PreIndexed((Register, i64)),
    PostIndexed((Register, i64)),
}

impl AddressingMode {
    pub fn stack_offset(off: i64) -> AddressingMode {
        AddressingMode::Offset((Register::stack_pointer(), off))
    }
    pub fn pre_indexed(off: i64) -> AddressingMode {
        AddressingMode::PreIndexed((Register::stack_pointer(), off))
    }
    pub fn post_indexed(off: i64) -> AddressingMode {
        AddressingMode::PostIndexed((Register::stack_pointer(), off))
    }
}

pub enum RValue {
    Register(Register),
    Immediate(i64),
    SymbolOffset(Symbol),
}

pub struct Symbol(pub String);
pub struct Label(pub String);

impl From<(nodes::Label, &str)> for Label {
    fn from(value: (nodes::Label, &str)) -> Self {
        match value {
            (nodes::Label::CompilerTemp(ct), fname) => Label(format!("L_{}_{}", fname, ct)),
            (nodes::Label::Source(s), _) => Label(s.as_str().to_owned()),
        }
    }
}

pub enum CondBranch {
    Equal,
    NotEqual,
}
pub enum Branch {
    Unconditional(Label),
    BranchLink(Label),
    BranchLinkRegister(Register),
    Return,
    Cond((CondBranch, Label)),
}

impl Branch {
    pub fn cond_eq(label: impl Into<Label>) -> Branch {
        Branch::Cond((CondBranch::Equal, label.into()))
    }

    pub fn cond_not_eq(label: impl Into<Label>) -> Branch {
        Branch::Cond((CondBranch::NotEqual, label.into()))
    }

    pub fn uncond(label: impl Into<Label>) -> Branch {
        Branch::Unconditional(label.into())
    }

    pub fn branch_link(label: impl Into<Label>) -> Branch {
        Branch::BranchLink(label.into())
    }

    pub fn branch_link_register(reg: Register) -> Branch {
        Branch::BranchLinkRegister(reg)
    }
}

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl ArithOp {
    pub fn try_from_nodes_op(op: nodes::Op) -> Self {
        match op {
            nodes::Op::Div => ArithOp::Div,
            nodes::Op::Plus => ArithOp::Add,
            nodes::Op::Mul => ArithOp::Mul,
            nodes::Op::Minus => ArithOp::Sub,
            _ => todo!(),
        }
    }
}

pub struct Arith {
    pub op: ArithOp,
    pub dest: Register,
    pub left: Register,
    pub right: RValue,
}

pub enum ConditionalCode {
    Eq,
    Ne,
    SignedLessThan,
    SignedGreaterThan,
}

impl ConditionalCode {
    pub fn try_from_nodes_op(op: nodes::Op) -> Self {
        match op {
            nodes::Op::Eq => ConditionalCode::Eq,
            nodes::Op::Lt => ConditionalCode::SignedLessThan,
            nodes::Op::Gt => ConditionalCode::SignedGreaterThan, // Todo: Add unsigned
            _ => todo!(),
        }
    }
}

pub enum Section {
    Text,
    TextCstring,
}

pub enum Directive {
    Section(Section),
    Extern(String),
    Global(String),
    AsciiCString(String),
}

pub enum Instruction {
    Directive(Directive),

    Label(String),

    StorePair {
        r1: Register,
        r2: Register,
        addressing: AddressingMode,
    },
    LoadPair {
        r1: Register,
        r2: Register,
        addressing: AddressingMode,
    },
    Mov {
        dest: Register,
        operand: RValue,
    },

    Cmp {
        left: Register,
        right: RValue,
    },
    CondSet {
        dest: Register,
        cond: ConditionalCode,
    },

    Load {
        width: Width,
        dest: Register,
        operand: AddressingMode,
    },

    Store {
        width: Width,
        source: Register,
        operand: AddressingMode,
    },

    Branch(Branch),
    Arith(Arith),
    AdressPage {
        dest: Register,
        symbol: Symbol,
    },
}

impl ArithOp {
    pub fn to_instr_string(&self) -> &str {
        match self {
            Self::Add => "add",
            Self::Div => "div",
            Self::Mul => "mul",
            Self::Sub => "sub",
        }
    }
}

impl Register {
    fn to_string(&self) -> String {
        match self.kind {
            RegisterKind::StackPointer => "sp".into(),
            _ => {
                let prefix = match self.width {
                    Width::Byte => "w",
                    Width::Short => "w",
                    Width::Word => "w",
                    Width::Long => "x",
                };
                let num = self.kind.to_gp_num().unwrap();
                format!("{}{}", prefix, num)
            }
        }
    }
}

impl RValue {
    fn to_string(&self) -> String {
        match self {
            Self::Immediate(c) => c.to_string(),
            Self::Register(r) => r.to_string(),
            Self::SymbolOffset(symb) => format!("{}@PAGEOFF", symb.0),
        }
    }
}

impl AddressingMode {
    fn to_string(&self) -> String {
        match self {
            Self::BaseRegister(br) => format!("[{}]", br.to_string()),
            Self::Offset((br, off)) => format!("[{}, {}]", br.to_string(), off),
            Self::PreIndexed((br, off)) => format!("[{}, {}]!", br.to_string(), off),
            Self::PostIndexed((br, off)) => format!("[{}], {}", br.to_string(), off),
        }
    }
}

impl ConditionalCode {
    fn to_string(&self) -> &str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::SignedGreaterThan => "gt",
            Self::SignedLessThan => "lt",
        }
    }
}

impl Branch {
    fn to_string(&self) -> String {
        match self {
            Self::Cond((CondBranch::Equal, label)) => {
                format!("beq {}", label.0)
            }

            Self::Cond((CondBranch::NotEqual, label)) => {
                format!("bne {}", label.0)
            }
            Self::BranchLink(label) => {
                format!("bl {}", label.0)
            }
            Self::BranchLinkRegister(reg) => {
                format!("bl {}", reg.to_string())
            }
            Self::Return => "ret".into(),
            Self::Unconditional(label) => {
                format!("b {}", label.0)
            }
        }
    }
}

impl Directive {
    fn to_string(&self) -> String {
        match self {
            Self::Extern(symbol_name) => format!(".extern _{}", symbol_name),
            Self::Global(symbol_name) => format!(".globl _{}", symbol_name),
            Self::AsciiCString(symbol_name) => format!(".asciz \"{}\"", symbol_name),
            Self::Section(Section::Text) => ".section __TEXT,__text".into(),
            Self::Section(Section::TextCstring) => ".section __TEXT,__cstring".into(),
        }
    }
}

impl Instruction {
    pub fn to_string(&self) -> String {
        match self {
            Self::Mov { dest, operand } => {
                format!("mov {}, {}", dest.to_string(), operand.to_string())
            }
            Self::Load {
                width,
                dest,
                operand,
            } => {
                let instruction_name = match width {
                    Width::Byte => "ldrb",
                    Width::Short => "ldrh",
                    Width::Word => "ldr",
                    Width::Long => "ldr",
                };

                format!(
                    "{} {}, {}",
                    instruction_name,
                    dest.to_string(),
                    operand.to_string()
                )
            }

            Self::Store {
                width,
                source: dest,
                operand,
            } => {
                let instruction_name = match width {
                    Width::Byte => "strb",
                    Width::Short => "strh",
                    Width::Word => "str",
                    Width::Long => "str",
                };

                format!(
                    "{} {}, {}",
                    instruction_name,
                    dest.to_string(),
                    operand.to_string()
                )
            }

            Self::Cmp { left, right } => {
                format!("cmp {}, {}", left.to_string(), right.to_string())
            }

            Self::Arith(ar) => {
                let arith_instr = ar.op.to_instr_string();
                format!(
                    "{} {}, {}, {}",
                    arith_instr,
                    ar.dest.to_string(),
                    ar.left.to_string(),
                    ar.right.to_string()
                )
            }

            Self::CondSet { dest, cond } => {
                format!("cset {}, {}", dest.to_string(), cond.to_string())
            }
            Self::Label(lab) => {
                format!("{}:", lab)
            }
            Self::Branch(b) => b.to_string(),
            Self::StorePair { r1, r2, addressing } => {
                format!(
                    "stp {}, {}, {}",
                    r1.to_string(),
                    r2.to_string(),
                    addressing.to_string()
                )
            }
            Self::LoadPair { r1, r2, addressing } => {
                format!(
                    "ldp {}, {}, {}",
                    r1.to_string(),
                    r2.to_string(),
                    addressing.to_string()
                )
            }
            Self::AdressPage { dest, symbol } => {
                format!("adrp {}, {}@PAGE", dest.to_string(), symbol.0)
            }
            Self::Directive(dir) => dir.to_string(),
            _ => todo!(),
        }
    }
}
