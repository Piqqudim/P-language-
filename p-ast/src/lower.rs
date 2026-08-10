// ! CST -> AST lowering: NodeId/ExprId assignment , implicit page-wrap,
// box model property arity normalization , route path-param extraction
// + body folding , extern export -name defaulting , Current through

// Note on TypeExpr here: thid id a plain structural mirror
// 


use crate::ast::*;
use p_lexer::Span;
use p_parser::cst;

use std::char::Lower;
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
        let root = self.wrap_or_string(p.root)?;
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
        if let Some(ty) = r.body_ty
    }

}



