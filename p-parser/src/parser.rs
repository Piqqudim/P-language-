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
    fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }
    fn err(&self, msg: impl  Into<String>) -> ParseError{
        let s = self.peek_span();
        ParseError { file: s.file, message: msg.into(), line: s.line, col: s.col }
    }

    fn advance(&mut self)-> Token {
        let t = self.tokens[self.pos].clone();
        self.last_span = t.span;
        if self.pos + 1 < self.tokens.len(){
            self.pos += 1;
        }
        t
    }

    fn check(&self, kind: &TokenKind)-> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
    }
    //Let eat here hahahahaha
    fn eat(&mut self, kind : &TokenKind)-> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind : &TokenKind) -> Result<Token,ParseError>{
        if self.check(&kind) {
            Ok(self.advance())

        } else {
            Err(self.err(format!("expected {kind}, found {}", self.peek())))
        }
    }

    fn expect_indent(&mut self)-> Result<String,ParseError>{
        match self.peek().clone(){
            TokenKind::Ident(s)=>{
                self.advance();
                Ok(s)
            }
            other => Err(self.err(format!("expected identifier, found {other}")))
        }
    }

}