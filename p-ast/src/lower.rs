// ! CST -> AST lowering: NodeId/ExprId assignment , implicit page-wrap,
// box model property arity normalization , route path-param extraction
// + body folding , extern export -name defaulting , Current through

// Note on TypeExpr here: thid id a plain structural mirror
// 


use crate::ast::*;
use p_lexer::Span;
use p_parser::cst;


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
    let mut l = Lowerer{next_node: 0, next_expr: 0};
    l.lower_module(module)

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
                cst::TopLevelItem::Import(path) => out.imports.push(path),
                cst::TopLevelItem::Enum(e) => {
                    out.enums.push(EnumDecl { name: e.name, name_span: e.name_span, variants: e.variants })
                }
                cst::TopLevelItem::Struct(s) => out.structs.push(self.lower_struct_decl(s)),
                cst::TopLevelItem::State(s) => out.state_decls.push(self.lower_state_decl(s)?),
                cst::TopLevelItem::Fn(f) => out.fns.push(self.lower_fn_decl(f)?),
                cst::TopLevelItem::Component(c) => out.components.push(self.lower_component_decl(c)?),
                cst::TopLevelItem::Layout(l) => out.layouts.push(self.lower_layout_decl(l)?),
                cst::TopLevelItem::Page(p) => out.pages.push(self.lower_page_decl(p)?),
                cst::TopLevelItem::Route(r) => out.routes.push(self.lower_route_decl(r)?),
                cst::TopLevelItem::Store(s) => out.stores.push(self.lower_store_decl(s)),
                cst::TopLevelItem::Extern(e) => out.externs.push(self.lower_extern_decl(e)),
                cst::TopLevelItem::Test(t) => out.tests.push(self.lower_test_decl(t)?)
            }
            
        }
        Ok(out)
        
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

    fn lower_layout_decl(&mut self, l: cst::LayoutDecl)-> Result<LayoutDecl,LowerError>{
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
    // We are giving the tree type node for httpmethod
    // HttpMethods are just for future reference to support little applications that wants to extend P
    // to be a backend language.
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
    //Let start with this we are walking down the tree for the property statement defined in a node
    // PropertyStmt {}
    //

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
                        LambdaBody::Assign { target: self.lower_lvalue(target)?, value: self.lower_expr(value)? }
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

    fn lower_for_node(&mut self, f: cst::ForNode) -> Result<Node,LowerError>{
        let id = self.node_id();
        let iter = self.lower_expr(f.iter)?;
        let body = self.lower_node_list(f.body)?;
        Ok(Node { id, span: f.span, kind: NodeKind::For { var: f.var, var_span: f.var_span, iter, body } })
    
    }


    fn lower_node_list(&mut self, nodes: Vec<cst::UiNode>)-> Result<Vec<Node>,LowerError>{
        nodes.into_iter().map(|n| self.lower_ui_node(n)).collect()
    }

    fn lower_args(&mut self, args: Vec<cst::Arg>) -> Result<Vec<Arg>, LowerError>{
        args.into_iter().map(|f| Ok(Arg{name: f.name, value: self.lower_expr(f.value)?})).collect()

    }

    fn lower_lvalue(&mut self, l: cst::LValue) -> Result<LValue,LowerError>{
        let mut accessors = Vec::with_capacity(l.accessors.len());
        for a in l.accessors {
            accessors.push(match a {
                cst::Accessor::Field(f)=> Accessor::Field(f),
                cst::Accessor::Index(e) => Accessor::Index(self.lower_expr(e)?),
            });
        }
        Ok(LValue { name: l.name, accessors })
    }

    fn lower_expr(&mut self, e: cst::Expr) -> Result<ExprNode,LowerError>{
        let id = self.expr_id();
        let span = e.span;
        let kind = match e.kind {
            cst::ExprKind::Int(n) => ExprKind::Int(n),
            cst::ExprKind::Float(n) => ExprKind::Float(n),
            cst::ExprKind::Str(s) => ExprKind::Str(s),
            cst::ExprKind::Bool(b) => ExprKind::Bool(b),
            cst::ExprKind::Color(c) => ExprKind::Color(c),
            cst::ExprKind::Size(n,u) => ExprKind::Size(n, u),
            cst::ExprKind::Ident(s) => ExprKind::Ident(s),
            cst::ExprKind::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(self.lower_expr(it)?);
                }
                ExprKind::List(out)
            }
            cst::ExprKind::StructLit { type_name, fields } =>{
                let mut out = Vec::with_capacity(fields.len());
                for (n,v) in fields {
                    out.push((n, self.lower_expr(v)?));
                }
                ExprKind::StructLit { type_name, fields: out }
            }
            cst::ExprKind::Unary { op, expr } => {
                ExprKind::Unary { op: lower_unary_op(op), expr: Box::new(self.lower_expr(*expr)?) }
            }
            cst::ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary { op: lower_binary_op(op), lhs: Box::new(self.lower_expr(*lhs)?), rhs: Box::new(self.lower_expr(*rhs)?) },
            cst::ExprKind::Call { callee, args } => {
                ExprKind::Call {callee:Box::new(self.lower_expr(*callee)?), args: self.lower_args(args)?}
            }
            cst::ExprKind::Field { base, name } => {
                ExprKind::Field { base: Box::new(self.lower_expr(*base)?), name }
            }
            cst::ExprKind::Index {base, index} => ExprKind::Index { base: Box::new(self.lower_expr(*base)?), index: Box::new(self.lower_expr(*index)?) },
            cst::ExprKind::AwaitFetch { type_args, url } => {
                ExprKind::AwaitFetch { type_arg: lower_type(type_args), url: Box::new(self.lower_expr(*url)?) }
            }
            cst::ExprKind::Await { expr } => ExprKind::Await { expr: Box::new(self.lower_expr(*expr)?) },
            
    };
    Ok(ExprNode { id, span, kind } )


    }
    fn lower_stmt(&mut self, s: cst::Stmt) -> Result<Stmt,LowerError>{
        Ok(match s {
            cst::Stmt::Let {name, name_span, ty,value} => {
                Stmt::Let { name, name_span, ty: ty.map(lower_type), value: self.lower_expr(value)? }
            }
            cst::Stmt::Assign {target, value} => {
                Stmt::Assign { target: self.lower_lvalue(target)?, value: self.lower_expr(value)? }
            }
            cst::Stmt::If { cond, then_branch, else_branch }=> Stmt::If { cond: self.lower_expr(cond)?, then_branch: self.lower_stmt_list(then_branch)?, else_branch: else_branch.map(|b| self.lower_stmt_list(b)).transpose()?,},
            cst::Stmt::For {var, var_span,iter, body} => Stmt::For { var, var_span, iter: self.lower_expr(iter)?, body: self.lower_stmt_list(body)?, },
            cst::Stmt::While { cond, body } => Stmt::While { cond: self.lower_expr(cond)?, body: self.lower_stmt_list(body)? },
            cst::Stmt::Return(e) => Stmt::Return(e.map(|e| self.lower_expr(e)).transpose()?),
            cst::Stmt::Assert { expr, span } => Stmt::Assert { expr: self.lower_expr(expr)?, span },
            cst::Stmt::Expr(e) => Stmt::Expr(self.lower_expr(e)?)
                
            
        })

    }

    fn lower_stmt_list(&mut self, stmts: Vec<cst::Stmt>) -> Result<Vec<Stmt>,LowerError>{
        stmts.into_iter().map(|s|self.lower_stmt(s)).collect()

    }
}

    fn lower_element_kind(k: cst::NodeKind)-> ElementKind {
        match k {
            cst::NodeKind::Button => ElementKind::Button,
            cst::NodeKind::Card => ElementKind::Card,
            cst::NodeKind::Checkbox => ElementKind::Checkbox,
            cst::NodeKind::Column => ElementKind::Column,
            cst::NodeKind::Container => ElementKind::Container,
            cst::NodeKind::Dialog => ElementKind::Dialog,
            cst::NodeKind::Dropdown => ElementKind::Dropdown,
            cst::NodeKind::Grid => ElementKind::Grid,
            cst::NodeKind::Icon => ElementKind::Icon,
            cst::NodeKind::Image => ElementKind::Image,
            cst::NodeKind::Input => ElementKind::Input,
            cst::NodeKind::List => ElementKind::List,
            cst::NodeKind::Menu => ElementKind::Menu,
            cst::NodeKind::Modal => ElementKind::Modal,
            cst::NodeKind::Navigation => ElementKind::Navigation,
            cst::NodeKind::Radio => ElementKind::Radio,
            cst::NodeKind::Row => ElementKind::Row,
            cst::NodeKind::Slot => ElementKind::Slot,
            cst::NodeKind::Stack => ElementKind::Stack,
            cst::NodeKind::Switch => ElementKind::Switch,
            cst::NodeKind::Table => ElementKind::Table,
            cst::NodeKind::Tabs => ElementKind::Tabs,
            cst::NodeKind::Text => ElementKind::Text,
            cst::NodeKind::Textarea => ElementKind::Textarea
            
        }
    }

    fn lower_unary_op(op: cst::UnaryOp)->  UnaryOp {
        match op {
            cst::UnaryOp::Neg=> UnaryOp::Neg,
            cst::UnaryOp::Not => UnaryOp::Not,
        }
    }

    fn lower_binary_op(op: cst::BinaryOp) -> BinaryOp {
        match op {
            cst::BinaryOp::Add => BinaryOp::Add,
            cst::BinaryOp::And => BinaryOp::And,
            cst::BinaryOp::Div => BinaryOp::Div,
            cst::BinaryOp::Eq => BinaryOp::Eq,
            cst::BinaryOp::Gt => BinaryOp::Gt,
            cst::BinaryOp::GtEq => BinaryOp::GtEq,
            cst::BinaryOp::Lt => BinaryOp::Lt,
            cst::BinaryOp::LtEq => BinaryOp::LtEq,
            cst::BinaryOp::Mod => BinaryOp::Mod,
            cst::BinaryOp::Mul => BinaryOp::Mul,
            cst::BinaryOp::NotEq => BinaryOp::NotEq,
            cst::BinaryOp::Or => BinaryOp::Or,
            cst::BinaryOp::Sub => BinaryOp::Sub,
            
        }
    }

    fn lower_type(t: cst::TypeExpr)-> TypeExpr{
        match t {
            cst::TypeExpr::Bool => TypeExpr::Bool,
            cst::TypeExpr::Color => TypeExpr::Color,
            cst::TypeExpr::Float => TypeExpr::Float,
            cst::TypeExpr::Int => TypeExpr::Int,
            cst::TypeExpr::List(inner) => TypeExpr::List(Box::new(lower_type(*inner))),
            cst::TypeExpr::Map(k,v ) => TypeExpr::Map(Box::new(lower_type(*k)), Box::new(lower_type(*v))),
            cst::TypeExpr::Option(inner)=> TypeExpr::Option(Box::new(lower_type(*inner))),
            cst::TypeExpr::Named(n) =>TypeExpr::Named(n),
            cst::TypeExpr::Size => TypeExpr::Size,
            cst::TypeExpr::String => TypeExpr::String,
            cst::TypeExpr::Void => TypeExpr::String,
        }
    }


#[cfg(test)]
mod tests {
    use super::*;
    use p_lexer::{FileId,tokenize};
    use p_parser::parse;

    fn lower_src(src: &str) -> Module{
        let tokens = tokenize(src,FileId(0)).expect("lex ok");
        let cst = parse(&tokens).expect("parse ok");
        lower(cst).expect("lower ok")
    }

    #[test]
    fn implicit_wrap_applies_when_page_has_multiple_roots(){
        let module = lower_src("page Home\n  text \"a\"\n  test \"b\"\n");
        let NodeKind::Element { kind:ElementKind::Column, children, .. } = &module.pages[0].root.kind else {
            panic!()
        };
        assert_eq!(children.len(), 2); 
    }

    #[test]
    fn dynamic_path_params_precede_body_and_has_body_is_explicit(){
        let module = lower_src(
            "struct NewLabel\n  label: String\n\nstruct Task\n  id:Int\n  label: String\n\nroute PUT \"/api/tasks/:id\" body: NewLabel -> Task\n  return Task { id: 1, label:body.label }\n",
        );
        dbg!(&module.routes);
        let r = &module.routes[0];
        dbg!(&r.params[0]);
        assert_eq!(r.params.len(), 2);
        assert_eq!(r.params[0].name,"id");
        dbg!(&r.params[1]);
        assert_eq!(r.params[1].name, "body");
        dbg!(&r.has_body);
        assert!(r.has_body);
    }

    #[test]
    fn path_only_route_has_body_false_not_inferred_from_params_len(){

        let module = lower_src(
            "struct T\n  id: Int\n\nroute GET \"/api/tasks/:id\" -> T\n  return T { id: 1 }\n",
        );
        let r = &module.routes[0];
        assert_eq!(r.params.len(),1);
        assert!(!r.has_body);
    }

    #[test]

    fn colliding_path_param_and_body_name_is_rejected(){
        let tokens = tokenize(
           "struct T\n  x:Int\n\nroute POST \"/api/:body\" body: T -> T\n  return body\n" , 
            FileId(0)).unwrap();
        let cst = parse(&tokens).unwrap();
        let err = lower(cst).unwrap_err();
        assert!(matches!(err,LowerError::DuplicateRouteParam { .. }));
    }

    #[test]
    fn extern_export_defaults_to_declared_name_when_as_omitted(){
        let module = lower_src("extern fn hashSync(s: String) -> String server npm \"bcrypt\"\n");
        let ExternTarget::ServerNpm {  export, .. } = &module.externs[0].target else {
            panic!()
        };
        assert_eq!(export,"hashSync");
    }

    #[test]
    fn test_decl_lowers_with_structural_type_only(){
        let module = lower_src("struct Task\n  id: Int\n\nstore tasks: List<Task>\n");
        assert_eq!(module.stores[0].name,"tasks");
        assert!(matches!(module.stores[0].ty,TypeExpr::List(_)));
    }

    #[test]
    fn test_decl_and_assert_stmt_survive_lowering(){
        let module = lower_src(
            "fn double(x: Int) -> Int\n  return x * 2\n\ntest \"doubles\"\n  let r = double(3)\n  assert r == 6\n",
        );
        assert_eq!(module.tests[0].description,"doubles");
        assert!(matches!(module.tests[0].body[1], Stmt::Assert { .. }));
    }

    #[test]
    fn two_value_property_is_still_rejected_with_a_real_span(){
        let tokens = tokenize(
            "page Home\n  column\n    padding 8px 16px\n", 
            FileId(0)).unwrap();
        let cst = parse(&tokens).unwrap();
        let err = lower(cst).unwrap_err();
        assert!(matches!(err,LowerError::InvalidPropertyArity { found : 2, .. }));
    }

    

}

     





