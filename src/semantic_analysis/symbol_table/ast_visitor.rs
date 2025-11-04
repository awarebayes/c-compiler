use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::common::StorageClass;
use crate::parsing::ast;
use crate::semantic_analysis::symbol_table::table::{self, Symbol, SymbolTable as SymbolTableRaw};

type SymbolTable = Rc<RefCell<SymbolTableRaw>>;

pub trait Visitable {
    fn visit(&self, state: SymbolTable, injection: Option<HashMap<String, Symbol>>);
}

impl Visitable for &ast::Statement {
    fn visit(&self, table: SymbolTable, _injection: Option<HashMap<String, Symbol>>) {
        match &self {
            ast::Statement::CompoundStatement(cs) => cs.visit(table.clone(), None),
            ast::Statement::ExpressionStatement(_) => (),
            ast::Statement::ReturnStatement(_) => (),
            ast::Statement::IfStatement(ifs) => ifs.body.as_ref().visit(table.clone(), None),
            ast::Statement::WhileStatement(ws) => ws.body.as_ref().visit(table.clone(), None),
            ast::Statement::Declaration(d) => d.visit(table.clone(), None),
        };
    }
}

impl Visitable for &ast::Declaration {
    fn visit(&self, table: SymbolTable, _injection: Option<HashMap<String, Symbol>>) {
        let dtype = &self.dtype;
        let symbol_type = table::SymbolType::try_from(dtype).unwrap();
        let identifier = self.declarator.get_identifier();
        let storage_class = self.storage_class;

        match self.declarator.as_ref() {
            ast::Declarator::InitDeclarator(init_dec) => {
                let symbol = Symbol {
                    name: identifier.0.clone(),
                    kind: table::SymbolKind::Variable { is_mutable: true },
                    type_info: table::SymbolType::try_from((
                        init_dec.declarator.as_ref(),
                        &symbol_type,
                    ))
                    .unwrap(),
                    storage_class,
                };
                table.borrow_mut().add_symbol(symbol);
            }
            ast::Declarator::PointerDeclarator(pointer_dec) => {
                let pointee =
                    table::SymbolType::try_from((pointer_dec.declarator.as_ref(), &symbol_type))
                        .unwrap();
                let symbol = Symbol {
                    name: identifier.0.clone(),
                    kind: table::SymbolKind::Variable { is_mutable: true },
                    type_info: table::SymbolType::Pointer(Box::new(pointee)),
                    storage_class: storage_class,
                };
                table.borrow_mut().add_symbol(symbol);
            }
            ast::Declarator::FunctionDeclarator(func_dec) => {
                let parameter_types: Vec<table::SymbolType> = func_dec
                    .parameters
                    .iter()
                    .filter_map(|fp| match fp {
                        ast::FunctionParameter::ParameterDeclaration(_) => {
                            Some(table::SymbolType::try_from(fp).unwrap())
                        }
                        ast::FunctionParameter::VariadicParameter => None,
                    })
                    .collect();

                let symbol = Symbol {
                    name: identifier.0.clone(),
                    kind: table::SymbolKind::Function {
                        parameters: parameter_types,
                        is_variadic: func_dec.is_variadic,
                    },
                    type_info: table::SymbolType::try_from((
                        func_dec.declarator.as_ref(),
                        &symbol_type,
                    ))
                    .unwrap(),
                    storage_class: storage_class,
                };

                table.borrow_mut().add_symbol(symbol);
            }

            _ => panic!(),
        }
    }
}

impl Visitable for &ast::CompoundStatement {
    fn visit(&self, table: SymbolTable, injection: Option<HashMap<String, Symbol>>) {
        table.borrow_mut().enter_scope_mut();

        if let Some(inj) = injection {
            for symb in inj.into_values() {
                table.borrow_mut().add_symbol(symb);
            }
        }

        for item in self.items.iter() {
            (&item).visit(table.clone(), None);
        }

        table.borrow_mut().exit_scope_mut();
    }
}

impl Visitable for &ast::FunctionDefinition {
    fn visit(&self, table: SymbolTable, _injection: Option<HashMap<String, Symbol>>) {
        let func_name = self.declarator.get_identifier();

        let return_type =
            if let ast::FunctionDeclaratorField::PointerDeclarator(pd) = &self.declarator {
                let nest = pd.get_nest_level();
                let base_type = table::SymbolType::try_from(&self.return_type).unwrap();
                table::SymbolType::make_ptr(base_type, nest)
            } else {
                table::SymbolType::try_from(&self.return_type).unwrap()
            };

        let parameter_names: Vec<String> = self
            .declarator
            .parameters()
            .into_iter()
            .filter_map(|x| match &x {
                &ast::FunctionParameter::ParameterDeclaration(param) => {
                    Some(param.declarator.get_identifier().0)
                }
                &ast::FunctionParameter::VariadicParameter => None,
            })
            .collect();

        let parameter_symbols: Vec<table::SymbolType> = self
            .declarator
            .parameters()
            .into_iter()
            .filter_map(|fp| match fp {
                ast::FunctionParameter::ParameterDeclaration(_) => {
                    Some(table::SymbolType::try_from(fp).unwrap())
                }
                ast::FunctionParameter::VariadicParameter => None,
            })
            .collect();

        let is_variadic = match &self.declarator {
            ast::FunctionDeclaratorField::FunctionDeclarator(fd) => fd.is_variadic,
            _ => panic!(),
        };

        table.borrow_mut().add_symbol(Symbol {
            name: func_name.0,
            kind: table::SymbolKind::Function {
                parameters: parameter_symbols.clone(),
                is_variadic: is_variadic,
            },
            type_info: return_type,
            storage_class: StorageClass::Auto,
        });

        let injected_parameters = parameter_names
            .iter()
            .zip(parameter_symbols.iter())
            .map(|(name, dtype)| {
                (
                    name.clone(),
                    Symbol {
                        name: name.clone(),
                        kind: table::SymbolKind::Variable { is_mutable: true },
                        type_info: dtype.clone(),
                        storage_class: StorageClass::Auto,
                    },
                )
            })
            .collect();

        (&self.body).visit(table.clone(), Some(injected_parameters));
    }
}

impl Visitable for &ast::TopLevelItem {
    fn visit(&self, table: SymbolTable, _injection: Option<HashMap<String, Symbol>>) {
        match self {
            ast::TopLevelItem::FunctionDefinition(fd) => fd.visit(table, None),
            ast::TopLevelItem::Declaration(dec) => dec.visit(table, None),
        }
    }
}

impl Visitable for &ast::TranslationUnit {
    fn visit(&self, table: SymbolTable, _injection: Option<HashMap<String, Symbol>>) {
        self.items
            .iter()
            .for_each(|item| item.visit(table.clone(), None))
    }
}
