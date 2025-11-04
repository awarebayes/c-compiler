use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::common::Width;
use crate::ir::nodes::{self, Address, FunctionDef, Label, ToplevelItem};
use crate::semantic_analysis::{SymbolKind, SymbolType};
use crate::{parsing::ast, semantic_analysis::SymbolTableRef};

#[derive(Debug, Clone)]
struct State {
    var_count: Rc<RefCell<usize>>,
    label_count: Rc<RefCell<usize>>,
    return_width: Option<Width>,
    expression_width: Option<Width>,
    parent_phi_label: Label,
    source_counts: Rc<RefCell<HashMap<String, usize>>>
}

impl State {
    fn dummy(&self) -> Self {
        Self {
            parent_phi_label: self.parent_phi_label.clone(),
            return_width: self.return_width,
            var_count: Rc::new(RefCell::new(0)),
            label_count: Rc::new(RefCell::new(0)),
            source_counts: Rc::new(RefCell::new(HashMap::new())),
            expression_width: None,
        }
    }

    fn new(return_width: Width, parent_phi_label: Label) -> Self {
        Self {
            parent_phi_label,
            return_width: Some(return_width),
            var_count: Rc::new(RefCell::new(0)),
            label_count: Rc::new(RefCell::new(0)),
            source_counts: Rc::new(RefCell::new(HashMap::new())),
            expression_width: None,
        }
    }

    fn with_parent_phi_label(&self, label: Label) -> Self {
        let mut copy = self.clone();
        copy.parent_phi_label = label;
        copy
    }

    fn with_expr_width(&self, expression_width: Width) -> Self {
        let mut copy = self.clone();
        copy.expression_width = Some(expression_width);
        copy
    }

    fn last_var(&self) -> usize {
        *self.var_count.borrow() - 1
    }

    fn var_count(&self) -> usize {
        *self.var_count.borrow()
    }

    fn label_count(&self) -> usize {
        *self.label_count.borrow()
    }

    fn inc_var_cnt(&self) {
        *self.var_count.borrow_mut() += 1;
    }

    fn clone_counts(&self) -> HashMap<String, usize> {
        self.source_counts.borrow().clone()
    }

    fn inc_label_cnt(&self) {
        *self.label_count.borrow_mut() += 1;
    }

    fn inc_source_address_count(&self, source_address: &str) -> usize {
        let mut counts=  self.source_counts.borrow_mut();
        if !counts.contains_key(source_address) {
            counts.insert(source_address.to_owned(), 0);
            0
        } else {
            let count = counts.get_mut(source_address).unwrap();
            *count += 1;
            *count
        }
    }


    fn get_last_address_count(&self, address: &str) -> usize {
        let counts=  self.source_counts.borrow();
        counts.get(address).cloned().unwrap_or_default()
    }
}

trait SsaBuilder {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa>;
}

fn apply_assignment_to_exp(
    symbol_table: SymbolTableRef,
    state: &State,
    lvalue: &ast::Identifier,
    exp: &ast::Expression,
    assigment_type: &ast::AssignmentType,
) -> Vec<nodes::Ssa> {
    let mut new_ssas = exp.visit(symbol_table, state);

    let assignment_op = assigment_type.to_op().map(|op| nodes::Op::from_binop(&op));
    match assignment_op {
        Some(op) => {
            new_ssas.push(nodes::Ssa::Quadriplet(nodes::Quadriplet {
                dest: nodes::Address::compiler_temp(state.var_count()),
                op: op,
                left: nodes::Address::source_count(lvalue.0.clone(), state.get_last_address_count(&lvalue.0)),
                right: Some(nodes::Address::compiler_temp(state.last_var())),
                width: state.expression_width.unwrap(),
            }));
            state.inc_var_cnt();
        }
        None => (),
    }
    new_ssas
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

#[derive(Debug, Clone)]
struct ChangedPhiVar {
    source_var: Address,
    compiler_temp: Address,
    width: Width,
}

fn changed_phi_vars(ir: &[nodes::Ssa]) -> Vec<ChangedPhiVar> {
    ir.iter()
        .filter_map(|instruction| match instruction {
            nodes::Ssa::Assignment { dest, source, width } 
                if matches!(dest, nodes::Address::Source(_)) 
                => Some(ChangedPhiVar { source_var: dest.clone(), compiler_temp: source.clone(), width: *width }),
            _ => None,
        })
        .collect()
}


fn expression_vars(ir: &[nodes::Ssa]) -> Vec<ChangedPhiVar> {
    ir.iter()
        .filter_map(|instruction| match instruction {
            nodes::Ssa::Assignment { dest, source, width } 
                if matches!(source, nodes::Address::Source(_)) 
                => Some(ChangedPhiVar { source_var: source.clone(), compiler_temp: dest.clone(), width: *width }),
            _ => None,
        })
        .collect()
}

impl SsaBuilder for &ast::Expression {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa> {
        let mut nodes = vec![];
        match self {
            ast::Expression::Identifier(id) => {
                let dtype = symbol_table.borrow().query(&id.0);
                let width = Width::from_type(&dtype.unwrap().type_info);
                if let Some(w) = state.expression_width {
                    assert_eq!(w, width)
                }
                nodes.push(nodes::Ssa::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count()),
                    source: nodes::Address::source_count( id.0.clone(), state.get_last_address_count(&id.0) ),
                    width,
                });
                state.inc_var_cnt();
            }
            ast::Expression::Binary(bin) => {
                let mut new_state = state.clone();
                let estimated_width = expression_width(symbol_table.clone(), self);
                if let Some(w) = &state.expression_width {
                    if let ExpressionWidth::Some(est) = &estimated_width {
                        assert_eq!(w, est)
                    }
                } else {
                    if let ExpressionWidth::Some(est) = estimated_width {
                        new_state = state.with_expr_width(est)
                    } else {
                        panic!("Width is neither provided nor can be estimated");
                    }
                }

                let left_expression =
                    bin.left.as_ref().visit(symbol_table.clone(), &new_state);

                let left_temp_id = new_state.last_var();
                let right_expression =
                    bin.right.as_ref().visit(symbol_table.clone(), &new_state);

                let right_temp_id = new_state.last_var();
                nodes.extend(left_expression);
                nodes.extend(right_expression);

                nodes.push(nodes::Ssa::Quadriplet(nodes::Quadriplet {
                    dest: nodes::Address::CompilerTemp(new_state.var_count()),
                    op: nodes::Op::from_binop(&bin.op),
                    left: nodes::Address::CompilerTemp(left_temp_id),
                    right: Some(nodes::Address::CompilerTemp(right_temp_id)),
                    width: new_state.expression_width.unwrap(),
                }));
                new_state.inc_var_cnt();
            }
            ast::Expression::NumberLiteral(nl) => {
                nodes.push(nodes::Ssa::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count()),
                    source: nodes::Address::constant(nodes::AddressConstant::Numeric(
                        nl.0.parse().unwrap(),
                    )),
                    width: state.expression_width.unwrap(),
                });
                state.inc_var_cnt();
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
                    let arg_ssa = arg.visit(
                        symbol_table.clone(),
                        state,
                    );
                    let arg_temp = state.last_var();
                    nodes.extend(arg_ssa);
                    args_temps.push((arg_temp, width))
                }

                let function_adress = match ce.function.as_ref() {
                    ast::Expression::Identifier(id) => nodes::Address::source_count(id.0.clone(), 0),
                    _ => {
                        let function_ssa =
                            ce.function.as_ref().visit(symbol_table, state);
                        let function_temp = state.last_var();
                        nodes.extend(function_ssa);
                        nodes::Address::CompilerTemp(function_temp)
                    }
                };


                for (counter, &(index, width)) in args_temps.iter().enumerate() {
                    nodes.push(nodes::Ssa::Param {
                        value: nodes::Address::CompilerTemp(index),
                        width: width,
                        number: counter,

                    });
                }

                let return_width = Width::from_type(&symbol.type_info);

                nodes.push(nodes::Ssa::Call {
                    dest: Some((nodes::Address::CompilerTemp(state.var_count()), return_width)),
                    func: function_adress,
                    num_params: ce.arguments.len(),
                });
                state.inc_var_cnt();
            }
            ast::Expression::Empty => (),
            ast::Expression::StringLiteral(sl) => {
                nodes.push(nodes::Ssa::Assignment {
                    dest: nodes::Address::compiler_temp(state.var_count()),
                    source: nodes::Address::constant(nodes::AddressConstant::StringLiteral(
                        sl.0.clone(),
                    )),
                    width: Width::Long,
                });
                state.inc_var_cnt();
            }
            ast::Expression::Assignment(ast) => match &ast.lvalue {
                ast::LValue::Identifier(id) => {
                    let identifier_type = symbol_table.borrow().query(&id.0).unwrap().type_info;
                    let identifier_width = Width::from_type(&identifier_type);
                    let exp_ssas = apply_assignment_to_exp(
                        symbol_table,
                        &state.with_expr_width(identifier_width),
                        id,
                        ast.rvalue.as_ref(),
                        &ast.atype,
                    );
                    nodes.extend(exp_ssas);

                    let count = state.inc_source_address_count(&id.0);

                    nodes.push(nodes::Ssa::Assignment {
                        dest: nodes::Address::source_count(id.0.clone(), count),
                        source: nodes::Address::CompilerTemp(state.last_var()),
                        width: identifier_width,
                    });
                }
            },
            _ => todo!(),
        }
        nodes
    }
}

fn generate_phi_if_else(
    changed_if: &[ChangedPhiVar], 
    changed_else: &[ChangedPhiVar],
    counts_before: &HashMap<String, usize>,
    state: &State,
    true_label: Label,
    false_label: Label,
) -> Vec<nodes::Ssa> {
    let mut res_ssa = vec![];
    let mut groupby_source_var: HashMap<String, (Option<ChangedPhiVar>, Option<ChangedPhiVar>)> = HashMap::new();
    for i in changed_if.iter().chain(changed_else.iter()) {
        groupby_source_var.insert(i.source_var.get_source().to_owned(), (None, None));
    }

    for i in changed_if.iter() {
        groupby_source_var.get_mut(i.source_var.get_source()).unwrap().0 = Some(i.clone());
    }

    for i in changed_else.iter() {
        groupby_source_var.get_mut(i.source_var.get_source()).unwrap().1 = Some(i.clone());
    }

    for (var_if, var_else) in groupby_source_var.values() {
        match (var_if, var_else) {
            (Some(var_if), Some(var_else)) => {
                let var_name = var_if.source_var.get_source();
                let count = state.inc_source_address_count(var_name);
                res_ssa.push(nodes::Ssa::Phi { dest: Address::source_count(var_name.to_owned(), count) , width: var_if.width, merging: vec![
                    (var_if.source_var.clone(), true_label.clone()),
                    (var_else.source_var.clone(), false_label.clone()),
                ]});
            },
            (Some(var_if), None) => {
                let var_name = var_if.source_var.get_source();
                let count = state.inc_source_address_count(var_name);
                res_ssa.push(nodes::Ssa::Phi { dest: Address::source_count(var_name.to_owned(), count) , width: var_if.width, merging: vec![
                    (var_if.source_var.clone(), true_label.clone()),
                    (Address::source_count(var_name.to_owned(), counts_before.get(var_name).cloned().unwrap_or_default() ), state.parent_phi_label.clone())
                ]});
            },
            (None, Some(var_else)) => {
                let var_name = var_else.source_var.get_source();
                let count = state.inc_source_address_count(var_name);
                res_ssa.push(nodes::Ssa::Phi { dest: Address::source_count(var_name.to_owned(), count) , width: var_else.width, merging: vec![
                    (Address::source_count(var_name.to_owned(), counts_before.get(var_name).cloned().unwrap_or_default() ), state.parent_phi_label.clone()),
                    (var_else.source_var.clone(), false_label.clone()),
                ]});
            },
            (None, None) => {}
        }
    }

    res_ssa
}

impl SsaBuilder for &ast::IfStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa> {
        let mut out = vec![];
        match self.else_body.as_ref() {
            None => {
                let expr_ssas =
                    self.condition
                        .expression
                        .as_ref()
                        .visit(symbol_table.clone(), state);


                let true_label = nodes::Label::compiler_temp(state.label_count());
                let false_label = nodes::Label::compiler_temp(state.label_count() + 1);
                out.extend(expr_ssas);
                out.push(nodes::Ssa::Branch {
                    cond: nodes::Address::compiler_temp(state.last_var()),
                    true_target: true_label.clone(),
                    false_target: false_label.clone(),
                });

                state.inc_label_cnt();
                state.inc_label_cnt();

                let counts_before = state.clone_counts();

                out.push(nodes::Ssa::Label(true_label.clone()));

                let true_ssas =
                    self.body.as_ref().visit(symbol_table.clone(), state);

                let changed_phi_vars = changed_phi_vars(&true_ssas);
                out.extend(true_ssas);
                out.push(nodes::Ssa::Label(false_label));

                out.extend(changed_phi_vars.iter().map(|var| {
                    let count = state.inc_source_address_count(&var.source_var.get_source());
                    nodes::Ssa::Phi { dest: Address::source_count(var.source_var.get_source().to_owned(), count) , width: var.width, merging: vec![
                        (var.source_var.clone(), true_label.clone()),
                        (Address::source_count(var.source_var.get_source().to_owned(), counts_before.get(var.source_var.get_source()).cloned().unwrap_or_default() ), state.parent_phi_label.clone())
                    ]}
                }));
            }
            Some(body) => {
                let expr_ssas =
                    self.condition
                        .expression
                        .as_ref()
                        .visit(symbol_table.clone(), &state.clone());

                let counts_before = state.clone_counts();

                let true_label = nodes::Label::compiler_temp(state.label_count());
                let false_label = nodes::Label::compiler_temp(state.label_count() + 1);
                let end_label = nodes::Label::compiler_temp(state.label_count() + 2);
                state.inc_label_cnt();
                state.inc_label_cnt();
                state.inc_label_cnt();

                out.extend(expr_ssas);
                out.push(nodes::Ssa::Branch {
                    cond: nodes::Address::compiler_temp(state.last_var()),
                    true_target: true_label.clone(),
                    false_target: false_label.clone(),
                });
                out.push(nodes::Ssa::Label(true_label.clone()));

                let true_ssas =
                    self.body.as_ref().visit(symbol_table.clone(), state);

                let changed_true = changed_phi_vars(&true_ssas);

                out.extend(true_ssas);
                out.push(nodes::Ssa::Jump(end_label.clone()));
                out.push(nodes::Ssa::Label(false_label.clone()));

                let false_ssas =
                    body.as_ref().visit(symbol_table.clone(), state);

                let changed_false = changed_phi_vars(&false_ssas);

                out.extend(false_ssas);
                out.push(nodes::Ssa::Label(end_label.clone()));

                out.extend(generate_phi_if_else(&changed_true, &changed_false, &counts_before, state, true_label, false_label))

            }
        }

        out
    }
}

fn apply_changes_to_ssa(changed_vars: &[ChangedPhiVar], ir: &mut [nodes::Ssa], label: Label) {
    for n in ir {
        for cw in changed_vars {
            if let nodes::Ssa::Phi { dest, width: _, merging } = n {
                if cw.source_var.get_source() != dest.get_source() {
                    continue;
                }

                merging.push((
                    cw.source_var.clone(),
                    label.clone()
                ));
            }
        }
    }
}

impl SsaBuilder for &ast::WhileStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa> {
        let mut out = vec![];

        let before_cond_count = state.clone_counts();

        let expr_ssas_temp =
            self.condition
                .expression
                .as_ref()
                .visit(symbol_table.clone(), &state.dummy());

        let cond_label = nodes::Label::compiler_temp(state.label_count());
        let start_label = nodes::Label::compiler_temp(state.label_count() + 1);
        let end_label = nodes::Label::compiler_temp(state.label_count() + 2);

        state.inc_label_cnt();
        state.inc_label_cnt();
        state.inc_label_cnt();


        let loop_branch = nodes::Ssa::Branch {
            cond: nodes::Address::compiler_temp(state.last_var()),
            true_target: start_label.clone(),
            false_target: end_label.clone(),
        };


        let expr_vars = expression_vars(&expr_ssas_temp);

        out.push(nodes::Ssa::Label(cond_label.clone()));

        let phi_cond_start = out.len();

        out.extend(expr_vars.iter().map(|var| {
            let count = state.inc_source_address_count(&var.source_var.get_source());
            nodes::Ssa::Phi { dest: Address::source_count(var.source_var.get_source().to_owned(), count),
                width: var.width, merging: vec![
                (var.source_var.clone(), state.parent_phi_label.clone()),
            ] }
        }));

        let expr_ssas=
            self.condition
                .expression
                .as_ref()
                .visit(symbol_table.clone(), state);
        
        let phi_cond_end = out.len();

        out.extend(expr_ssas);
        out.push(loop_branch);
        out.push(nodes::Ssa::Label(start_label.clone()));


        let body_ssas = self.body.as_ref().visit(symbol_table.clone(), state);
        let changed_vars = changed_phi_vars(&body_ssas);

        out.extend(body_ssas);
        out.push(nodes::Ssa::Jump(cond_label.clone()));
        out.push(nodes::Ssa::Label(end_label.clone()));

        apply_changes_to_ssa(&changed_vars, &mut out[phi_cond_start..phi_cond_end], start_label.clone());
        
        out.extend(changed_vars.iter().map(|var| {
            let count = state.inc_source_address_count(&var.source_var.get_source());
            nodes::Ssa::Phi { dest: Address::source_count(var.source_var.get_source().to_owned(), count),
                width: var.width, merging: vec![
                (Address::source_count(var.source_var.get_source().to_owned(), before_cond_count.get(var.source_var.get_source()).cloned().unwrap_or_default() ), state.parent_phi_label.clone()),
                (var.source_var.clone(), start_label.clone()),
            ] }
        }));

        out
    }
}

impl SsaBuilder for &ast::Statement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa> {
        match self {
            ast::Statement::Declaration(decl) => match decl.declarator.as_ref() {
                ast::Declarator::FunctionDeclarator(_)
                | ast::Declarator::Identifier(_)
                | ast::Declarator::PointerDeclarator(_) => {
                    return vec![];
                }
                ast::Declarator::InitDeclarator(id) => {
                    let var_name = &decl.declarator.get_identifier().0;
                    let expr = &id.value;

                    let width = Width::from_type(
                        &symbol_table.borrow().query(&var_name).unwrap().type_info,
                    );

                    let mut expr_ssas = expr.visit(
                        symbol_table,
                        &state.with_expr_width(width),
                    );
                    let last_id = state.last_var();

                    let count = state.inc_source_address_count(&var_name);

                    expr_ssas.push(nodes::Ssa::Assignment {
                        dest: nodes::Address::source_count(var_name.clone(), count) ,
                        source: nodes::Address::compiler_temp(last_id),
                        width: width,
                    });

                    expr_ssas
                }
            },
            ast::Statement::ReturnStatement(rs) => {
                if matches!(rs.expression, ast::Expression::Empty) {
                    return vec![nodes::Ssa::Return { value: None }];
                } else {
                    let expr_width = expression_width(symbol_table.clone(), &rs.expression);
                    if let ExpressionWidth::Some(est) = expr_width {
                        assert_eq!(est, state.return_width.unwrap());
                    }
                    let mut expr_ssas =
                        (&rs.expression).visit(symbol_table, state);
                    let expression_res_var = state.last_var();
                    expr_ssas.push(nodes::Ssa::Return {
                        value: Some((
                            nodes::Address::compiler_temp(expression_res_var),
                            state.return_width.unwrap(),
                        )),
                    });
                    state.inc_var_cnt();
                    return expr_ssas;
                }
            }
            ast::Statement::ExpressionStatement(es) => {
                let expr_ssas = (&es.expression).visit(symbol_table, state);
                expr_ssas
            }
            ast::Statement::IfStatement(ifs) => ifs.visit(symbol_table, state),
            ast::Statement::WhileStatement(cs) => cs.visit(symbol_table, state),
            ast::Statement::CompoundStatement(cs) => cs.visit(symbol_table, state),
        }
    }
}

impl SsaBuilder for &ast::CompoundStatement {
    fn visit(
        &self,
        symbol_table: SymbolTableRef,
        state: &State,
    ) -> Vec<nodes::Ssa> {
        let mut ssas = vec![];

        // TODO: ADJUST SYMBOL TABLE HERE!!!!!!!!!!!!!

        for statement in &self.items {
            let new_ssas = statement.visit(symbol_table.clone(), &state);
            ssas.extend(new_ssas);
        }

        ssas
    }
}

fn function_ssa(fd: &ast::FunctionDefinition, symbol_table: SymbolTableRef) -> ToplevelItem {
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

    let begin_label = Label::source(format!("start_function_{}", function_name));

    ToplevelItem::Function(FunctionDef {
        name: function_name,
        parameters,
        body: ([
            vec![nodes::Ssa::Label(begin_label.clone())],
            (&fd.body)
            .visit(
                symbol_table,
                &State::new(return_width, begin_label)
            ),
            ]).concat(),
        return_width: return_width,
    })
}

fn declaration_ssa(dec: &ast::Declaration) -> ToplevelItem {
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

pub fn build_ssa(
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
                toplevels.push(function_ssa(
                    &fd,
                    symbol_table.borrow().new_with_scope(context),
                ));
                function_decl_count += 1;
            }
            ast::TopLevelItem::Declaration(dec) => {
                toplevels.push(declaration_ssa(&dec));
            }
        }
    }

    toplevels
}
