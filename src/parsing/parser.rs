use crate::parsing::ast;

pub trait Parser {
    fn parse(&self, source_code: &str) -> ast::TranslationUnit;
}
