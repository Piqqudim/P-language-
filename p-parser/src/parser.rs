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

    fn expect(&mut self, kind : TokenKind) -> Result<Token,ParseError>{
        if self.check(&kind) {
            Ok(self.advance())

        } else {
            Err(self.err(format!("expected {kind}, found {}", self.peek())))
        }
    }

    fn expect_ident(&mut self)-> Result<String,ParseError>{
        match self.peek().clone(){
            TokenKind::Ident(s)=>{
                self.advance();
                Ok(s)
            }
            other => Err(self.err(format!("expected identifier, found {other}")))
        }
    }

    // extern/ route target parsing needs bare string literals
    // outside of ordinary expression position(a URL, a package name, an export name)- this is the helper
    fn expect_string(&mut self)-> Result<String,ParseError>{
        match self.advance().kind{
            TokenKind::Str(s) => Ok(s),
            other=> Err(self.err(format!("expected a string, found {other}")))
        }
    } 

    fn skip_newlines(&mut self){
        while matches!(self.peek(),TokenKind::Newline){
            self.advance();
        }
    }

    fn parse_module(&mut self) -> Result<Module,ParseError>{
        let mut items = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), TokenKind::Eof){
            items.push(self.parse_top_level_item()?);
            self.skip_newlines();
        }
        Ok(Module { items })
    }

    fn parse_top_level_item(&mut self )-> Result<TopLevelItem,ParseError>{
        match self.peek().clone(){
            
        }
    }


    fn parse_import(&mut self)-> Result<TopLevelItem,ParseError>{
        self.expect(TokenKind::Import)?;
        let mut parts = vec![self.expect_ident()?];
        while self.eat(&TokenKind::Slash){
            parts.push(self.expect_ident()?);
        }
        self.expect(TokenKind::Newline)?;
        Ok(TopLevelItem::Import(parts))
    }

    fn parse_enum(&mut self)-> Result<EnumDecl, ParseError>{
        self.expect(TokenKind::Enum)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let mut variants = Vec::new();
        while let TokenKind::Ident(_) = self.peek(){
            variants.push(self.expect_ident()?);
            self.expect(TokenKind::Newline)?;
        }
        if variants.is_empty(){
            return Err(self.err("enum must declare at least one variant"));
        }
        self.expect(TokenKind::Dedent)?;
        Ok(EnumDecl { name, name_span, variants })
    }

    fn parse_struct_decl(&mut self)-> Result<StructDecl,ParseError>{
        self.expect(TokenKind::Struct)?;
        let name_span=  self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let mut fields = Vec::new();
        while let TokenKind::Ident(_)  = self.peek(){
            let fname = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let fty = self.parse_type_expr()?;
            self.expect(TokenKind::Newline)?;
            fields.push((fname,fty))
        }
        if fields.is_empty(){
            return Err(self.err("struct must declare at least one field"));
        }
        self.expect(TokenKind::Dedent)?;
       Ok(StructDecl { name, name_span, fields })
    }

    fn parse_state_decl(&mut self) -> Result<StateDecl, ParseError>{
        self.expect(TokenKind::State)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        Ok(StateDecl { name, name_span, ty, value })
    }

    fn parse_params(&mut self)-> Result<Vec<Param>,ParseError>{
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen){
            loop{
                let name_span = self.peek_span();
                let name = self.expect_ident()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type_expr()?;
                params.push(Param{name,name_span,ty});
                if !self.eat(&TokenKind::Comma){
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(params)
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl,ParseError>{
        self.expect(TokenKind::Fn)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        let params = self.parse_params()?;
        let ret = if self.eat(&TokenKind::Return) {
            Some(self.parse_type_expr()?)
        }
        else {
            None
        };
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::Dedent){
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::Dedent)?;
        if body.is_empty(){
            return Err(self.err("function body must contain at least one statement"));
        }
        Ok(FnDecl { name, name_span, params, ret, body })

    }

    fn parse_component_decl(&mut self) -> Result<ComponentDecl,ParseError>{
        self.expect(TokenKind::Component)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        let params = self.parse_params()?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let mut state_decls = Vec::new();
        let mut fns = Vec::new();
        while matches!(self.peek(),TokenKind::State){
            state_decls.push(self.parse_state_decl()?);
        }
        while matches!(self.peek(),TokenKind::Fn){
            fns.push(self.parse_fn_decl()?);
        }
        let mut nodes = Vec::new();
        while !self.check(&TokenKind::Dedent){
            nodes.push(self.parse_ui_node()?);
        }

        self.expect(TokenKind::Dedent)?;
        if nodes.is_empty(){
            return Err(self.err("component must have at least one root UI node"));
        }
        Ok(ComponentDecl { name, name_span, params, state_decls, fns, root: nodes })
    }

}