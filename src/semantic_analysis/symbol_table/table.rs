use crate::{
    common::StorageClass, parsing::ast, semantic_analysis::symbol_table::ast_visitor::Visitable,
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum SymbolType {
    Int,
    Char,
    Pointer(Box<SymbolType>),
}

impl SymbolType {
    pub fn make_ptr(base_type: SymbolType, nest: usize) -> SymbolType {
        if nest == 0 {
            return base_type;
        }
        return SymbolType::Pointer(Box::new(Self::make_ptr(base_type, nest - 1)));
    }
}

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("InvalidPointerDeclarator")]
    InvalidPointerDeclarator,

    #[error("InvalidParameterDeclarator")]
    InvalidParameterDeclarator,
}

impl TryFrom<&ast::DataType> for SymbolType {
    type Error = SemanticError;
    fn try_from(value: &ast::DataType) -> Result<Self, Self::Error> {
        Ok(match value {
            ast::DataType::Char => Self::Char,
            ast::DataType::Int => Self::Int,
        })
    }
}

impl TryFrom<(&ast::Declarator, &SymbolType)> for SymbolType {
    type Error = SemanticError;
    fn try_from((value, symb): (&ast::Declarator, &SymbolType)) -> Result<Self, Self::Error> {
        match &value {
            &ast::Declarator::PointerDeclarator(pr) => {
                let pointee = Box::new(TryFrom::<(&ast::Declarator, &SymbolType)>::try_from((
                    &pr.declarator,
                    symb,
                ))?);
                Ok(SymbolType::Pointer(pointee))
            }
            &ast::Declarator::Identifier(_) => Ok(symb.clone()),
            &ast::Declarator::FunctionDeclarator(fd) => {
                let pointee =
                    TryFrom::<(&ast::Declarator, &SymbolType)>::try_from((&fd.declarator, symb))?;
                Ok(pointee)
            }
            _ => Err(SemanticError::InvalidPointerDeclarator),
        }
    }
}

impl TryFrom<&ast::ParameterDeclaration> for SymbolType {
    type Error = SemanticError;
    fn try_from(value: &ast::ParameterDeclaration) -> Result<Self, Self::Error> {
        let dtype = TryFrom::try_from(&value.dtype)?;
        match value.declarator.as_ref() {
            ast::Declarator::Identifier(_) => Ok(dtype),
            ast::Declarator::PointerDeclarator(pd) => {
                let pointee = Box::new(TryFrom::<(&ast::Declarator, &SymbolType)>::try_from((
                    &pd.declarator,
                    &dtype,
                ))?);
                Ok(SymbolType::Pointer(pointee))
            }
            _ => Err(SemanticError::InvalidPointerDeclarator),
        }
    }
}

impl TryFrom<&ast::FunctionParameter> for SymbolType {
    type Error = SemanticError;
    fn try_from(value: &ast::FunctionParameter) -> Result<Self, Self::Error> {
        match value {
            ast::FunctionParameter::VariadicParameter => panic!("Unsupported"),
            ast::FunctionParameter::ParameterDeclaration(pd) => TryFrom::try_from(pd),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable { is_mutable: bool },
    Function { parameters: Vec<SymbolType> },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub type_info: SymbolType,
    pub storage_class: StorageClass,
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("symbols", &self.symbols)
            .field("children", &self.children) // Show just count
            .field("has_parent", &self.parent.is_some())
            .finish()
    }
}

#[derive(Default)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub children: Vec<Rc<RefCell<Scope>>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Rc<RefCell<Scope>>,
    pub global_scope: Rc<RefCell<Scope>>,
}

impl SymbolTable {
    pub fn enter_scope_mut(&mut self) {
        let new_scope = Rc::new(RefCell::new(Scope {
            symbols: HashMap::new(),
            parent: Some(self.current_scope.clone()),
            children: vec![],
        }));

        self.current_scope
            .borrow_mut()
            .children
            .push(new_scope.clone());
        self.current_scope = new_scope;
    }

    pub fn new_with_scope(&self, scope: Rc<RefCell<Scope>>) -> SymbolTableRef {
        Rc::new(RefCell::new(SymbolTable {
            current_scope: scope,
            global_scope: self.global_scope.clone(),
        }))
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.current_scope
            .borrow_mut()
            .symbols
            .insert(symbol.name.clone(), symbol);
    }

    pub fn exit_scope_mut(&mut self) {
        let maybe_parent = self
            .current_scope
            .borrow()
            .parent
            .as_ref()
            .map(|p| p.clone());

        if let Some(parent) = maybe_parent {
            self.current_scope = parent.clone()
        }
    }

    pub fn from_translation_unit(unit: &ast::TranslationUnit) -> Rc<RefCell<Self>> {
        let global_scope = Rc::new(RefCell::new(Scope::default()));
        let table = Rc::new(RefCell::new(SymbolTable {
            current_scope: global_scope.clone(),
            global_scope: global_scope.clone(),
        }));
        unit.visit(table.clone(), None);

        table
    }

    pub fn query(&self, name: &str) -> Option<Symbol> {
        let current = &self.current_scope.borrow().symbols;
        match current.get(name) {
            Some(symb) => Some(symb.clone()),
            None => {
                let parent = &self.current_scope.borrow().parent;
                match parent {
                    Some(par) => {
                        let parent_table = self.new_with_scope(par.clone());
                        parent_table.borrow().query(name)
                    }
                    None => None,
                }
            }
        }
    }
}

pub type SymbolTableRef = Rc<RefCell<SymbolTable>>;
