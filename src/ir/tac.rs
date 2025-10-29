use crate::common::Width;
use crate::ir::nodes::{self, FunctionDef, ToplevelItem};
use crate::semantic_analysis::{SymbolKind, SymbolType};
use crate::{parsing::ast, semantic_analysis::SymbolTableRef};

#[derive(Debug, Default, Clone, Copy)]
struct State {
    var_count: usize,
    label_count: usize,
    return_width: Option<Width>,
}

#[derive(Debug, Clone, Copy)]
struct VisitExtra {
    expression_width: Width,
}

impl State {
    pub fn last_var(&self) -> usize {
        self.var_count - 1
    }
}

trait TacBuilder {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State);
}

fn apply_assignment_to_exp(
    symbol_table: SymbolTableRef,
    mut state: State,
    lvalue: &ast::Identifier,
    exp: &ast::Expression,
    assigment_type: &ast::AssignmentType,
    extra: Option<VisitExtra>,
) -> (Vec<nodes::Tac>, State) {
    let (mut new_tacs, new_state) = exp.visit(symbol_table, state, extra);
    state = new_state;

    let assignment_op = assigment_type.to_op().map(|op| nodes::Op::from_binop(&op));
    match assignment_op {
        Some(op) => {
            new_tacs.push(nodes::Tac::Quadriplet(nodes::Quadriplet {
                dest: nodes::Address::compiler_temp(state.var_count),
                op: op,
                left: nodes::Address::source(lvalue.0.clone()),
                right: Some(nodes::Address::compiler_temp(state.last_var())),
                width: extra.unwrap().expression_width,
            }));
            state.var_count += 1;
        }
        None => (),
    }
    (new_tacs, state)
}

enum ExpressionWidth {
    Some(Width),
    CastableWidth,
}

fn expression_width(symbol_table: SymbolTableRef, expression: &ast::Expression) -> ExpressionWidth {
    match expression {
        ast::Expression::Identifier(id) => {
            let symbol = symbol_table.borrow().query(&id.0);
            ExpressionWidth::Some(Width::from_type(&symbol.unwrap().type_info))
        }
        ast::Expression::Binary(bin) => {
            let left_width = expression_width(symbol_table.clone(), &bin.left);
            let right_width = expression_width(symbol_table.clone(), &bin.left);
            match (&left_width, &right_width) {
                (ExpressionWidth::Some(lw), ExpressionWidth::Some(rw)) => {
                    assert_eq!(lw, rw);
                    left_width
                }
                (ExpressionWidth::Some(w), ExpressionWidth::CastableWidth) => {
                    ExpressionWidth::Some(*w)
                }
                (ExpressionWidth::CastableWidth, ExpressionWidth::Some(w)) => {
                    ExpressionWidth::Some(*w)
                }
                (ExpressionWidth::CastableWidth, ExpressionWidth::CastableWidth) => {
                    ExpressionWidth::CastableWidth
                }
            }
        }
        ast::Expression::Assignment(asn) => match &asn.lvalue {
            ast::LValue::Identifier(id) => {
                let symbol = symbol_table.borrow().query(&id.0);
                ExpressionWidth::Some(Width::from_type(&symbol.unwrap().type_info))
            }
        },
        ast::Expression::Call(cl) => {
            let function_name = match cl.function.as_ref() {
                ast::Expression::Identifier(f) => &f.0,
                _ => todo!(),
            };
            let symbol = symbol_table.borrow().query(&function_name);
            ExpressionWidth::Some(Width::from_type(&symbol.unwrap().type_info))
        }
        ast::Expression::Parenthesized(pe) => expression_width(symbol_table, &pe.expression),
        ast::Expression::NumberLiteral(_) => ExpressionWidth::CastableWidth,
        ast::Expression::StringLiteral(_) => ExpressionWidth::Some(Width::Long),
        ast::Expression::Empty => {
            panic!()
        }
    }
}

impl TacBuilder for &ast::Expression {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        mut state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State) {
        let mut nodes = vec![];
        match self {
            ast::Expression::Identifier(id) => {
                let dtype = symbol_table.borrow().query(&id.0);
                let width = Width::from_type(&dtype.unwrap().type_info);
                if let Some(ex) = extra {
                    assert_eq!(ex.expression_width, width)
                }
                nodes.push(nodes::Tac::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count),
                    source: nodes::Address::source(id.0.clone()),
                    width,
                });
                state.var_count += 1;
            }
            ast::Expression::Binary(bin) => {
                let mut width = extra;
                let estimated_width = expression_width(symbol_table.clone(), self);
                if let Some(extra) = extra {
                    if let ExpressionWidth::Some(est) = estimated_width {
                        assert_eq!(extra.expression_width, est)
                    }
                } else {
                    if let ExpressionWidth::Some(est) = estimated_width {
                        width = Some(VisitExtra {
                            expression_width: est,
                        })
                    } else {
                        panic!("Width is neither provided nor can be estimated");
                    }
                }

                let (left_expression, new_state) =
                    bin.left.as_ref().visit(symbol_table.clone(), state, width);
                state = new_state;
                let left_temp_id = state.last_var();
                let (right_expression, new_state) =
                    bin.right.as_ref().visit(symbol_table.clone(), state, width);
                state = new_state;
                let right_temp_id = state.last_var();
                nodes.extend(left_expression);
                nodes.extend(right_expression);

                nodes.push(nodes::Tac::Quadriplet(nodes::Quadriplet {
                    dest: nodes::Address::CompilerTemp(state.var_count),
                    op: nodes::Op::from_binop(&bin.op),
                    left: nodes::Address::CompilerTemp(left_temp_id),
                    right: Some(nodes::Address::CompilerTemp(right_temp_id)),
                    width: width.unwrap().expression_width,
                }));
                state.var_count += 1;
            }
            ast::Expression::NumberLiteral(nl) => {
                nodes.push(nodes::Tac::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count),
                    source: nodes::Address::constant(nodes::AddressConstant::Numeric(
                        nl.0.parse().unwrap(),
                    )),
                    width: extra.unwrap().expression_width,
                });
                state.var_count += 1;
            }
            ast::Expression::Call(ce) => {
                let mut args_temps = vec![];
                let symbol = symbol_table
                    .borrow()
                    .query(&ce.get_identifier().as_ref().unwrap().0)
                    .unwrap();

                let parameters = match &symbol.kind {
                    SymbolKind::Variable { is_mutable: _ } => panic!("Cannot call a variable"),
                    SymbolKind::Function { parameters } => parameters,
                };

                if ce.arguments.len() != parameters.len() {
                    panic!(
                        "Wrong number of arguments, expected {} got {}",
                        parameters.len(),
                        ce.arguments.len()
                    )
                }

                for (arg, param) in ce.arguments.iter().zip(parameters.iter()) {
                    let estimated_width = expression_width(symbol_table.clone(), arg);
                    let width = Width::from_type(param);
                    if let ExpressionWidth::Some(est) = estimated_width {
                        assert_eq!(width, est);
                    }
                    let (arg_tac, new_state) = arg.visit(
                        symbol_table.clone(),
                        state,
                        Some(VisitExtra {
                            expression_width: width,
                        }),
                    );
                    state = new_state;
                    let arg_temp = state.last_var();
                    nodes.extend(arg_tac);
                    args_temps.push((arg_temp, width))
                }

                let function_adress = match ce.function.as_ref() {
                    ast::Expression::Identifier(id) => nodes::Address::Source(id.0.clone()),
                    _ => {
                        let (function_tac, new_state) =
                            ce.function.as_ref().visit(symbol_table, state, None);
                        state = new_state;
                        let function_temp = state.last_var();
                        nodes.extend(function_tac);
                        nodes::Address::CompilerTemp(function_temp)
                    }
                };


                for (counter, &(index, width)) in args_temps.iter().enumerate() {
                    nodes.push(nodes::Tac::Param {
                        value: nodes::Address::CompilerTemp(index),
                        width: width,
                        number: counter,

                    });
                }

                let return_width = Width::from_type(&symbol.type_info);

                nodes.push(nodes::Tac::Call {
                    dest: Some((nodes::Address::CompilerTemp(state.var_count), return_width)),
                    func: function_adress,
                    num_params: ce.arguments.len(),
                });
                state.var_count += 1;
            }
            ast::Expression::Empty => (),
            ast::Expression::StringLiteral(sl) => {
                nodes.push(nodes::Tac::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count),
                    source: nodes::Address::constant(nodes::AddressConstant::StringLiteral(
                        sl.0.clone(),
                    )),
                    width: Width::Long,
                });
                state.var_count += 1;
            }
            ast::Expression::Assignment(ast) => match &ast.lvalue {
                ast::LValue::Identifier(id) => {
                    let identifier_type = symbol_table.borrow().query(&id.0).unwrap().type_info;
                    let identifier_width = Width::from_type(&identifier_type);
                    let (exp_tacs, new_state) = apply_assignment_to_exp(
                        symbol_table,
                        state,
                        id,
                        ast.rvalue.as_ref(),
                        &ast.atype,
                        Some(VisitExtra {
                            expression_width: identifier_width,
                        }),
                    );
                    state = new_state;
                    nodes.extend(exp_tacs);
                    nodes.push(nodes::Tac::Assignment {
                        dest: nodes::Address::Source(id.0.clone()),
                        source: nodes::Address::CompilerTemp(state.last_var()),
                        width: identifier_width,
                    });
                }
            },
            _ => todo!(),
        }
        (nodes, state)
    }
}

impl TacBuilder for &ast::IfStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        mut state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State) {
        let mut out = vec![];
        match self.else_body.as_ref() {
            None => {
                let (expr_tacs, new_state) =
                    self.condition
                        .expression
                        .as_ref()
                        .visit(symbol_table.clone(), state, None);

                state = new_state;

                let true_label = nodes::Label::compiler_temp(state.label_count);
                let false_label = nodes::Label::compiler_temp(state.label_count + 1);
                out.extend(expr_tacs);
                out.push(nodes::Tac::Branch {
                    cond: nodes::Address::compiler_temp(state.last_var()),
                    true_target: true_label.clone(),
                    false_target: false_label.clone(),
                });
                state.label_count += 2;

                out.push(nodes::Tac::Label(true_label));

                let (true_tacs, new_state) =
                    self.body.as_ref().visit(symbol_table.clone(), state, None);

                state = new_state;
                out.extend(true_tacs);
                out.push(nodes::Tac::Label(false_label));
            }
            Some(body) => {
                let (expr_tacs, new_state) =
                    self.condition
                        .expression
                        .as_ref()
                        .visit(symbol_table.clone(), state, None);

                state = new_state;
                let true_label = nodes::Label::compiler_temp(state.label_count);
                let false_label = nodes::Label::compiler_temp(state.label_count + 1);
                let end_label = nodes::Label::compiler_temp(state.label_count + 2);
                state.label_count += 3;

                out.extend(expr_tacs);
                out.push(nodes::Tac::Branch {
                    cond: nodes::Address::compiler_temp(state.last_var()),
                    true_target: true_label.clone(),
                    false_target: false_label.clone(),
                });
                out.push(nodes::Tac::Label(true_label));

                let (true_tacs, new_state) =
                    self.body.as_ref().visit(symbol_table.clone(), state, None);

                state = new_state;
                out.extend(true_tacs);
                out.push(nodes::Tac::Jump(end_label.clone()));
                out.push(nodes::Tac::Label(false_label));

                let (false_tacs, new_state) =
                    body.as_ref().visit(symbol_table.clone(), state, None);

                state = new_state;
                out.extend(false_tacs);
                out.push(nodes::Tac::Label(end_label));
            }
        }

        (out, state)
    }
}

impl TacBuilder for &ast::WhileStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        mut state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State) {
        let mut out = vec![];

        let (expr_tacs, new_state) =
            self.condition
                .expression
                .as_ref()
                .visit(symbol_table.clone(), state, None);

        state = new_state;

        let cond_label = nodes::Label::compiler_temp(state.label_count);
        let start_label = nodes::Label::compiler_temp(state.label_count + 1);
        let end_label = nodes::Label::compiler_temp(state.label_count + 2);
        state.label_count += 3;

        out.push(nodes::Tac::Label(cond_label.clone()));
        out.extend(expr_tacs);
        out.push(nodes::Tac::Branch {
            cond: nodes::Address::compiler_temp(state.last_var()),
            true_target: start_label.clone(),
            false_target: end_label.clone(),
        });
        out.push(nodes::Tac::Label(start_label));

        let (body_tacs, new_state) = self.body.as_ref().visit(symbol_table.clone(), state, None);

        state = new_state;
        out.extend(body_tacs);
        out.push(nodes::Tac::Jump(cond_label));
        out.push(nodes::Tac::Label(end_label));

        (out, state)
    }
}

impl TacBuilder for &ast::Statement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        mut state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State) {
        match self {
            ast::Statement::Declaration(decl) => match decl.declarator.as_ref() {
                ast::Declarator::FunctionDeclarator(_)
                | ast::Declarator::Identifier(_)
                | ast::Declarator::PointerDeclarator(_) => {
                    return (vec![], state);
                }
                ast::Declarator::InitDeclarator(id) => {
                    let var_name = &decl.declarator.get_identifier().0;
                    let expr = &id.value;

                    let width = Width::from_type(
                        &symbol_table.borrow().query(&var_name).unwrap().type_info,
                    );

                    let (mut expr_tacs, new_state) = expr.visit(
                        symbol_table,
                        state,
                        Some(VisitExtra {
                            expression_width: width,
                        }),
                    );
                    state = new_state;
                    let last_id = state.last_var();

                    expr_tacs.push(nodes::Tac::Assignment {
                        dest: nodes::Address::source(var_name.clone()),
                        source: nodes::Address::compiler_temp(last_id),
                        width: width,
                    });

                    (expr_tacs, state)
                }
            },
            ast::Statement::ReturnStatement(rs) => {
                if matches!(rs.expression, ast::Expression::Empty) {
                    return (vec![nodes::Tac::Return { value: None }], state);
                } else {
                    let expr_width = expression_width(symbol_table.clone(), &rs.expression);
                    if let ExpressionWidth::Some(est) = expr_width {
                        assert_eq!(est, state.return_width.unwrap());
                    }
                    let (mut expr_tacs, new_state) =
                        (&rs.expression).visit(symbol_table, state, None);
                    state = new_state;
                    let expression_res_var = state.last_var();
                    expr_tacs.push(nodes::Tac::Return {
                        value: Some((
                            nodes::Address::compiler_temp(expression_res_var),
                            state.return_width.unwrap(),
                        )),
                    });
                    state.var_count += 1;
                    return (expr_tacs, state);
                }
            }
            ast::Statement::ExpressionStatement(es) => {
                let (expr_tacs, new_count) = (&es.expression).visit(symbol_table, state, None);
                (expr_tacs, new_count)
            }
            ast::Statement::IfStatement(ifs) => ifs.visit(symbol_table, state, None),
            ast::Statement::WhileStatement(cs) => cs.visit(symbol_table, state, None),
            ast::Statement::CompoundStatement(cs) => cs.visit(symbol_table, state, None),
        }
    }
}

impl TacBuilder for &ast::CompoundStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        mut state: State,
        extra: Option<VisitExtra>,
    ) -> (Vec<nodes::Tac>, State) {
        let mut tacs = vec![];

        // TODO: ADJUST SYMBOL TABLE HERE!!!!!!!!!!!!!

        for statement in &self.items {
            let (new_tacs, new_var_count) = statement.visit(symbol_table.clone(), state, None);
            state = new_var_count;
            tacs.extend(new_tacs);
        }

        (tacs, state)
    }
}

fn function_tac(fd: &ast::FunctionDefinition, symbol_table: SymbolTableRef) -> ToplevelItem {
    let current_context = symbol_table.borrow().current_scope.clone();
    let global_context = symbol_table.borrow().global_scope.clone();
    let symbols = &current_context.borrow().symbols;

    let parameter_names: Vec<String> = fd
        .declarator
        .parameters()
        .into_iter()
        .filter_map(|param| match &param {
            &ast::FunctionParameter::ParameterDeclaration(pd) => {
                Some(pd.declarator.get_identifier().0)
            }
            &ast::FunctionParameter::VariadicParameter => None,
        })
        .collect();

    let parameters: Vec<_> = parameter_names
        .iter()
        .map(|name| {
            let symbol = symbols.get(name).unwrap();
            let dtype = &symbol.type_info;
            let width = Width::from_type(dtype);
            (symbol.name.clone(), width)
        })
        .collect();

    let function_name = fd.declarator.get_identifier().0;
    let function_symbol_type = &global_context.borrow().symbols[&function_name].type_info;
    let return_width = Width::from_type(function_symbol_type);

    ToplevelItem::Function(FunctionDef {
        name: function_name,
        parameters,
        body: (&fd.body)
            .visit(
                symbol_table,
                State {
                    return_width: Some(return_width),
                    ..Default::default()
                },
                None,
            )
            .0,
        return_width: return_width,
    })
}

fn declaration_tac(dec: &ast::Declaration) -> ToplevelItem {
    match dec.declarator.as_ref() {
        ast::Declarator::FunctionDeclarator(fd) => {
            let function_name = fd.declarator.get_identifier().0;
            let symbol_type = SymbolType::try_from(&dec.dtype).unwrap();
            let decl_type = SymbolType::try_from((dec.declarator.as_ref(), &symbol_type)).unwrap();
            let return_width = Width::from_type(&decl_type);
            let storage_class = dec.storage_class;

            let parameter_widths = fd
                .parameters
                .iter()
                .filter_map(|fp| match fp {
                    ast::FunctionParameter::ParameterDeclaration(pd) => {
                        let symbol_type = SymbolType::try_from(&pd.dtype).unwrap();
                        let decl_type =
                            SymbolType::try_from((pd.declarator.as_ref(), &symbol_type)).unwrap();
                        let return_width = Width::from_type(&decl_type);
                        Some(return_width)
                    }
                    ast::FunctionParameter::VariadicParameter => None,
                })
                .collect();

            ToplevelItem::Declaration(nodes::ToplevelDeclaration::Function {
                storage_class,
                name: function_name,
                return_width: return_width,
                parameters: parameter_widths,
            })
        }
        _ => todo!(),
    }
}

pub fn build_tac(
    unit: &ast::TranslationUnit,
    symbol_table: SymbolTableRef,
) -> Vec<crate::ir::nodes::ToplevelItem> {
    let mut toplevels = vec![];
    let mut function_decl_count = 0;
    for i in unit.items.iter() {
        match i {
            ast::TopLevelItem::FunctionDefinition(fd) => {
                let context = symbol_table.borrow().global_scope.borrow().children
                    [function_decl_count]
                    .clone();
                toplevels.push(function_tac(
                    &fd,
                    symbol_table.borrow().new_with_scope(context),
                ));
                function_decl_count += 1;
            }
            ast::TopLevelItem::Declaration(dec) => {
                toplevels.push(declaration_tac(&dec));
            }
        }
    }

    toplevels
}
