// ! CST -> AST lowering: NodeId/ExprId assignment , implicit page-wrap,
// box model property arity normalization , route path-param extraction
// + body folding , extern export -name defaulting , Current through

// Note on TypeExpr here: thid id a plain structural mirror
// 


use crate::ast::*;
use p_lexer::Span;
use p_parser::{NodeBodyItem, cst};


use std::collections::HashSet;
use std::fmt;

#[derive(Debug,Clone,PartialEq)]
pub enum LowerError {

    //grammar only defines the 1-value and 4-value box model shorthand
    InvalidPropertyArity {property: String, found:usize, span:Span},

    //dynamic route paths . A path segment literally named 'body' (or two identically-named path segment)
    DuplicateRouteParam{ name:String, path: String, span:Span},
}

impl fmt::Display for LowerError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self {
            LowerError::InvalidPropertyArity { property, found, .. } => {
                write!(f, "property '{property}' must have exactly 1 or 4 values, found {found}")
            }
            LowerError::DuplicateRouteParam { name, path, .. } => write!(f, "route '{path}' declares '{name}' more than once (path parameter and/or body)"),
        }
    }
}

impl std::error::Error for LowerError {}

pub fn lower(module: cst::Module) -> Result<Module,LowerError>{

}
struct Lowerer {
    next_node: u32,
    next_expr: u32,
}

impl Lowerer {
    fn node_id(&mut self) -> NodeId{
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }

    fn expr_id(&mut self) -> ExprId {
        let id = ExprId(self.next_expr);
        self.next_expr += 1;
        id
    }

    fn lower_module(&mut self, m: cst::Module) -> Result<Module,LowerError>{
        let mut out = Module{
            imports: Vec::new(),
            enums: Vec::new(),
            structs: Vec::new(),
            state_decls: Vec::new(),
            fns: Vec::new(),
            components: Vec::new(),
            layouts : Vec::new(),
            pages: Vec::new(),
            routes: Vec::new(),
            stores: Vec::new(),
            externs: Vec::new(),
            tests: Vec::new(),
        };
        for  item  in m.items {
            match item {
                cst::TopLevelItem::Import(path)= > out.imports.push(path),
                cst::TopLevelItem::Enum(e) => {
                    out.enums.push(EnumDecl { name: e.name, name_span: e.name_span, variants: e.variants })
                }
                cst::TopLevelItem::Struct(s) => out.structs.push(value),
            }
            
        }
        
    }

    fn lower_struct_decl(&mut self, s: cst::StructDecl) -> StructDecl{
        StructDecl { name: s.name, name_span: s.name_span, fields: s.fields.into_iter().map(|(n,t)|(n, lower_type(t))).collect(), }
    }

    fn lower_state_decl(&mut self, s: cst::StateDecl) -> Result<StateDecl,LowerError>{
        Ok(StateDecl { name: s.name, name_span: s.name_span, ty: lower_type(s.ty), value: self.lower_expr(s.value)?,})
    }

    fn lower_params(&self, params:Vec<cst::Param>) -> Vec<Param>{
        params.into_iter().map(|p| Param{name: p.name, name_span: p.name_span, ty: lower_type(p.ty)}).collect()
    }
    fn lower_fn_decl(&mut self, f: cst::FnDecl) -> Result<FnDecl,LowerError>{
        let params = self.lower_params(f.params);
        let ret = f.ret.map(lower_type);
        let mut body = Vec::with_capacity(f.body.len());
        for s in f.body{
            body.push(self.lower_stmt(s)?);
        }
        Ok(FnDecl { name: f.name, name_span: f.name_span, params, ret, body })
    }

    fn lower_component_decl(&mut self, c: cst::ComponentDecl) -> Result<ComponentDecl,LowerError>{
        let params = self.lower_params(c.params);
        let mut state_decls = Vec::with_capacity(c.state_decls.len());
        for s in c.state_decls{
            state_decls.push(self.lower_state_decl(s)?);

            }
            let mut fns = Vec::with_capacity(c.fns.len());
            for f in c.fns{
                fns.push(self.lower_fn_decl(f)?);
            }
            let root = self.wrap_or_single(c.root)?;
            Ok(ComponentDecl { name: c.name, name_span: c.name_span, params, state_decls, fns, root })
    }

    fn lower_lower_layout_decl(&mut self, l: cst::LayoutDecl)-> Result<LayoutDecl,LowerError>{
        let root = self.lower_ui_node(l.root)?;
        Ok(LayoutDecl { name: l.name, name_span: l.name_span, root })
    }

    fn lower_page_decl(&mut self,p: cst::PageDecl)-> Result<PageDecl,LowerError>{
        let mut state_decls = Vec::with_capacity(p.state_decls.len());
        for s in p.state_decls{
            state_decls.push(self.lower_state_decl(s)?);
        }
        let mut fns = Vec::with_capacity(p.fns.len());
        for f in p.fns{
            fns.push(self.lower_fn_decl(f)?);
        }
        let root = self.wrap_or_single(p.root)?;
        Ok(PageDecl { name: p.name, name_span: p.name_span, uses: p.uses, state_decls, fns, root })
    }

    fn lower_method(m: cst::HttpMethod)-> HttpMethod{
        match m {
            cst::HttpMethod::Get=> HttpMethod::Get,
            cst::HttpMethod::Post => HttpMethod::Post,
            cst::HttpMethod::Put => HttpMethod::Put,
            cst::HttpMethod::Delete => HttpMethod::Delete,
            cst::HttpMethod::Patch =>HttpMethod::Patch,
        }
    }


    fn extract_path_param_names(path: &str)->Vec<String>{
        path.split('/').filter_map(|seg| seg.strip_prefix(':').map(|n| n.to_string())).collect()
    }

    fn lower_route_decl(&mut self,r: cst::RouteDecl) -> Result<RouteDecl,LowerError>{
        let mut params: Vec<Param> = Self::extract_path_param_names(&r.path).into_iter().map(|name| Param{name, name_span:r.method_span,ty: TypeExpr::String}).collect();
        let has_body = r.body_ty.is_some();
        if let Some(ty) = r.body_ty{
            params.push(Param { name: "body".to_string(), name_span: r.method_span, ty: lower_type(ty) });
        }

        let mut seen = HashSet::new();
        for p in &params{
            if !seen.insert(p.name.clone()){
                return Err(LowerError::DuplicateRouteParam { name: p.name.clone(), path: r.path.clone(), span: r.method_span, });
            }
        }
        let mut body = Vec::with_capacity(r.body.len());
        for s in r.body{
            body.push(self.lower_stmt(s)?);
        }

        Ok(RouteDecl { method: Self::lower_method(r.method), method_span: r.method_span, path: r.path, params, has_body, ret: lower_type(r.ret), body })
    }

    fn lower_store_decl(&self, s: cst::StoreDecl) -> StoreDecl{
        StoreDecl { name: s.name, name_span: s.name_span, ty: lower_type(s.ty) }
    }


    // defaulting happens exactly once, here - every later stage can assume ExternTarget's export name is always populated
    fn lower_extern_decl(&self, e: cst::ExternDecl) -> ExternDecl{
        let target = match e.target{
            cst::ExternTarget::ClientGlobal { name } => ExternTarget::ClientGlobal { name },
            cst::ExternTarget::ClientModule { url, export } => {
                ExternTarget::ClientModule { url, export: export.unwrap_or_else(|| e.name.clone()) }
            }
            cst::ExternTarget::ServerNpm { package, export } => { ExternTarget::ServerNpm { package, export: export.unwrap_or_else(|| e.name.clone()) }}
            
        };
        ExternDecl { name: e.name, name_span: e.name_span, params: self.lower_params(e.params), ret: e.ret.map(lower_type), target, }
    } 


    fn lower_test_decl(&mut self, t : cst::TestDecl) -> Result<TestDecl,LowerError>{
        let mut body = Vec::with_capacity(t.body.len());
        for s in t.body{
            body.push(self.lower_stmt(s)?);
        }
        Ok(TestDecl { description: t.description, description_span: t.description_span, body })
    }

    fn wrap_or_single(&mut self, mut nodes: Vec<cst::UiNode>) -> Result<Node,LowerError>{
        if nodes.len() == 1 {
            return self.lower_ui_node(nodes.remove(0));
        }
        let span = nodes[0].span().to(nodes[nodes.len() - 1].span());
        let mut children = Vec::with_capacity(nodes.len());
        for n in nodes {
            children.push(self.lower_ui_node(n)?);
        }
        Ok(Node { id: self.node_id(), span, kind: NodeKind::Element { kind: ElementKind::Column, inline_arg: None, properties: Vec::new(), events: Vec::new(), children }})
    }

    fn lower_ui_node(&mut self, node: cst::UiNode) -> Result<Node,LowerError>{
        let id = self.node_id();
        let span = node.span();
        let kind = match node {
            cst::UiNode::Kind { kind, inline_arg, body, .. } => {
                let inline_arg = inline_arg.map(|e| self.lower_expr(e)).transpose()?;
                let (properties, events,children) = self.split_body(body)?;
                NodeKind::Element { kind: lower_element_kind(kind), inline_arg, properties, events , children }
               
        }

        cst::UiNode::Call { name, args, body, .. } => {
            let args = self.lower_args(args)?;
            let (properties, events, children) = self.split_body(body)?;
            debug_assert!(
                properties.is_empty() && events.is_empty(),
                " component call body should only ever contain child nodes"
            );
            NodeKind::ComponentCall { name, args, children }
        }
    
    };
    Ok(Node { id, span, kind })

    }

    fn split_body(&mut self, body: Vec<cst::NodeBodyItem>) -> Result<(Vec<Property>,Vec<Event>,Vec<Node>),LowerError>{
        let mut properties = Vec::new();
        let mut events = Vec::new();
        let mut children = Vec::new();
        for item in body{
            match item {
                cst::NodeBodyItem::Property(p)=> properties.push(self.lower_property(p)?),
                cst::NodeBodyItem::Event(e) => events.push(self.lower_event(e)?),
                cst::NodeBodyItem::Node(n) => children.push(self.lower_ui_node(n)?),
                cst::NodeBodyItem::If(i) => children.push(self.lower_if_node(i)?),
                cst::NodeBodyItem::For(f) => children.push(self.lower_for_node(f)?),
            }
        }
        Ok((properties, events, children))
    }

    fn lower_property(&mut self, p: cst::PropertyStmt) -> Result<Property,LowerError>{
        let value = match p.values.len(){
            1 => {
                let mut vals = p.values;
                PropertyValue::Single(self.lower_expr(vals.remove(0))?)
            }
            4 => {
                let mut vals = p.values.into_iter();
                let top = self.lower_expr(vals.next().unwrap())?;
                let right = self.lower_expr(vals.next().unwrap())?;
                let bottom = self.lower_expr(vals.next().unwrap())?;
                let left = self.lower_expr(vals.next().unwrap())?;
                PropertyValue::Box { top, right, bottom, left }
            }
            n => return Err(LowerError::InvalidPropertyArity { property: p.name, found: n, span: p.span })
        };
        Ok(Property { name: p.name, value })
    }

    fn lower_event(&mut self, e: cst::EventStmt) -> Result<Event, LowerError>{
        let handler = match e.handler {
            cst::EventHandler::Call(expr) => EventHandler::Call(self.lower_expr(expr)?),
            cst::EventHandler::Lambda { params, body } => {
                let body = match body {
                    cst::LambdaBody::Expr(e) => LambdaBody::Expr(self.lower_expr(e)?),
                    cst::LambdaBody::Assign { target, value }=> {
                        LambdaBody::Assign { target: self.lower_lvalue(target)?, value: self.lower_expr(value) }
                    }
                };
                EventHandler::Lambda { params, body}
            }
        };
        Ok(Event { name: e.name, handler })
    }

    fn lower_if_node(&mut self,i: cst::IfNode) -> Result<Node,LowerError>{
        let id = self.node_id();
        let cond = self.lower_expr(i.cond)?;
        let then_branch = self.lower_node_list(i.then_branch)?;
        let else_branch = i.else_branch.map(|b| self.lower_node_list(b)).transpose()?;
        Ok(Node { id, span: i.span, kind: NodeKind::If { cond, then_branch, else_branch } })
    }

    fn lower_expr(&mut self, e: cst::Expr) -> Result<ExprNode,LowerError>{

    }
    fn lower_stmt(&mut self, s: cst::Stmt) -> Result<

     

}



