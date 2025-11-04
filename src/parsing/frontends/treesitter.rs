use crate::common::StorageClass;
use crate::parsing::ast::{
    self, CallExpression, CompoundStatement, Expression, ExpressionStatement, Identifier,
    ParenthesizedExpression, PointerDeclarator, ReturnStatement, TopLevelItem,
};
use crate::parsing::parser::Parser;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::os::fd::AsRawFd;
use thiserror::Error;
use tree_sitter::{Node, Parser as TsParser};

#[derive(Error, Debug)]
pub enum NodeConversionError {
    #[error("invalid node type (expected {expected:?}, found {found:?})")]
    InvalidNodeType { expected: String, found: String },
    #[error("missing child of parent {parent:?}: {child:?}")]
    MissingChild { parent: String, child: String },

    #[error("invalid source value (expected {expected:?}, found {found:?})")]
    InvalidSourceValue { expected: String, found: String },
}

#[derive(Debug, Default)]
pub struct TreeSitterParser {}

fn children_iter<'a>(parent: &'a Node) -> impl Iterator<Item = Node<'a>> {
    let num_children = parent.child_count();
    (0..num_children).map(|idx| parent.child(idx).unwrap())
}

fn named_children_map<'a>(parent: &'a Node) -> HashMap<&'static str, Node<'a>> {
    let num_children = parent.child_count();
    (0..num_children)
        .filter_map(|idx| {
            let child = parent.child(idx).unwrap();
            let name = parent.field_name_for_child(idx as u32);
            match name {
                Some(n) => Some((n, child)),
                None => None,
            }
        })
        .collect()
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::DataType {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "primitive_type" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "primitive_type".into(),
                found: node.kind().into(),
            });
        }

        let val = &source[node.start_byte()..node.end_byte()];
        match val {
            "int" => Ok(ast::DataType::Int),
            "char" => Ok(ast::DataType::Char),
            _ => Err(NodeConversionError::InvalidSourceValue {
                expected: "int".into(),
                found: val.into(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::Identifier {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        let val = &source[node.start_byte()..node.end_byte()];
        Ok(ast::Identifier(val.into()))
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::StringLiteral {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        let val = &source[node.start_byte() + 1..node.end_byte() - 1]; // ignore the ""
        Ok(ast::StringLiteral(val.into()))
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::NumberLiteral {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        let val = &source[node.start_byte()..node.end_byte()];
        Ok(ast::NumberLiteral(val.into()))
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::CallExpression {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "call_expression" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "call_expression".into(),
                found: node.kind().into(),
            });
        }

        let named_children = named_children_map(node);

        let function = Expression::try_from((
            named_children
                .get("function")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: node.kind().into(),
                    child: "function".into(),
                })?,
            source,
        ))?;

        let arguments =
            named_children
                .get("arguments")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: node.kind().into(),
                    child: "function".into(),
                })?;

        let num_arguments = arguments.child_count() - 2;
        let arguments_vec = children_iter(arguments)
            .skip(1)
            .take(num_arguments)
            .step_by(2)
            .map(|n| ast::Expression::try_from((&n, source)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CallExpression {
            function: Box::new(function),
            arguments: arguments_vec,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::ExpressionBinary {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        let left = node.child(0).unwrap();
        let op = node.child(1).unwrap();
        let right = node.child(2).unwrap();

        let op_val = &source[op.start_byte()..op.end_byte()];
        let op = ast::BinOp::from_str(op_val);

        Ok(ast::ExpressionBinary {
            left: Box::new(TryFrom::try_from((&left, source))?),
            right: Box::new(TryFrom::try_from((&right, source))?),
            op,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::ParenthesizedExpression {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "parenthesized_expression" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "parenthesized_expression".into(),
                found: node.kind().into(),
            });
        }
        Ok(ParenthesizedExpression {
            expression: Box::new(TryFrom::try_from((&node.child(1).unwrap(), source))?),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::LValue {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "identifier" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "identifier".into(),
                found: node.kind().into(),
            });
        }
        let identifier = Identifier::try_from((node, source))?;
        Ok(ast::LValue::Identifier(identifier))
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::AssignmentExpression {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "assignment_expression" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "assignment_expression".into(),
                found: node.kind().into(),
            });
        }
        let lvalue_node = node.child(0).unwrap();
        let op_node = node.child(1).unwrap();
        let rvalue_node = node.child(2).unwrap();

        let assignment_val = &source[op_node.start_byte()..op_node.end_byte()];
        let assignment_type = ast::AssignmentType::from_str(assignment_val);
        let lvalue = ast::LValue::try_from((&lvalue_node, source))?;
        let rvalue = ast::Expression::try_from((&rvalue_node, source))?;
        Ok(ast::AssignmentExpression {
            lvalue,
            rvalue: Box::new(rvalue),
            atype: assignment_type,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::Expression {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "call_expression" => Ok(ast::Expression::Call(ast::CallExpression::try_from((
                node, source,
            ))?)),
            "identifier" => Ok(ast::Expression::Identifier(ast::Identifier::try_from((
                node, source,
            ))?)),
            "string_literal" => Ok(ast::Expression::StringLiteral(
                ast::StringLiteral::try_from((node, source))?,
            )),
            "number_literal" => Ok(ast::Expression::NumberLiteral(
                ast::NumberLiteral::try_from((node, source))?,
            )),
            "binary_expression" => Ok(ast::Expression::Binary(ast::ExpressionBinary::try_from((
                node, source,
            ))?)),
            "parenthesized_expression" => Ok(ast::Expression::Parenthesized(
                ast::ParenthesizedExpression::try_from((node, source))?,
            )),
            ";" => {
                Ok(ast::Expression::Empty)
            },
            "assignment_expression" => 
                Ok(
                    ast::Expression::Assignment(
                        ast::AssignmentExpression::try_from((node, source))?
                    )
                ),
            _ => Err(NodeConversionError::InvalidNodeType {
                expected: "call_expression | identifier | string_literal | binary_expression | parenthesized_expression"
                    .into(),
                found: node.kind().into(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::ExpressionStatement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "expression_statement" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "expression_statement".into(),
                found: node.kind().into(),
            });
        }

        let main_node = node
            .child(0)
            .ok_or_else(|| NodeConversionError::MissingChild {
                parent: node.kind().into(),
                child: "0".into(),
            })?;
        Ok(ExpressionStatement {
            expression: ast::Expression::try_from((&main_node, source))?,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::ReturnStatement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "return_statement" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "return_statement".into(),
                found: node.kind().into(),
            });
        }

        let main_node = node
            .child(1)
            .ok_or_else(|| NodeConversionError::MissingChild {
                parent: node.kind().into(),
                child: "0".into(),
            })?;
        Ok(ReturnStatement {
            expression: ast::Expression::try_from((&main_node, source))?,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::Declaration {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "declaration" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "declaration".into(),
                found: node.kind().into(),
            });
        }

        let named_children = named_children_map(node);

        let data_type = ast::DataType::try_from((
            named_children
                .get("type")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "declaration".into(),
                    child: "type".into(),
                })?,
            source,
        ))?;

        let declarator = ast::Declarator::try_from((
            named_children
                .get("declarator")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "declaration".into(),
                    child: "declarator".into(),
                })?,
            source,
        ))?;

        let storage_specifier = children_iter(node)
            .find(|n| n.kind() == "storage_class_specifier")
            .map(|n| {
                let val = &source[n.start_byte()..n.end_byte()];
                match val {
                    "extern" => StorageClass::Extern,
                    "auto" => StorageClass::Auto,
                    _ => todo!(),
                }
            })
            .or_else(|| Some(StorageClass::Auto))
            .unwrap();

        Ok(ast::Declaration {
            storage_class: storage_specifier,
            dtype: data_type,
            declarator: Box::new(declarator),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::Statement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "expression_statement" => Ok(ast::Statement::ExpressionStatement(
                ast::ExpressionStatement::try_from((node, source))?,
            )),
            "return_statement" => Ok(ast::Statement::ReturnStatement(
                ast::ReturnStatement::try_from((node, source))?,
            )),
            "declaration" => Ok(ast::Statement::Declaration(ast::Declaration::try_from((
                node, source,
            ))?)),
            "compound_statement" => Ok(ast::Statement::CompoundStatement(
                ast::CompoundStatement::try_from((node, source))?,
            )),
            "if_statement" => Ok(ast::Statement::IfStatement(
                ast::IfStatement::try_from((node, source))?,
            )),
            "while_statement" => Ok(ast::Statement::WhileStatement(
                ast::WhileStatement::try_from((node, source))?,
            )),
            _ => Err(NodeConversionError::InvalidNodeType {
                expected:
                    "exrpession_statement | return_statement | declaration | compound_statement | if_statement"
                        .into(),
                found: node.kind().into(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::CompoundStatement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "compound_statement" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "compound_statement".into(),
                found: node.kind().into(),
            });
        }

        let num_arguments = node.child_count() - 2;

        Ok(CompoundStatement {
            items: children_iter(node)
                .skip(1) // ignore opening {
                .take(num_arguments) // ignore closing }
                .map(|c| ast::Statement::try_from((&c, source)))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::IfStatement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "if_statement" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "if_statement".into(),
                found: node.kind().into(),
            });
        }

        let cond_child = ParenthesizedExpression::try_from((&node.child(1).unwrap(), source))?;
        let body_child = ast::Statement::try_from((&node.child(2).unwrap(), source))?;
        let else_body_child = node
            .child(3)
            .map(|n| Box::new(ast::Statement::try_from((&n.child(1).unwrap(), source)).unwrap()));

        Ok(ast::IfStatement {
            condition: cond_child,
            body: Box::new(body_child),
            else_body: else_body_child,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::WhileStatement {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "while_statement" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "while_statement".into(),
                found: node.kind().into(),
            });
        }

        let cond_child = ParenthesizedExpression::try_from((&node.child(1).unwrap(), source))?;
        let body_child = ast::Statement::try_from((&node.child(2).unwrap(), source))?;

        Ok(ast::WhileStatement {
            condition: cond_child,
            body: Box::new(body_child),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::PointerDeclarator {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "pointer_declarator" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "pointer_declarator".into(),
                found: node.kind().into(),
            });
        }

        let main_node = node
            .child(1)
            .ok_or_else(|| NodeConversionError::MissingChild {
                parent: node.kind().into(),
                child: "1".into(),
            })?;
        Ok(PointerDeclarator {
            declarator: Box::new(ast::Declarator::try_from((&main_node, source))?),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::InitDeclarator {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "init_declarator" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "init_declarator".into(),
                found: node.kind().into(),
            });
        }

        let named_children = named_children_map(node);

        let value = ast::Expression::try_from((
            named_children
                .get("value")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "init_declarator".into(),
                    child: "value".into(),
                })?,
            source,
        ))?;

        let declarator = ast::Declarator::try_from((
            named_children
                .get("declarator")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "init_declarator".into(),
                    child: "type".into(),
                })?,
            source,
        ))?;

        Ok(ast::InitDeclarator {
            value,
            declarator: Box::new(declarator),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::Declarator {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "identifier" => Ok(ast::Declarator::Identifier(ast::Identifier::try_from((node, source))?)),
            "function_declarator" => Ok(ast::Declarator::FunctionDeclarator(ast::FunctionDeclarator::try_from((node, source))?)),
            "pointer_declarator" => Ok(ast::Declarator::PointerDeclarator(ast::PointerDeclarator::try_from((node, source))?)),
            "init_declarator" => Ok(ast::Declarator::InitDeclarator(ast::InitDeclarator::try_from((node, source))?)),
            _ => Err(NodeConversionError::InvalidNodeType { expected: "one of 'identifier' | 'function_declarator' | 'init_declarator' | 'pointer_declarator'".into(), found: node.kind().to_owned() })
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::ParameterDeclaration {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "parameter_declaration" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "parameter_declaration".into(),
                found: node.kind().into(),
            });
        }

        let named_children = named_children_map(node);

        let data_type = ast::DataType::try_from((
            named_children
                .get("type")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "parameter_declaration".into(),
                    child: "type".into(),
                })?,
            source,
        ))?;

        let declarator = ast::Declarator::try_from((
            named_children
                .get("declarator")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "parameter_declaration".into(),
                    child: "type".into(),
                })?,
            source,
        ))?;

        Ok(ast::ParameterDeclaration {
            dtype: data_type,
            declarator: Box::new(declarator),
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::FunctionParameter {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "parameter_declaration" => Ok(ast::FunctionParameter::ParameterDeclaration(
                ast::ParameterDeclaration::try_from((node, source))?,
            )),
            "variadic_parameter" => Ok(ast::FunctionParameter::VariadicParameter),
            _ => Err(NodeConversionError::InvalidNodeType {
                expected: "one of 'parameter_declaration' | 'variadic_parameter'".into(),
                found: node.kind().to_owned(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::FunctionDeclarator {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "function_declarator" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "function_declarator".into(),
                found: node.kind().into(),
            });
        }

        let named_children = named_children_map(node);

        let declarator = ast::Declarator::try_from((
            named_children
                .get("declarator")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "function_declarator".into(),
                    child: "declarator".into(),
                })?,
            source,
        ))?;

        let parameters =
            named_children
                .get("parameters")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: node.kind().into(),
                    child: "parameters".into(),
                })?;
        let num_parameters = parameters.child_count() - 2;

        let parameters_vec = children_iter(parameters)
            .skip(1)
            .take(num_parameters)
            .step_by(2)
            .map(|n| ast::FunctionParameter::try_from((&n, source)))
            .collect::<Result<Vec<_>, _>>()?;

        let is_variadic = parameters_vec
            .iter()
            .find(|x| matches!(x, ast::FunctionParameter::VariadicParameter))
            .is_some();

        Ok(ast::FunctionDeclarator {
            declarator: Box::new(declarator),
            parameters: parameters_vec,
            is_variadic,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::FunctionDeclaratorField {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "function_declarator" => Ok(ast::FunctionDeclaratorField::FunctionDeclarator(
                ast::FunctionDeclarator::try_from((node, source))?,
            )),
            "pointer_declarator" => Ok(ast::FunctionDeclaratorField::PointerDeclarator(
                ast::PointerDeclarator::try_from((node, source))?,
            )),
            _ => Err(NodeConversionError::InvalidNodeType {
                expected: "one of 'function_declarator' | 'pointer_declarator'".into(),
                found: node.kind().to_owned(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::FunctionDefinition {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "function_definition" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "function_definition".into(),
                found: node.kind().to_owned(),
            });
        }

        let named_children = named_children_map(&node);

        let return_type = ast::DataType::try_from((
            named_children
                .get("type")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "function_definition".into(),
                    child: "type".into(),
                })?,
            source,
        ))?;

        let body = ast::CompoundStatement::try_from((
            named_children
                .get("body")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "function_definition".into(),
                    child: "body".into(),
                })?,
            source,
        ))?;

        let declarator_node =
            named_children
                .get("declarator")
                .ok_or_else(|| NodeConversionError::MissingChild {
                    parent: "function_definition".into(),
                    child: "declarator".into(),
                })?;

        let declarator = ast::FunctionDeclaratorField::try_from((declarator_node, source))?;

        Ok(ast::FunctionDefinition {
            return_type,
            body,
            declarator,
        })
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::TopLevelItem {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        match node.kind() {
            "function_definition" => Ok(ast::TopLevelItem::FunctionDefinition(
                ast::FunctionDefinition::try_from((node, source))?,
            )),
            "declaration" => Ok(ast::TopLevelItem::Declaration(ast::Declaration::try_from(
                (node, source),
            )?)),
            _ => Err(NodeConversionError::InvalidNodeType {
                expected: "one of 'function_definition' | 'declaration'".into(),
                found: node.kind().to_owned(),
            }),
        }
    }
}

impl<'a> TryFrom<(&'a Node<'a>, &'a str)> for ast::TranslationUnit {
    type Error = NodeConversionError;

    fn try_from((node, source): (&'a Node<'a>, &'a str)) -> Result<Self, Self::Error> {
        if node.kind() != "translation_unit" {
            return Err(NodeConversionError::InvalidNodeType {
                expected: "translation_unit".into(),
                found: node.kind().to_owned(),
            });
        }

        return Ok(ast::TranslationUnit {
            items: children_iter(&node)
                .map(|n| TopLevelItem::try_from((&n, source)))
                .collect::<Result<Vec<_>, _>>()?,
        });
    }
}

impl Parser for TreeSitterParser {
    fn parse(&self, source_code: &str) -> ast::TranslationUnit {
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .expect("Error loading C grammar");
        let tree = parser.parse(source_code, None).unwrap();
        tree.print_dot_graph(&fs::File::create("foo.dot").unwrap().as_raw_fd());
        let root_node = tree.root_node();
        ast::TranslationUnit::try_from((&root_node, source_code)).unwrap()
    }
}
