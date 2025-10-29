use crate::common::StorageClass;

#[derive(Debug)]
pub enum FunctionDeclaratorField {
    FunctionDeclarator(FunctionDeclarator),
    PointerDeclarator(PointerDeclarator),
}

impl FunctionDeclaratorField {
    pub fn parameters(&self) -> impl IntoIterator<Item = &FunctionParameter> {
        match self {
            Self::FunctionDeclarator(fd) => fd.parameters.iter(),
            Self::PointerDeclarator(pd) => {
                let mut current = &pd.declarator;
                loop {
                    match current.as_ref() {
                        Declarator::FunctionDeclarator(fd) => {
                            return fd.parameters.iter();
                        }
                        Declarator::PointerDeclarator(pd) => current = &pd.declarator,
                        _ => panic!("Wrong synax"),
                    }
                }
            }
        }
    }

    pub fn get_identifier(&self) -> Identifier {
        match self {
            Self::FunctionDeclarator(fd) => fd.declarator.get_identifier(),
            Self::PointerDeclarator(pd) => pd.declarator.get_identifier(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub return_type: DataType,
    pub declarator: FunctionDeclaratorField,
    pub body: CompoundStatement,
}

#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub struct StringLiteral(pub String);

#[derive(Debug)]
pub struct NumberLiteral(pub String);

#[derive(Debug)]
pub struct ParameterDeclaration {
    pub dtype: DataType,
    pub declarator: Box<Declarator>,
}

#[derive(Debug)]
pub enum FunctionParameter {
    ParameterDeclaration(ParameterDeclaration),
    VariadicParameter,
}

#[derive(Debug)]
pub struct FunctionDeclarator {
    pub declarator: Box<Declarator>,
    pub parameters: Vec<FunctionParameter>,
}

#[derive(Debug)]
pub struct PointerDeclarator {
    pub declarator: Box<Declarator>,
}

impl PointerDeclarator {
    pub fn get_nest_level(&self) -> usize {
        let mut level = 1;
        let mut current = &self.declarator;
        loop {
            match current.as_ref() {
                Declarator::FunctionDeclarator(function_declarator) => {
                    current = &function_declarator.declarator;
                }
                Declarator::PointerDeclarator(pointer_declarator) => {
                    current = &pointer_declarator.declarator;
                    level += 1;
                }
                Declarator::Identifier(_) => {
                    break;
                }
                Declarator::InitDeclarator(_) => panic!("Syntax"),
            }
        }
        level
    }
}

#[derive(Debug)]
pub struct InitDeclarator {
    pub declarator: Box<Declarator>,
    pub value: Expression,
}

#[derive(Debug)]
pub enum Declarator {
    FunctionDeclarator(FunctionDeclarator),
    PointerDeclarator(PointerDeclarator),
    Identifier(Identifier),
    InitDeclarator(InitDeclarator),
}

impl Declarator {
    pub fn get_identifier(&self) -> Identifier {
        match self {
            Self::FunctionDeclarator(fd) => fd.declarator.get_identifier(),
            Self::Identifier(i) => i.clone(),
            Self::PointerDeclarator(i) => i.declarator.get_identifier(),
            Self::InitDeclarator(i) => i.declarator.get_identifier(),
        }
    }
}

#[derive(Debug)]
pub enum DataType {
    Int,
    Char,
}

#[derive(Debug)]
pub struct CallExpression {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

impl CallExpression {
    pub fn get_identifier(&self) -> Option<Identifier> {
        match self.function.as_ref() {
            Expression::Identifier(id) => Some(id.clone()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum BinOp {
    Plus,
    Mul,
    Minus,
    Div,
    Gt,
    Lt,
    Eq,
}

impl BinOp {
    pub fn from_str(value: &str) -> Self {
        match value {
            "+" => BinOp::Plus,
            "*" => BinOp::Mul,
            "-" => BinOp::Minus,
            "/" => BinOp::Div,
            ">" => BinOp::Gt,
            "<" => BinOp::Lt,
            "==" => BinOp::Eq,
            _ => panic!("Unsupported bin op: {}", value),
        }
    }
}

#[derive(Debug)]
pub struct ExpressionBinary {
    pub left: Box<Expression>,
    pub op: BinOp,
    pub right: Box<Expression>,
}

#[derive(Debug)]
pub struct ParenthesizedExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug)]
pub enum AssignmentType {
    Eq,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
}

impl AssignmentType {
    pub fn from_str(inp: &str) -> Self {
        match inp {
            "=" => AssignmentType::Eq,
            "+=" => AssignmentType::AddEq,
            "-=" => AssignmentType::SubEq,
            "*=" => AssignmentType::MulEq,
            "/=" => AssignmentType::DivEq,
            _ => todo!(),
        }
    }

    pub fn to_op(&self) -> Option<BinOp> {
        match self {
            Self::Eq => None,
            Self::AddEq => Some(BinOp::Plus),
            Self::SubEq => Some(BinOp::Minus),
            Self::MulEq => Some(BinOp::Mul),
            Self::DivEq => Some(BinOp::Div),
        }
    }
}

#[derive(Debug)]
pub enum LValue {
    Identifier(Identifier),
}

#[derive(Debug)]
pub struct AssignmentExpression {
    pub lvalue: LValue,
    pub rvalue: Box<Expression>,
    pub atype: AssignmentType,
}

#[derive(Debug)]
pub enum Expression {
    Binary(ExpressionBinary),
    Parenthesized(ParenthesizedExpression),
    Call(CallExpression),
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    NumberLiteral(NumberLiteral),
    Empty,
    Assignment(AssignmentExpression),
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub struct Declaration {
    pub storage_class: StorageClass,
    pub dtype: DataType,
    pub declarator: Box<Declarator>,
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: ParenthesizedExpression,
    pub body: Box<Statement>,
    pub else_body: Option<Box<Statement>>,
}

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: ParenthesizedExpression,
    pub body: Box<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    ExpressionStatement(ExpressionStatement),
    ReturnStatement(ReturnStatement),
    Declaration(Declaration),
    CompoundStatement(CompoundStatement),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
}

#[derive(Debug)]
pub struct CompoundStatement {
    pub items: Vec<Statement>,
}

#[derive(Debug)]
pub enum TopLevelItem {
    FunctionDefinition(FunctionDefinition),
    Declaration(Declaration),
}

#[derive(Debug)]
pub struct TranslationUnit {
    pub items: Vec<TopLevelItem>,
}
