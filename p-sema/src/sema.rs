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
    }
}