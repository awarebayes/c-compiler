use std::rc::Rc;

use crate::{
    common::{StorageClass, Width},
    parsing::ast,
};

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Plus,
    Mul,
    Minus,
    Div,
    Gt,
    Lt,
    Eq,
}

impl Op {
    pub fn from_binop(binop: &ast::BinOp) -> Self {
        match binop {
            ast::BinOp::Mul => Op::Mul,
            ast::BinOp::Plus => Op::Plus,
            ast::BinOp::Minus => Op::Minus,
            ast::BinOp::Div => Op::Div,
            ast::BinOp::Gt => Op::Gt,
            ast::BinOp::Lt => Op::Lt,
            ast::BinOp::Eq => Op::Eq,
        }
    }

    pub fn is_cmp(&self) -> bool {
        match self {
            Self::Eq | Self::Lt | Self::Gt => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quadriplet {
    pub width: Width,
    pub dest: Address,
    pub op: Op,
    pub left: Address,
    pub right: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressConstant {
    Numeric(i64),
    StringLiteral(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Source((Rc<String>, usize)),
    CompilerTemp(usize),
    Constant(AddressConstant),
}

impl Address {
    pub fn source_count(s: String, count: usize) -> Self {
        Address::Source((Rc::new(s), count))
    }

    pub fn constant(c: AddressConstant) -> Self {
        Address::Constant(c)
    }

    pub fn compiler_temp(n: usize) -> Self {
        Address::CompilerTemp(n)
    }

    pub fn get_source(&self) -> &str {
        match self {
            Self::Source(s) => &s.0,
            _ => panic!("Not a source var"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Label {
    Source(Rc<String>),
    CompilerTemp(usize),
}

impl Label {
    pub fn source(s: String) -> Self {
        Label::Source(Rc::new(s))
    }
    pub fn compiler_temp(n: usize) -> Self {
        Label::CompilerTemp(n)
    }
}

#[derive(Debug, Clone)]
pub struct PhiFunction {
    pub dest: Address,
    pub width: Width,
    pub merging: Vec<(Address, Label)>,
}

#[derive(Debug, Clone)]
pub enum Ssa {
    // Quadriplet
    Quadriplet(Quadriplet),

    Assignment {
        dest: Address,
        source: Address,
        width: Width,
    },

    Phi(PhiFunction),

    // Function parameters: param value
    Param {
        number: usize,
        value: Address,
        width: Width,
    },

    // Function calls: dest = call func_name, num_params
    Call {
        dest: Option<(Address, Width)>,
        func: Address,
        num_params: usize,
    },

    // Return with optional value
    Return {
        value: Option<(Address, Width)>,
    },

    // Labels for basic blocks
    Label(Label),

    // Control flow
    Jump(Label),
    Branch {
        cond: Address,
        true_target: Label,
        false_target: Label,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub return_width: Width,
    pub parameters: Vec<(String, Width)>,
    pub body: Vec<Ssa>,
}

#[derive(Debug, Clone)]
pub enum ToplevelDeclaration {
    Function {
        storage_class: StorageClass,
        name: String,
        return_width: Width,
        parameters: Vec<Width>,
    },
}

impl ToplevelDeclaration {
    pub fn name(&self) -> String {
        match self {
            Self::Function {
                storage_class: _,
                name,
                return_width: _,
                parameters: _,
            } => name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToplevelItem {
    Function(FunctionDef),
    Declaration(ToplevelDeclaration),
}
