//Recursive descent parser with precedence climbing for expressions
//Consume tokens and produces the CST from cst.rs
//Current through everything



use p_lexer::{Span, Token, TokenKind, token::FileId};
use crate::cst::*;
use std::fmt::{self};

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
    let mut p = Parser { tokens,pos:0, last_span: tokens[0].span};
    p.parse_module()
    
    
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
        
         dbg!(&self.tokens);
        let mut items = Vec::new();
        self.skip_newlines();
       while !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                     if self.check(&TokenKind::Eof) {
                  break;
                     }
                items.push(self.parse_top_level_item()?);
                }
         Ok(Module { items })
    }

    fn parse_top_level_item(&mut self )-> Result<TopLevelItem,ParseError>{
        match self.peek().clone(){
            TokenKind::Import => self.parse_import(),
            TokenKind::Enum => self.parse_enum().map(TopLevelItem::Enum),
            TokenKind::Struct => self.parse_struct_decl().map(TopLevelItem::Struct),
            TokenKind::State => self.parse_state_decl().map(TopLevelItem::State),
            TokenKind::Fn => self.parse_fn_decl().map(TopLevelItem::Fn),
            TokenKind::Component => self.parse_component_decl().map(TopLevelItem::Component),
            TokenKind::Layout => self.parse_layout_decl().map(TopLevelItem::Layout),
            TokenKind::Page => self.parse_page_decl().map(TopLevelItem::Page),
            TokenKind::Route => self.parse_route_decl().map(TopLevelItem::Route),
            TokenKind::Store => self.parse_store_decl().map(TopLevelItem::Store),
            TokenKind::Extern => self.parse_extern_decl().map(TopLevelItem::Extern),
            TokenKind::Test => self.parse_test_decl().map(TopLevelItem::Test),
            other  => Err(self.err(format!("expected a top-level item(import/enum/struct/state/fn/component/layout/page/route/store/extern/test), found {other}")))

            
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
        let ret = if self.eat(&TokenKind::Arrow) {
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

    fn parse_layout_decl(&mut self)-> Result<LayoutDecl,ParseError>{
        self.expect(TokenKind::Layout)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let root = self.parse_ui_node()?;
        self.expect(TokenKind::Dedent)?;
        Ok(LayoutDecl { name, name_span, root })

    }

    fn parse_page_decl(&mut self) -> Result<PageDecl,ParseError>{
        self.expect(TokenKind::Page)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        let uses = if self.eat(&TokenKind::Uses){Some(self.expect_ident()?)} else {None};
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
            return Err(self.err("page must have at least one root UI node"));
        }
        Ok(PageDecl { name, name_span, uses, state_decls, fns, root:nodes })
    }

    fn parse_route_decl(&mut self)-> Result<RouteDecl,ParseError>{
        self.expect(TokenKind::Route)?;
        let method_span = self.peek_span();
        let method = match self.advance().kind{
            TokenKind::Get => HttpMethod::Get,
            TokenKind::Post => HttpMethod::Post,
            TokenKind::Put => HttpMethod::Put,
            TokenKind::Delete => HttpMethod::Delete,
            TokenKind::Patch => HttpMethod::Patch,
            other => return Err(self.err(format!("expected an HTTP method , found {other}")))

        };
        let path = self.expect_string()?;
        let body_ty = if self.eat(&TokenKind::Body){ 
             self.expect(TokenKind::Colon)?;
             Some(self.parse_type_expr()?)} else { None};
        self.expect(TokenKind::Arrow)?;
        let ret = self.parse_type_expr()?;
        self.expect(TokenKind::Newline)?;
        self.expect(TokenKind::Indent)?;
        let mut body = Vec::new();
        while !self.check(&TokenKind::Dedent){
            body.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::Dedent)?;
        if body.is_empty(){
            return Err(self.err("route handler must contain at least one statement"));
        }
        Ok(RouteDecl { method, method_span, path, body_ty, ret, body})
        }

        fn parse_store_decl(&mut self) -> Result<StoreDecl,ParseError>{
            self.expect(TokenKind::Store)?;
            let name_span = self.peek_span();
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type_expr()?;
            self.expect(TokenKind::Newline)?;
            Ok(StoreDecl { name, name_span, ty})
        }

        // Added JS interoperability
        fn parse_extern_decl(&mut self)-> Result<ExternDecl,ParseError>{
            self.expect(TokenKind::Extern)?;
            self.expect(TokenKind::Fn)?;
            let name_span= self.peek_span();
            let name = self.expect_ident()?;
            let params = self.parse_params()?;
            let ret = if self.eat(&TokenKind::Arrow) {Some(self.parse_type_expr()?)} else { None};
            let target = match self.advance().kind{
                TokenKind::Client => match self.advance().kind{
                    TokenKind::Global => {
                        let global_name = self.expect_string()?;
                        ExternTarget::ClientGlobal { name: global_name }
                    }
                    TokenKind::Module =>{
                        let url = self.expect_string()?;
                        let export = if self.eat(&TokenKind::As){Some(self.expect_string()?)} else { None };
                        ExternTarget::ClientModule { url, export }
                    }
                    other => return Err(self.err(format!("expected 'global'or 'module', found {other}")))
                },
                TokenKind::Server => {
                    self.expect(TokenKind::Npm)?;
                    let package = self.expect_string()?;
                    let export = if self.eat(&TokenKind::As){Some(self.expect_string()?)} else { None};
                    ExternTarget::ServerNpm { package, export }
                }
                other => return Err(self.err(format!("expected 'client' or 'server', found {other} "))),

            };
            self.expect(TokenKind::Newline)?;
            Ok(ExternDecl { name, name_span, params, ret, target })
        }

        fn parse_test_decl(&mut self)-> Result<TestDecl,ParseError>{
            self.expect(TokenKind::Test)?;
            let description_span = self.peek_span();
            let description = self.expect_string()?;
            self.expect(TokenKind::Newline)?;
            self.expect(TokenKind::Indent)?;
            let mut body = Vec::new();

            while !self.check(&TokenKind::Dedent){
                body.push(self.parse_stmt()?);
            }
            self.expect(TokenKind::Dedent)?;
            Ok(TestDecl { description, description_span, body })
        }
      

        fn node_kind_from_token(&self)->Option<NodeKind>{
            Some(match self.peek(){
                TokenKind::Row => NodeKind::Row,
                TokenKind::Column => NodeKind::Column,
                TokenKind::Stack => NodeKind::Stack,
                TokenKind::Container => NodeKind::Container,
                TokenKind::Card => NodeKind::Card,
                TokenKind::Grid => NodeKind::Grid,
                TokenKind::List => NodeKind::List,
                TokenKind::Text => NodeKind::Text,
                TokenKind::Image => NodeKind::Image,
                TokenKind::Icon => NodeKind::Icon,
                TokenKind::Input => NodeKind::Input,
                TokenKind::Textarea => NodeKind::Textarea,
                TokenKind::Button => NodeKind::Button,
                TokenKind::Checkbox => NodeKind::Checkbox,
                TokenKind::Switch => NodeKind::Switch,
                TokenKind::Radio => NodeKind::Radio,
                TokenKind::Dropdown => NodeKind::Dropdown,
                TokenKind::Table => NodeKind::Table,
                TokenKind::Navigation => NodeKind::Navigation,
                TokenKind::Tabs => NodeKind::Tabs,
                TokenKind::Dialog => NodeKind::Dialog,
                TokenKind::Modal => NodeKind::Modal,
                TokenKind::Menu => NodeKind::Menu,
                TokenKind::Slot => NodeKind::Slot,
                _ => return None

            })
        }
        fn parse_ui_node(&mut self)->Result<UiNode,ParseError>{

            let start = self.peek_span();
            if let Some(kind) = self.node_kind_from_token(){
                self.advance();
                let inline_arg = if matches!(self.peek(),TokenKind::Newline){ None} else { Some(self.parse_expr()?)};
                self.expect(TokenKind::Newline)?;
                let body = self.parse_node_body()?;

                let span = start.to(self.last_span);
                return Ok(UiNode::Kind { kind, inline_arg, body, span });
            }
            if let TokenKind::Ident(name) = self.peek().clone(){
                self.advance();
                self.expect(TokenKind::LParen)?;
                let args = self.parse_arg_list()?;
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::Newline)?;
                let body = self.parse_node_body()?;
                let span = start.to(self.last_span);
                return Ok(UiNode::Call { name, args, body, span })
            }
            Err(self.err(format!("expect a UI node , found {}", self.peek())))
        }

        fn parse_node_body(&mut self) -> Result<Vec<NodeBodyItem>,ParseError>{
            if !self.eat(&TokenKind::Indent){
                return Ok(Vec::new());
            }
            let mut items = Vec::new();
            while !self.check(&TokenKind::Dedent){
                items.push(self.parse_node_body_item()?);

            }

            self.expect(TokenKind::Dedent)?;
            Ok(items)

        }
        fn parse_node_body_item(&mut self)->Result<NodeBodyItem,ParseError>{

            match self.peek().clone(){
                TokenKind::On => self.parse_event_stmt().map(NodeBodyItem::Event),
                TokenKind::If => self.parse_if_node().map(NodeBodyItem::If),
                TokenKind::For => self.parse_for_node().map(NodeBodyItem::For),
                _ if self.node_kind_from_token().is_some() => self.parse_ui_node().map(NodeBodyItem::Node),
                TokenKind::Ident(_) if self.is_component_call_ahead() => {
                    self.parse_ui_node().map(NodeBodyItem::Node)
                }
                TokenKind::Ident(_) => self.parse_property_stmt().map(NodeBodyItem::Property),
                other => Err(self.err(format!("expected a property, event or child node, found{other}")))
            }
        }
        fn is_component_call_ahead(&self) -> bool {
            matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::LParen))
        }

        fn parse_property_stmt(&mut self)-> Result<PropertyStmt,ParseError>{
            let start = self.peek_span();
            let name = self.expect_ident()?;
            let mut values = vec![self.parse_expr()?];
            while values.len() < 4 && !matches!(self.peek(),TokenKind::Newline){
                values.push(self.parse_expr()?);
            }
            self.expect(TokenKind::Newline)?;
            let span = start.to(self.last_span);

            Ok(PropertyStmt { name, values, span })
        }

        fn parse_event_stmt(&mut self)->Result<EventStmt,ParseError>{
            self.expect(TokenKind::On)?;
            let event = self.expect_ident()?;
            let handler = if matches!(self.peek(),TokenKind::LParen) && self.looks_like_lambda_params(){
                self.expect(TokenKind::LParen)?;
                let mut params = Vec::new();
                if !self.check(&TokenKind::RParen){
                    loop{
                        params.push(self.expect_ident()?);
                        if !self.eat(&TokenKind::Comma){
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen)?;
                self.expect(TokenKind::FatArrow)?;
                let body = if matches!(self.peek(),TokenKind::Ident(_)) && self.is_assignment_ahead(){
                    let target = self.parse_lvalue()?;
                    self.expect(TokenKind::Assign)?;
                    let value = self.parse_expr()?;
                    LambdaBody::Assign { target, value }
                }
                else {
                    LambdaBody::Expr(self.parse_expr()?)
                };
                EventHandler::Lambda { params, body }
            } else {
                EventHandler::Call(self.parse_expr()?)
            };
            self.expect(TokenKind::Newline)?;
            Ok(EventStmt { name: event , handler: handler })
        }

        fn looks_like_lambda_params(&self)-> bool{
            let mut depth = 0i32;
            let mut i = self.pos;
            loop {
                match self.tokens.get(i).map(|t|&t.kind){
                    Some(TokenKind::LParen) => depth +=1,
                    Some(TokenKind::RParen) =>{
                        depth -=1;
                        if depth == 0 {
                            return matches!(self.tokens.get(i+1).map(|t| &t.kind), Some(TokenKind::FatArrow));
                        }
                    }
                    None | Some(TokenKind::Eof) => return false,
                    _ => {}
                }
                i +=1;
            }
        }

        fn parse_if_node(&mut self) -> Result<IfNode,ParseError>{
            let start = self.peek_span();
            self.expect(TokenKind::If)?;
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Newline)?;
            self.expect(TokenKind::Indent)?;
            let mut then_branch = Vec::new();
            while !self.check(&TokenKind::Dedent){
                then_branch.push(self.parse_ui_node()?);
            }
            self.expect(TokenKind::Dedent)?;
            let else_branch = if self.eat(&TokenKind::Else){
                self.expect(TokenKind::Newline)?;
                self.expect(TokenKind::Indent)?;
                let mut nodes = Vec::new();
                while !self.check(&TokenKind::Dedent){
                    nodes.push(self.parse_ui_node()?);
                }
                self.expect(TokenKind::Dedent)?;
                Some(nodes)
            } 
            else {
                None
            };
            let span = start.to(self.last_span);
            Ok(IfNode { cond, then_branch, else_branch, span })
        }


        fn parse_for_node(&mut self) -> Result<ForNode,ParseError>{
            let start = self.peek_span();
            self.expect(TokenKind::For)?;
            let var_span = self.peek_span();
            let var = self.expect_ident()?;
            self.expect(TokenKind::In)?;
            let iter = self.parse_expr()?;
            self.expect(TokenKind::Newline)?;
            self.expect(TokenKind::Indent)?;
            let mut body = Vec::new();
            while !self.check(&TokenKind::Dedent){
                body.push(self.parse_ui_node()?)
            }
            self.expect(TokenKind::Dedent)?;
            let span = start.to(self.last_span);
            Ok(ForNode { var, var_span, iter, body, span })
        }

        fn parse_stmt(&mut self) ->Result<Stmt,ParseError>{
            match self.peek().clone(){
                TokenKind::Let => self.parse_let_stmt(),
                TokenKind::If => self.parse_if_stmt(),
                TokenKind::For => self.parse_for_stmt(),
                TokenKind::While => self.parse_while_stmt(),
                TokenKind::Return => self.parse_return_stmt(),
                TokenKind::Assert => self.parse_assert_stmt(),
                TokenKind::Ident(_) if self.is_assignment_ahead() => self.parse_assign_stmt(),
                _=> {
                    let e = self .parse_expr()?;
                    self.expect(TokenKind::Newline)?;
                    Ok(Stmt::Expr(e))

                }
            }



    }

    fn parse_assert_stmt(&mut self)-> Result<Stmt,ParseError>{
        let start = self.peek_span();
        self.expect(TokenKind::Assert)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        let span = start.to(self.last_span);
        Ok(Stmt::Assert { expr, span })
    }

    fn is_assignment_ahead(&self) -> bool {
        let mut i = self.pos + 1;
        loop {
            match self.tokens.get(i).map(|t|&t.kind){
                Some(TokenKind::Dot) => {
                    i +=2;
                }
                Some(TokenKind::LBracket) => {
                    let mut depth = 1i32;
                    i +=1;
                    while depth > 0 {
                        match self.tokens.get(i).map(|t| &t.kind){
                            Some(TokenKind::LBracket) => depth += 1,
                            Some(TokenKind::RBracket) => depth +=1,
                            None => return false,
                            _ => {}
                        }
                        i += 1;
                    }
                }
                Some(TokenKind::Assign) => return true,
                _=> return false,
            }
        }
    }
    fn parse_lvalue(&mut self)-> Result<LValue,ParseError>{
        let name = self.expect_ident()?;
        let mut accessors = Vec::new();
        loop {
            if self.eat(&TokenKind::Dot){
                accessors.push(Accessor::Field(self.expect_ident()?));

            }
            else if self.eat(&TokenKind::LBracket){

                let idx = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                accessors.push(Accessor::Index(idx));

            } else {
                break;
            }
        }
        Ok(LValue { name, accessors })
    }

    fn parse_assign_stmt(&mut self) -> Result<Stmt,ParseError>{
        let target = self.parse_lvalue()?;
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        Ok(Stmt::Assign { target, value })
    }

    fn parse_let_stmt(&mut self)-> Result<Stmt,ParseError>{
        self.expect(TokenKind::Let)?;
        let name_span = self.peek_span();
        let name = self.expect_ident()?;
        let ty = if self.eat(&TokenKind::Colon) {Some(self.parse_type_expr()?)} else {None};
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        Ok(Stmt::Let { name, name_span, ty, value })
    }

    fn parse_stmt_block(&mut self) -> Result<Vec<Stmt>,ParseError>{
        self.expect(TokenKind::Indent)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::Dedent){
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::Dedent)?;
        Ok(stmts)
    }

    fn parse_if_stmt(&mut self)-> Result<Stmt,ParseError>{
        self.expect(TokenKind::If)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        let then_branch = self.parse_stmt_block()?;
        let else_branch = if self.eat(&TokenKind::Else){
            self.expect(TokenKind::Newline)?;
            Some(self.parse_stmt_block()?)
        } else {
            None
        };
        Ok(Stmt::If { cond, then_branch, else_branch })
    }

    fn parse_for_stmt(&mut self)-> Result<Stmt,ParseError>{
        self.expect(TokenKind::For)?;
        let var_span =self.peek_span();
        let var = self.expect_ident()?;
        self.expect(TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        let body = self.parse_stmt_block()?;
        Ok(Stmt::For { var, var_span, iter, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt,ParseError>{
        self.expect(TokenKind::While)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::Newline)?;
        let body = self.parse_stmt_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_return_stmt(&mut self)-> Result<Stmt,ParseError>{
        self.expect(TokenKind::Return)?;
        let value = if matches!(self.peek(), TokenKind::Newline){ None } else {Some(self.parse_expr()?)};
        self.expect(TokenKind::Newline)?;
        Ok(Stmt::Return(value))

        
    }

    fn parse_expr(&mut self)-> Result<Expr,ParseError>{
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr,ParseError>{
        let mut lhs = self.parse_and()?;
        while self.eat(&TokenKind::Or){
            let rhs = self.parse_and()?;
            let span = lhs.span.to(rhs.span);
            lhs = Expr{kind:ExprKind::Binary { op: BinaryOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span};
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr,ParseError>{
        let mut lhs = self.parse_not()?;
        while self.eat(&TokenKind::And){
            let rhs = self.parse_not()?;
            let span = lhs.span.to(rhs.span);
            lhs = Expr{ kind : ExprKind::Binary { op: BinaryOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs) },span};
        }
        Ok(lhs)
    }
    fn parse_not(&mut self)-> Result<Expr,ParseError>{
        if matches!(self.peek(),TokenKind::Not){
            let start = self.peek_span();
            self.advance();
            let e = self.parse_not()?;
            let span= start.to(e.span);
            return Ok(Expr { kind: ExprKind::Unary { op: UnaryOp::Not, expr: Box::new(e) }, span });
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self)-> Result<Expr,ParseError>{
        let lhs = self.parse_add()?;
        let op = match self.peek(){
            TokenKind::EqEq => BinaryOp::Eq,
            TokenKind::NotEq => BinaryOp::NotEq,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::LtEq => BinaryOp::LtEq,
            TokenKind::GtEq => BinaryOp::GtEq,
            _=> return Ok(lhs),
        };
        self.advance();
        let rhs = self.parse_add()?;
        let span = lhs.span.to(rhs.span);
        Ok(Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span })
    }

    fn parse_add(&mut self) -> Result<Expr,ParseError>{
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek(){
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _=> break,
            };
            self.advance();
            let rhs = self.parse_mul()?;
            let span = lhs.span.to(rhs.span);
            lhs = Expr { kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Ok(lhs)
    }
    fn parse_mul(&mut self) -> Result<Expr,ParseError>{
        let mut lhs = self.parse_unary()?;
        loop 
            {
                let op = match self.peek() {
                    TokenKind::Star => BinaryOp::Mul,
                    TokenKind::Slash => BinaryOp::Div,
                    TokenKind::Percent => BinaryOp::Mod,
                    _=> break,
                };
                self.advance();
                let rhs = self.parse_unary()?;
                let span = lhs.span.to(rhs.span);
                lhs = Expr {kind : ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span};
            }
            Ok(lhs)
        }

        fn parse_unary(&mut self)->Result<Expr,ParseError>{
            if matches!(self.peek(),TokenKind::Minus){
                let start = self.peek_span();
                self.advance();  
                let e = self.parse_unary()?;
                let span = start.to(e.span);
                return Ok(Expr { kind: ExprKind::Unary { op: UnaryOp::Neg, expr: Box::new(e) }, span });
            }
            self.parse_postfix()
        }

        fn parse_postfix(&mut self) -> Result<Expr,ParseError>{
            let start = self.peek_span();
            let mut e = self.parse_primary()?;
            loop {
                 if self.eat(&TokenKind::Dot){
                    let name = self.expect_ident()?;
                    let span = start.to(self.last_span);
                    e = Expr{kind:ExprKind::Field { base: Box::new(e), name },span};
                 }
                 else if self.eat(&TokenKind::LBracket) {
                    let idx = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    let span = start.to(self.last_span);
                    e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(idx) }, span };

                 }
                 else if matches!(self.peek(),TokenKind::LParen){
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(TokenKind::RParen)?;
                    let span = start.to(self.last_span);
                    e = Expr { kind: ExprKind::Call { callee: Box::new(e), args }, span };

                 } else {
                    break;
                 }
            }
            Ok(e)
        }

    fn parse_arg_list(&mut self)-> Result<Vec<Arg>,ParseError>{
        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen){
            loop{
                let name = match self.peek().clone(){
                    TokenKind::Ident(n)=> {
                        let is_named = matches!(self.tokens.get(self.pos + 1) , Some(token) if matches!(token.kind,TokenKind::Colon));
                        if is_named {
                            self.advance(); //identifier
                            self.advance(); // colon
                            Some(n)
                        } else {
                            None
                        }
                    }
                    _=> None,
                };
                let value = self.parse_expr()?;
                args.push(Arg{name,value});
                if !self.eat(&TokenKind::Comma){
                    break;
                }
            }
        } Ok(args)
    }
            
    fn parse_primary(&mut self)-> Result<Expr,ParseError>{
        let start = self.peek_span();
        match self.peek().clone() {
            TokenKind::Int(n)=> {
                self.advance();
                Ok(Expr { kind: ExprKind::Int(n), span: start })
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Float(n), span: start })
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Str(s), span: start })
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr { kind: ExprKind::Bool(b), span: start })
            }
            TokenKind::Color(c)=> {
                self.advance();
                Ok(Expr { kind: ExprKind::Color(c), span: start })
            }
            TokenKind::Size(n,u )=> {
                self.advance();
                Ok(Expr { kind: ExprKind::Size(n, u), span: start })
            }
            TokenKind::Await => {
                self.advance();
                if matches!(self.peek(),TokenKind::Fetch){
                    self.advance();
                    self.expect(TokenKind::Lt)?;
                    let type_args = self.parse_type_expr()?;
                    self.expect(TokenKind::Gt)?;
                    self.expect(TokenKind::LParen)?;
                    let url = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    let span = start.to(self.last_span);
                    Ok(Expr{kind:ExprKind::AwaitFetch { type_args, url: Box::new(url) }, span})
                } else {
                    let inner = self.parse_unary()?;
                    let span = start.to(inner.span);
                    Ok(Expr { kind: ExprKind::Await { expr: Box::new(inner) }, span })
                }
            }
            TokenKind::Body=> {
                self.advance();
                Ok(Expr { kind: ExprKind::Ident("body".to_string()), span: start })
            }
            TokenKind::Ident(s)=> {
                self.advance();
                if matches!(self.peek(), TokenKind::LBrace){
                    self.parse_struct_lit(s,start)
                } else {
                    Ok(Expr { kind: ExprKind::Ident(s), span: start })
                }
            }
            TokenKind::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(&TokenKind::RBracket){
                    loop {
                        items.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma){
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBracket)?;
                let span = start.to(self.last_span);
                Ok(Expr {kind : ExprKind::List(items), span})
            }
            other => Err(self.err(format!("expected an expression, found {other}"))),
        }
    }

    fn parse_struct_lit(&mut self, type_name : String, start:Span)-> Result<Expr,ParseError>{
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        if !self.check(&TokenKind::RBrace){
            loop {
                let fname = self.expect_ident()?;
                self.expect(TokenKind::Colon)?;
                let value = self.parse_expr()?;
                fields.push((fname,value));
                if !self.eat(&TokenKind::Comma){
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        let span = start.to(self.last_span);
        Ok(Expr{kind:ExprKind::StructLit { type_name, fields }, span
        })

    }
    fn parse_type_expr(&mut self)-> Result<TypeExpr,ParseError>{
        match self.peek().clone(){
            TokenKind::IntType => {self.advance(); Ok(TypeExpr::Int)}
            TokenKind::FloatType => {self.advance(); Ok(TypeExpr::Float)}
            TokenKind::StringType => {self.advance(); Ok(TypeExpr::String)}
            TokenKind::BoolType => {self.advance(); Ok(TypeExpr::Bool)}
            TokenKind::ColorType => {self.advance(); Ok(TypeExpr::Color)}
            TokenKind::SizeType => {self.advance(); Ok(TypeExpr::Size)}
            TokenKind::VoidType => {self.advance(); Ok(TypeExpr::Void)}
            TokenKind::ListType => {
                self.advance();
                self.expect(TokenKind::Lt)?;
                let inner = self.parse_type_expr()?;
                self.expect(TokenKind::Gt)?;
                Ok(TypeExpr::List(Box::new(inner)))
            }
            TokenKind::MapType => {
                self.advance();
                self.expect(TokenKind::Lt)?;
                let k = self.parse_type_expr()?;
                self.expect(TokenKind::Comma)?;
                let v = self.parse_type_expr()?;
                self.expect(TokenKind::Gt)?;
                Ok(TypeExpr::Map(Box::new(k), Box::new(v)))
            }
            TokenKind::OptionType =>{
                self.advance();
                self.expect(TokenKind::Lt)?;
                let inner = self.parse_type_expr()?;
                self.expect(TokenKind::Gt)?;
                Ok(TypeExpr::Option(Box::new(inner)))
            }

            TokenKind::Ident(name) =>{
                self.advance();
                Ok(TypeExpr::Named(name))
            }
            other => Err(self.err(format!("expected a type, found {other}")))
        }

    }

}

//let work on the testing
#[cfg(test)]
mod tests {
    use super::*;
    use p_lexer::{tokenize,FileId};

    fn parse_src(src: &str) -> Module {
        for (line_no, line) in src.lines().enumerate(){
            println!("LINE {}: {:?}", line_no +1, line);
        }
        let tokens = tokenize(src, FileId(0)).expect("lex is Ok");
        parse(&tokens).expect("parse ok")
    }

    #[test]
    fn store_decl_parses(){
        let src = "struct Task\n id: Int\n\nstore tasks:List<Task>\n";
        let module=parse_src(src);
        assert!(matches!(module.items[1],TopLevelItem::Store(_)));
    }

    #[test]
    fn extern_client_global_parse(){
        let src = "extern fn alert(msg: String)-> Void client global \"alert\"\n";
        let module = parse_src(src);
        let TopLevelItem::Extern(e)=&module.items[0] else {panic!()};
        assert!(matches!(&e.target,ExternTarget::ClientGlobal { name } if name == "alert"));
    }

    #[test]
    fn extern_client_module_with_as_parses(){
        let src = "extern fn parseISO(s: String) -> Int client module \"https://esm.sh/date-fns\" as \"parseISO\"\n";
        let module = parse_src(src);
        let TopLevelItem::Extern(e) = &module.items[0] else  {panic!()};
        assert!(matches!(&e.target,ExternTarget::ClientModule { url, export: Some(ex) } if url == "https://esm.sh/date-fns" && ex == "parseISO"));
    }

    #[test]
    fn extern_server_npm_without_as_parses(){
        let src = "extern fn hashSync(s:String)-> String server npm \"bcrypt\"\n";
        let module = parse_src(src);
        let TopLevelItem::Extern(e) = &module.items[0] else {panic!()};
        assert!(matches!(&e.target,ExternTarget::ServerNpm { package, export } if package == "bcrypt"));
    }

    #[test]
    fn test_and_assert_parse(){
        let src = "fn double(x:Int) -> Int\n   return x * 2\n\ntest \"doubles correctly\"\n   let r = double(3)\n   assert r == 6\n";
        let module = parse_src(src);
        let TopLevelItem::Test(t) = &module.items[1] else { panic!()};
        assert_eq!(t.description,"doubles correctly");
        assert!(matches!(t.body[1], Stmt::Assert { .. }));
    }

    #[test]
    fn dynamic_route_path_is_just_a_plain_string_at_parse_time(){
        // id extraction happens at P- AST lowering not here
        // it just treat it as a whole string path
        let src = "struct T\n  id:Int\n\nroute GET \"/api/tasks/:id\" -> T\n  return T {id: 1 } \n";
        let module = parse_src(src);
        let TopLevelItem::Route(r) = &module.items[1] else { panic!()};
        assert_eq!(r.path,"/api/tasks/id");

        
    }

    #[test]
    fn lambda_assignment_still_parses(){
        let src = "page Home\n  state username: String = \"\"\n  input username\n    on change(val) => username = val\n";
        let module = parse_src(src);
        
        let TopLevelItem::Page(p) = &module.items[0] else {panic!()};
        
        let UiNode::Kind { body, .. } = &p.root[0] else {panic!()};
        
        let NodeBodyItem::Event(ev) = &body[0] else {panic!()};
        

        assert!(matches!(&ev.handler,EventHandler::Lambda {  body : LambdaBody::Assign { .. }, .. }));
    }

    #[test]
    fn chained_comparison_is_still_a_parse_error(){
        let src = "fn f() -> Bool\n  return 1 < 2 < 3\n";
        let tokens = tokenize(src, FileId(0)).unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn parse_stage1_worked(){
        let src = 
r#"
page Home
    state count : Int  = 0

    column
        padding 24px
        spacing 16px
    text "Hello"
        fontSize 24
        fontWeight bold

    text count
        color "333"
    row
        spacing 16
        button "Refresh"
            on click increment()
        button "Reset"
            on click reset()
fn increment()-> Void 
    count = count + 1

fn reset() -> Void 
    count = 0
      


"#;
        let module = parse_src(src);
        assert_eq!(module.items.len(), 3);
        assert!(matches!(module.items[0], TopLevelItem::Page(_)));
        assert!(matches!(module.items[1], TopLevelItem::Fn(_)));
    }

  

}