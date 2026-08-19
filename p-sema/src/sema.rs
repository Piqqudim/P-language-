// Semantic analysis: name resolution + structural validation, NOT type
// checking (p-typeck's job) - this only confirms things exist and are used
// in a structurally valid way.

// Every stages has their checkers independently of the typechecking
// that makes the engine very strict to make sure the Intermediate Representation(IR)
// is relieved of the burden to typecheck everything down from the parser to the Ir
// to later the engine that emits html, css and javascript.


// Visibility model: FnContext replaces what used to be a single "allow_stores : bool" flag
// the issue encountered is the i don't really rely on the basic construction of it fully supporting
// backend. It is actually a great idea
// Dropped the unused variant rather than ship dead code; Kept 'ExternUsedDirectly' (parallel to 'StoreUsedDirectly')
// which IS is actually reachable - a bare extern reference used as a value, not a call

use p_ast::*;
use p_elementkind::attr_name;
use p_lexer::{Span, Spanned};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug,Clone,PartialEq)]
pub enum Resolution{
    State,
    Local,
    Param,
    Fn,
    Component,
    EnumVariant { enum_name : String},
    Store,
    Extern,
}

#[derive(Debug,Clone,PartialEq)]
pub enum SemaError {
    DuplicateTopLevelName { name: String, first_kind: &'static str, second_kind: &'static str},
    UnKnownLayout {page: String, layout: String},
    UnKnownIdentifier {name: String},
    ComponentCallableAsPlainFunction {name : String},
    UnKnownCallable { name: String},
    ComponentCallHasChildren {component: String},
    DuplicateNamedArgument {component: String, arg: String},
    UnKnownComponentParam { component: String, param: String},
    TooManyPositionalArgs {component: String, expected: usize, found: usize},
    SlotUsedOutSideLayout,
    UnKnownEvent { element: String, event : String},
    AssignToParam { name: String},
    AssignToUnKnownName { name: String},
    UnKnownStruct { name: String},
    UnKnownStructField { struct_name: String, field: String},
    DuplicateStructField { struct_name: String, field: String},
    MissingStructField {struct_name: String, field: String},
    DuplicateRoute {method: String, path: String},
    StoreUsedDirectly {name: String},
    UnKnownStoreMethod {store: String, method: String},
    ExternUsedDirectly { name: String},

}

impl fmt::Display for SemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self {
            SemaError::DuplicateTopLevelName { name, first_kind, second_kind } => {write!(f, "'{name}' is declared twice (as {first_kind} and as {second_kind}")}
            SemaError::UnKnownLayout { page, layout } => {
                write!(f, "page '{page}' uses unknown layout '{layout}'")
            }
            SemaError::UnKnownIdentifier { name } => write!(f, "unknown identifier '{name}'"),
            SemaError::ComponentCallableAsPlainFunction { name } => {
                write!(f, " '{name}' is a component and can only be used in UI-node position, not called as a function")
            }
            SemaError::UnKnownCallable { name } => write!(f, "unknown function or component '{name}' "),
            SemaError::ComponentCallHasChildren { component } => {
                write!(f,"component '{component}' does not accept child node (parameters only)")
            }
            SemaError::DuplicateNamedArgument { component, arg } => {
                write!(f, "argument '{arg}' given more than once in call to '{component}'")
            }
            SemaError::UnKnownComponentParam { component, param } => {
                write!(f, "component '{component}' has no parameter named '{param}'")
            }
            SemaError::TooManyPositionalArgs { component, expected, found } => {
                write!(f, "component '{component} takes {expected} parameter(s), found {found} positional argument(s)'")
            }
            SemaError::SlotUsedOutSideLayout=> write!(f, " 'slot' may only be used inside a layout"),
            SemaError::UnKnownEvent { element, event } => write!(f, " '{element}' has no '{event}' event"),
            SemaError::AssignToParam { name } => write!(f, "cannot assign to {name}: parameters are immutable"),
            SemaError::AssignToUnKnownName { name } => {
                write!(f, "cannot assign to unknown name '{name}'")
            }
            SemaError::UnKnownStruct { name } => write!(f, "unknown struct {name}"),
            SemaError::UnKnownStructField { struct_name, field } => {
                write!(f, "struct '{struct_name}' has no field name '{field}'")
            }
            SemaError::DuplicateStructField { struct_name, field } => {
                write!(f, "field '{field}' given more than once in a {struct_name}' literal")
            }
            SemaError::MissingStructField { struct_name, field } => {
                write!(f, "struct '{struct_name}' literal is missing field '{field}'")
            }
            SemaError::DuplicateRoute { method, path } => {
                write!(f, "route '{method}{path}' is declared more than once")
            }
            SemaError::StoreUsedDirectly { name } => {
                write!(f, "store '{name}' cannot be used directly; call one of its methods: .all(), .find(id), .insert(x), .update(id, x), .delete(id)")
            }
            SemaError::UnKnownStoreMethod { store, method } =>{
                write!(f, "store '{store}' has no method '{method}'")
            }
            SemaError::ExternUsedDirectly { name } => {
                write!(f, "extern '{name}' can only be called, not used as a value")
            }
        }
    }
}

impl std::error::Error for SemaError{}

pub struct SemaResult {
    pub resolutions: HashMap<ExprId,Resolution>,
    pub errors: Vec<Spanned<SemaError>>,
}

type Scope = HashMap<String,Resolution>;

//Which of the four real visibility shapes a body being checked has
// Replaces what used to be a single 'allow_stores: bool' once
// client/server externs added a second, orthogonal axis, a bool
//could no longer represent every real combination(e.g Route needs
// both stores and server externs)
#[derive(Clone, Copy, PartialEq)]
enum FnContext {
    //module.fns - compiled into both client and server bundles, so it gets neither
    // stores nor either extern kind: giving it either would either leak a server
    //only API client-side , or require per-target fn compilation (a bigger change, not done)
    Shared,

    //component/page-local fns , event handlers, UI-tree expression- client bundle only
    ClientOnly,

    // route bodies - server bundle only
    Route,

    //test bodies - compiled into dist/test.js only. Deliberately No
    // store access even though tests run "server-side": no test database
    // isolation exists yet, so allowing it would mean tests
    // silently read/write the SAME ./data.sqlite a real running server use
    Test,
}
pub fn analyzer(module: &Module) -> SemaResult{
    let mut a  = Analyzer {
        fn_names : HashSet::new(),
        component_names : HashSet::new(),
        layout_names : HashSet::new(),
        page_names : HashSet::new(),
        enum_variants : HashMap::new(),
        component_params : HashMap::new(),
        struct_names : HashSet::new(),
        struct_fields : HashMap::new(),
        route_keys : HashSet::new(),
        global_state_names : HashSet::new(),
        store_names :  HashSet::new(),
        client_extern_names : HashSet::new(),
        server_extern_names : HashSet::new(),
        resolutions: HashMap::new(),
        errors : Vec::new(),

    };
    a.collect_globals(module);
    a.check_layout_used(module);

    for f in &module.fns {
        a.check_fn(&f.params, &f.body, FnContext::Shared);
    }
    for c in &module.components {
        let mut scope = a.seed_scope(FnContext::ClientOnly);
        for p in &c.params {
            scope.insert(p.name.clone(), Resolution::Param);
        }
        for s in &c.state_decls {
            a.check_expr(&s.value, &scope);
            scope.insert(s.name.clone(), Resolution::State);
        }
        for f in &c.fns {
            a.check_fn(&f.params, &f.body, FnContext::ClientOnly);
        }
        a.check_node(&c.root, &scope, false);
    }
    for p in &module.pages {
        let mut scope = a.seed_scope(FnContext::ClientOnly);
        for s in &p.state_decls {
            a.check_expr(&s.value, &scope);
            scope.insert(s.name.clone(), Resolution::State);
        }
        for f in &p.fns {
            a.check_fn(&f.params,&f.body,FnContext::ClientOnly);
        }
        a.check_node(&p.root, &scope, false);
    }
    for l in &module.layouts {
        a.check_node(&l.root, &Scope::new(), true);
    }
    for r in &module.routes {
        a.check_fn(&r.params, &r.body, FnContext::Route);
    }
    for t in &module.tests {
        a.check_fn(&[], &t.body, FnContext::Test);
    }
    SemaResult { resolutions: a.resolutions, errors: a.errors}
    
}

struct Analyzer{
    fn_names: HashSet<String>,
    component_names: HashSet<String>,
    layout_names: HashSet<String>,
    page_names: HashSet<String>,
    enum_variants: HashMap<String,String>,
    component_params: HashMap<String, Vec<String>>,
    struct_names: HashSet<String>,
    struct_fields: HashMap<String, Vec<String>>,
    route_keys : HashSet<(String, String)>,
    global_state_names: HashSet<String>,
    store_names: HashSet<String>,
    client_extern_names: HashSet<String>,
    server_extern_names: HashSet<String>,
    resolutions: HashMap<ExprId,Resolution>,
    errors: Vec<Spanned<SemaError>>,

}
impl Analyzer {
    fn push(&mut self, span: Option<Span>, error: SemaError){
        self.errors.push(Spanned{ span, error});
    }

    fn collect_globals(&mut self, module: &Module){
        for f in &module.fns{
            if !self.fn_names.insert(f.name.clone()){
                self.push(Some(f.name_span), SemaError::DuplicateTopLevelName { name: f.name.clone(), first_kind: "fn", second_kind: "fn" });
            }
        }
        for c in &module.components{
            if self.fn_names.contains(&c.name){
                self.push(Some(c.name_span), SemaError::DuplicateTopLevelName { name: c.name.clone(), first_kind: "fn", second_kind: "component" });
            }
            if !self.component_names.insert(c.name.clone()){
                self.push(Some(c.name_span), SemaError::DuplicateTopLevelName { name: c.name.clone(), first_kind: "component", second_kind: "component"});
            }
            self.component_params.insert(c.name.clone(),c.params.iter().map(|p| p.name.clone()).collect());
        }
        for s in &module.structs{
            if !self.struct_names.insert(s.name.clone()){
                self.push(Some(s.name_span), SemaError::DuplicateTopLevelName { name: s.name.clone(), first_kind: "struct", second_kind: "struct" });
            }
            self.struct_fields.insert(s.name.clone(), s.fields.iter().map(|(n,_)| n.clone()).collect());
        }
        for l in &module.layouts {
            if !self.layout_names.insert(l.name.clone()){
                self.push(Some(l.name_span),SemaError::DuplicateTopLevelName { name: l.name.clone(), first_kind: "layout", second_kind: "layout" });
            }
        }
        for p in &module.pages{
            if !self.page_names.insert(p.name.clone()){
                self.push(Some(p.name_span), SemaError::DuplicateTopLevelName { name: p.name.clone(), first_kind: "page", second_kind: "page" });
            }
        }
        for e in &module.enums {
            for v in &e.variants{
                self.enum_variants.insert(v.clone(), e.name.clone());
            }
        }
        for r in &module.routes {
            let key = (method_str(r.method).to_string(), r.path.clone());
            if !self.route_keys.insert(key) {
                self.push(Some(r.method_span), SemaError::DuplicateRoute { method: method_str(r.method).to_string(), path: r.path.clone() });
            }
            
        }
        // for Persistence
        for s in &module.stores {
            if !self.store_names.insert(s.name.clone()){
                self.push(Some(s.name_span), SemaError::DuplicateTopLevelName { name: s.name.clone(), first_kind: "store", second_kind: "store" });
            }
        }

        //JS interop: split by which side the extern is sourced on
        for e in &module.externs {
            let names = match &e.target {
                ExternTarget::ClientGlobal { .. } | ExternTarget::ClientModule { .. } => &mut self.client_extern_names,
                ExternTarget::ServerNpm { .. } => &mut self.server_extern_names,
            };
            if !names.insert(e.name.clone()){
                self.push(Some(e.name_span), SemaError::DuplicateTopLevelName { name: e.name.clone(), first_kind: "extern", second_kind: "extern" });
            }

        }
        for s in &module.state_decls{
            self.global_state_names.insert(s.name.clone());
        }
        for p in &module.pages {
            for s in &p.state_decls {
                self.global_state_names.insert(s.name.clone());
            }
        }
        for c in &module.components{
            for s in &c.state_decls {
                self.global_state_names.insert(s.name.clone());
            }
        }
       
    }

    fn check_layout_used(&mut self, module : &Module){
        for p in &module.pages{
            if let Some(layout) = &p.uses{
                if !self.layout_names.contains(layout){
                    self.push(Some(p.name_span), SemaError::UnKnownLayout { page: p.name.clone(), layout: layout.clone() });
                }
            }
        }
    }


    // The entire enforcement mechanism for every visisbility rule in
    // this crate: what gets seeded into the initial scope IS the
    //policy. a name never seeded here is simply unresolved in that
    //context, falling through to the ordinary UnKnownIdentfier

    fn seed_scope(&self, ctx: FnContext) -> Scope {
        let mut scope = Scope::new();
        for name in &self.global_state_names{
            scope.insert(name.clone(), Resolution::State);
        }
        match ctx{
            FnContext::Route => {
                for name in &self.store_names{
                    scope.insert(name.clone(), Resolution::Store);
                }
                for name in &self.server_extern_names{
                    scope.insert(name.clone(), Resolution::Extern);
                }
            }
            FnContext::ClientOnly => {
                for name in &self.client_extern_names{
                    scope.insert(name.clone(), Resolution::Extern);
                }

            }
            FnContext::Test => {
                for name in &self.server_extern_names{
                    scope.insert(name.clone(), Resolution::Extern);
                }
            }
            FnContext::Shared => {}
        }
        scope
    }

    fn check_fn(&mut self, params: &[Param], body: &[Stmt], ctx: FnContext){
        let mut scope = self.seed_scope(ctx);
        for p in params{
            scope.insert(p.name.clone(), Resolution::Param);
        }
        self.check_stmts(body, &mut scope);
    }

    fn check_stmts(&mut self, stmts: &[Stmt], scope: &mut Scope){
        for s in stmts{
            self.check_stmt(s,scope);
        }
    }
    fn check_stmt(&mut self, s: &Stmt, scope: &mut Scope){
        match s {
            Stmt::Let { name, value , ..} => {
                self.check_expr(value,scope);
                scope.insert(name.clone(), Resolution::Local);
            }
            Stmt::Assign { target, value } => {
                self.check_expr(value, scope);
                self.check_assign_target(target, scope);
            }
            Stmt::If { cond, then_branch, else_branch } => {
                self.check_expr(cond, scope);
                let mut then_scope = scope.clone();
                self.check_stmts(then_branch, &mut then_scope);
                if let Some(eb) = else_branch {
                    let mut else_scope = scope.clone();
                    self.check_stmts(eb, &mut else_scope);
                }
            }
            Stmt::For { var, iter, body,.. } => {
                self.check_expr(iter,scope);
                let mut body_scope = scope.clone();
                body_scope.insert(var.clone(), Resolution::Local);
                self.check_stmts(body, &mut body_scope);
            }
            Stmt::While { cond, body } =>{
                self.check_expr(cond, scope);
                let mut body_scope = scope.clone();
                self.check_stmts(body, &mut body_scope);
            }
            Stmt::Return(Some(e)) => self.check_expr(e, scope),
            Stmt::Return(None) => {}
            Stmt::Assert { expr, .. } => self.check_expr(expr,scope),
            Stmt::Expr(e) => self.check_expr(e, scope),

        }
    }

    fn check_assign_target(&mut self, target: &LValue, scope: &Scope){
           match scope.get(&target.name){
                Some(Resolution::Param) => self.push(None, SemaError::AssignToParam { name: target.name.clone() }),
                Some(Resolution::State) | Some(Resolution::Local) => {}
                Some(_) | None => self.push(None, SemaError::AssignToUnKnownName { name: target.name.clone() }),


           }
           for acc in &target.accessors{
            if let Accessor::Index(e) = acc {
                self.check_expr(e, scope);
            }
           }


    }
    fn resolve_ident(&self, name: &str, scope : &Scope) -> Option<Resolution> {
        if let Some(r) = scope.get(name){
            return  Some(r.clone());
        }
        if self.fn_names.contains(name){
            return Some(Resolution::Fn);
        }
        if self.component_names.contains(name){
            return Some(Resolution::Component);
        }
        if let Some(enum_name) = self.enum_variants.get(name){
            return Some(Resolution::EnumVariant { enum_name: enum_name.clone() });
        }
        if is_builtin_fn(name) {
            return Some(Resolution::Fn); // parseInt/ awaitAll - checked last , deliberately, so user decls always shadow
        }
        None
    }

    fn check_expr(&mut self, e: &ExprNode, scope: &Scope){
        match &e.kind {
            ExprKind::Ident(name)=> match self.resolve_ident(name, scope){
                Some(Resolution::Store) => self.push(Some(e.span), SemaError::StoreUsedDirectly { name: name.clone() }),
                Some(Resolution::Extern) => self.push(Some(e.span), SemaError::ExternUsedDirectly { name: name.clone() }),
                Some(r) => {
                    self.resolutions.insert(e.id, r);
                }
                None => self.push(Some(e.span), SemaError::UnKnownIdentifier { name: name.clone() }),
            }
            ExprKind::List(items) => {
                for it in items {
                    self.check_expr(it, scope);
                }
            }
            ExprKind::StructLit { type_name, fields } => {
                if !self.struct_names.contains(type_name){
                    self.push(Some(e.span), SemaError::UnKnownStruct { name: type_name.clone() });
                    for(_, v) in fields {
                        self.check_expr(v, scope);
                    }
                    return ;
                }
                let declared = self.struct_fields.get(type_name).cloned().unwrap_or_default();
                let mut seen = HashSet::new();
                for(fname, fvalue) in fields {
                    if !declared.contains(fname) {
                        self.push(
                            Some(fvalue.span),
                            SemaError::UnKnownStructField { struct_name: type_name.clone(), field: fname.clone()},
                        );
                    } else if !seen.insert(fname.clone()) {
                        self.push(Some(fvalue.span), SemaError::DuplicateStructField { struct_name: type_name.clone(), field: fname.clone() });
                    }
                    self.check_expr(fvalue, scope);
                }
                for dname in &declared {
                    if !fields.iter().any(|(n,_)| n == dname){
                        self.push(Some(e.span), SemaError::MissingStructField { struct_name: type_name.clone(), field: dname.clone() });

                    }
                }
            }
            ExprKind::Unary {  expr , ..} => self.check_expr(expr, scope),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.check_expr(lhs, scope);
                self.check_expr(rhs, scope);
            }
            ExprKind::Call { callee, args } => {
                // Store method calls (tasks.all()) are recognized
                // here, BEFORE the generic Ident/Field handling below-
                //this is the one call shape that needs special structural 
                // validation (method-name checking) rather 
                // than falling through to ordinary field-access rules
                if let ExprKind::Field { base, name:method }= &callee.kind{
                    if let ExprKind::Ident(store_name) = &base.kind{
                        if let Some(Resolution::Store) = self.resolve_ident(store_name, scope){
                            self.resolutions.insert(base.id, Resolution::Store);
                            const STORE_METHOD : &[&str] = &["all", "find", "insert", "update", "delete"];
                            if !STORE_METHOD.contains(&method.as_str()){
                                self.push(Some(callee.span), SemaError::UnKnownStoreMethod { store: store_name.clone(), method: method.clone() });
                            }
                            for a in args {
                                self.check_expr(&a.value, scope);
                            }
                            return;
                        }
                    }
                }
                if let ExprKind::Ident(name) = &callee.kind{
                    match self.resolve_ident(name, scope) {
                        Some(Resolution::Fn) => {
                            self.resolutions.insert(callee.id,Resolution::Fn);
                        }
                        Some(Resolution::Component) => self.push(Some(callee.span), SemaError::ComponentCallableAsPlainFunction { name: name.clone() }),
                        // Extern falls in here too - calling one is
                        // exactly the shape a Fn call already has , no
                        // special- casing needed (unlike Store, which needed the .method() syntax handled above)
                        Some(other) => {
                            self.resolutions.insert(callee.id, other);
                        }
                        None => self.push(Some(callee.span), SemaError::UnKnownCallable { name: name.clone() }),
                    }
                }  else {
                        self.check_expr(callee, scope);
                    }
                    for a in args {
                        self.check_expr(&a.value, scope);
                    }
                }
                  
                ExprKind::Field { base , ..} => self.check_expr(base, scope),
                ExprKind::Index { base, index } => {
                    self.check_expr(base, scope);
                    self.check_expr(index, scope);
                }
                ExprKind::AwaitFetch {  url , ..} => self.check_expr(url, scope),
                ExprKind::Await { expr } => self.check_expr(expr, scope),
                ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Str(_) | ExprKind::Bool(_) | ExprKind::Color(_) | ExprKind::Size(_,_) => {}
              
            }

        }
        fn check_node(&mut self, node: &Node, scope: &Scope, in_layout: bool) {
            match &node.kind {
                NodeKind::Element { kind, inline_arg, properties, events, children } => {
                    if matches!(kind, ElementKind::Slot) && !in_layout{
                        self.push(Some(node.span), SemaError::SlotUsedOutSideLayout);
                    }
                    if let Some(arg) = inline_arg{
                        self.check_expr(arg, scope);
                    }
                    for p in properties {
                        match &p.value {
                            PropertyValue::Single(e) => {
                              let is_bare_keyword =  matches!(e.kind, ExprKind::Ident(_)) &&  is_key_word_enum_property(&p.name);
                              if !is_bare_keyword {
                                self.check_expr(e, scope);
                              }
                                
                            }
                            PropertyValue::Box { top, right, bottom, left } => {
                                self.check_expr(top, scope);
                                self.check_expr(right, scope);
                                self.check_expr(bottom, scope);
                                self.check_expr(left, scope);
                            }
                        }
                    }
                    for ev in events {
                        self.check_event(*kind, ev, scope);
                    }
                    for c in children {
                        self.check_node(c, scope, in_layout);
                    }
                }
                NodeKind::ComponentCall { name, args, children } => {
                    if !self.component_names.contains(name) {
                        self.push(Some(node.span), SemaError::UnKnownCallable { name: name.clone() });
                    } else {
                        if !children.is_empty(){
                            self.push(Some(node.span), SemaError::ComponentCallHasChildren { component: name.clone() });
                        }
                        self.check_component_args(name, args, node.span);
                    }
                    for a in args {
                        self.check_expr(&a.value, scope);
                    }
                    for c in children {
                        self.check_node(c, scope, in_layout);
                    }
                }
                NodeKind::If { cond, then_branch, else_branch } => {
                    self.check_expr(cond, scope);
                for c in then_branch {
                    self.check_node(c, scope, in_layout);
                }
                if let Some(eb) =  else_branch {
                    for c in eb {
                        self.check_node(c, scope, in_layout);
                    }
                }
                    
                }
                NodeKind::For { var,  iter, body, .. } => {
                    self.check_expr(iter, scope);
                    let mut body_scope = scope.clone();
                    body_scope.insert(var.clone(), Resolution::Local);
                    for c in body {
                        self.check_node(c, &body_scope, in_layout);
                    }
                }
                }



                    
                
            }

            fn check_component_args(&mut self, component : &str, args: &[Arg], call_span : Span){
                let Some(param_names) = self.component_params.get(component).cloned() else { return};
                let mut seen_named = HashSet::new();
                let mut positional_count = 0usize;
                for a  in args {
                    match &a.name {
                        Some(n) => {
                            if !param_names.contains(n) {
                                self.push(Some(a.value.span), SemaError::UnKnownComponentParam { component: component.to_string(), param: n.clone() });
                            }
                            else if !seen_named.insert(n.clone()) {
                                self.push(
                                    Some(a.value.span),
                                    SemaError::DuplicateNamedArgument { component: component.to_string(), arg: n.clone()}
                                );
                            }

                        }
                        None => positional_count += 1,
                    }
                }
                if positional_count > param_names.len(){
                    self.push(Some(call_span), SemaError::TooManyPositionalArgs { component: component.to_string(), expected: param_names.len(), found: positional_count});
                }

            }

            fn check_event(&mut self, kind: ElementKind, ev: &Event, scope: &Scope){
                if !allowed_events(kind).contains(&ev.name.as_str()){
                    self.push(None, SemaError::UnKnownEvent { element: attr_name(kind).to_string(), event: ev.name.clone() });
                }
                match &ev.handler {
                    EventHandler::Call(e) => self.check_expr(e, &scope),
                    EventHandler::Lambda { params, body } => {
                        let mut lambda_scope = scope.clone();
                        for p in params{
                            lambda_scope.insert(p.clone(), Resolution::Local);
                        }
                        match body {
                            LambdaBody::Expr(e) => self.check_expr(e, &lambda_scope),
                            LambdaBody::Assign { target, value } => {
                                self.check_expr(value, &lambda_scope);
                                self.check_assign_target(target, &lambda_scope);
                            }
                        }
                    }
                }
            }

        }
fn method_str(m: HttpMethod) -> &'static str {
    match m {
                HttpMethod::Get => "GET",
                HttpMethod::Post => "POST",
                HttpMethod::Put => "PUT",
                HttpMethod::Delete => "DELETE",
                HttpMethod::Patch => "PATCH"
            }
        }

fn is_key_word_enum_property(name: &str) -> bool {
    matches!(name, "align" | "justify" |"fontweight")
        }


        //the first true language builtins- recognized here at name- resolution
        fn is_builtin_fn(name: &str) -> bool {
            matches!(name, "parseInt" | "awaitAll")
        }
        fn allowed_events(kind: ElementKind)-> &'static [&'static str] {
            use ElementKind::*;
            match kind {
                Button => &["click"],
                Input | Textarea | Dropdown => &["change", "focus", "blur"],
                Checkbox | Switch | Radio =>  &["change"],
                Row | Column | Stack | Container | Card | Grid | List | Table => &["click", "hover"],
                Navigation | Tabs | Menu => &["click"],
                Dialog | Modal => &["click"],
                Text | Image | Icon | Slot => &[],

                
            }
        }
    
#[cfg(test)]
mod tests {
    use super::*;
    use p_ast::lower;
    use p_lexer::{FileId,  tokenize};
    use p_parser::parse;

    fn analyze_src(src: &str) -> SemaResult {
        let tokens = tokenize(src, FileId(0)).unwrap();
        let cst = parse(&tokens).unwrap();
        let module = lower(cst).unwrap();
        analyzer(&module)
    }

    #[test]
    fn stage1_no_failed_scope_bug(){
        let src =
r#"
page Home
    state count: Int = 0

    text count

fn increment()-> Void

        count = count + 1




"#;
        let r = analyze_src(src);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
    }
    #[test]
    fn store_visible_in_route(){
        let src = 
r#"
struct T
    id: Int


store ts: List<T>

route GET "x" -> T
    return ts.all()






"#;
        let good = analyze_src(src);
        assert!(good.errors.is_empty(), "{:?}", good.errors);

    }
    #[test]
    fn server_extern_visible_in_route_not_test(){
        let src = 
r#"
struct Dealer
    id: Int
    age: Int
    name: String
    dealer: Owner

struct Owner
    name: String


extern fn hashSync(s: String) -> String server npm "bcrypt"

route POST "/dealer" -> Dealer
    let h = hashSync("pw")
    return Dealer {id: 1, age: 20, name: "Acme", dealer: Owner{ name: "Ty"} }



"#;
        let good = analyze_src(src);
        assert!(good.errors.is_empty(), "{:?}", good.errors);
    }

}