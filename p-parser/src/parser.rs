//Recursive descent parser with precedence climbing for expressions
//Consume tokens and produces the CST from cst.rs
//Current through everything



use p_lexer::{Span, Token, TokenKind, token::FileId};
use crate::cst::*;
use std::fmt;

#[derive(Debug, Clone,PartialEq)]
pub struct  ParseError {
    pub file: FileId,
    pub message : String,
    pub line : u32,
    pub col : u32,
}

impl fmt::Display for ParseError{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)-> fmt::Result{
        write!(f,"{}:{}: {}", self.line,self.col,self.message)
    }
}
impl std::error::Error for ParseError{}

pub fn parse(tokens:&[Token]) -> Result<Module,ParseError>{
    
}

struct Parser<'t> {
    tokens: &'t[Token],
    last_span: Span,
    pos: usize
}

impl<'t> Parser<'t> {
    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }
}