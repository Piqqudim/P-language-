use p_lexer::{Span,SizeUnit};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd, Ord)]
pub struct NodeId(pub u32);

#[derive(Debug,Clone,Copy,PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprId(pub u32);
#[derive(Debug,Clone,PartialEq)]
pub struct Module {
    pub items : Vec<TopLevelItem>,
}


#[derive(Debug,Clone,PartialEq)]
pub enum TopLevelItem {
    Import(Vec<String>),
    Enum(EnumDecl),
    Struct(StructDecl),
    State(StateDecl),
    Fn(FnDecl),
    Component(ComponentDecl),
    Layout(LayoutDecl),
    Page(PageDecl),
    Route(RouteDecl),
    Store(StoreDecl),
    Extern(ExternDecl),
    Test(TestDecl),

}

#[derive(Debug,Clone,PartialEq)]
pub struct EnumDecl {

    pub name : String,
    pub name_span :  Span,
    pub variants : Vec<String>,

}

#[derive(Debug,Clone,PartialEq)]
pub struct StructDecl{
    pub name : String,
    pub name_span: Span,
    pub fields : Vec<(String, TypeExpr)>,
    

}

#[derive(Debug,Clone,PartialEq)]
pub struct StateDecl{
    pub name: String,
    pub name_span: Span,
    pub ty: TypeExpr,
    pub value : ExprNode,

}

#[derive(Debug,Clone,PartialEq)]
pub struct Param {
    pub name: String,
    pub name_span : Span,
    pub ty: TypeExpr,

}

#[derive(Debug,Clone,PartialEq)]
pub  struct FnDecl{
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub ret : Option<TypeExpr>,
    pub body : Vec<Stmt>,

}

#[derive(Debug,Clone,PartialEq)]
pub struct ComponentDecl{
    pub name: String,
    pub name_span: Span,
    pub params : Vec<Param>,
    pub state_decls : Vec<StateDecl>,
    pub fns: Vec<FnDecl>,
    pub root: Node

}

#[derive(Debug,Clone,PartialEq)]
pub struct  LayoutDecl{
    pub name: String,
    pub name_span: Span,
    pub root: Node,

}

#[derive(Debug,Clone,PartialEq)]
pub struct PageDecl{
    pub name : String,
    pub name_span : Span,
    pub uses: Option<String>,
    pub state_decls: Vec<StateDecl>,
    pub fns: Vec<FnDecl>,
    pub root: Node

}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum HttpMethod {Get, Post, Put, Delete, Patch }

#[derive(Debug,Clone,PartialEq)]
pub struct RouteDecl {
    pub method : HttpMethod,
    pub method_span: Span,
    pub path: String,

    //Path params (in path order, always String) followed by the
    //"body" param (if any)- folded together here so a route is
    //shape-identical to a plain fn everywhere scope checking happens
    pub params: Vec<Param>,

    //Explicit , NOT rederived from params.len(), we could use params.len(), but not or parameters have a body
    pub has_body :bool,
    pub ret : TypeExpr,
    pub body: Vec<Stmt>,

}


//I added this for persistency
#[derive(Debug,Clone,PartialEq)]
pub struct StoreDecl{
    pub name: String,
    pub name_span: Span,
    pub ty : TypeExpr,


}
//JS interoperability . Exports are fuuly resolved here the Concrete Syntax Tree(CST) is optional as "..." clause is defaulted to the
//declared P-side name at lowering time, once , so every later stage can assume a real export name always exists
#[derive(Debug,Clone,PartialEq)]
pub enum ExternTarget {
    ClientGlobal {name : String},
    ClientModule { url : String, export : String},
    ServerNpm {package: String, export : String },
}

#[derive(Debug,Clone,PartialEq)]
pub struct ExternDecl {
    pub name : String,
    pub name_span : Span,
    pub params : Vec<Param>,
    pub ret : Option<TypeExpr>,
    pub target : ExternTarget,
    

}

#[derive(Debug,Clone,PartialEq)]
pub struct TestDecl {
    pub description: String,
    pub description_span: Span,
    pub body : Vec<Stmt>,


}


#[derive(Debug,Clone,Copy,PartialEq)]
pub enum ElementKind{
    Row, Column, Stack, Container, Card, Grid, List, Text, Image, Icon,
    Input, Textarea, Button, Checkbox, Switch, Radio, Dropdown, Table,
    Navigation, Tabs, Dialog, Modal, Menu, Slot,
}

#[derive(Debug, Clone,PartialEq)]
pub struct Node{
    pub id: NodeId,
    pub span: Span,
    pub kind: NodeKind,

}
#[derive(Debug,Clone,PartialEq)]
pub enum NodeKind {
    Element {
        kind: ElementKind,
        inline_arg: Option<ExprNode>,
        properties : Vec<Property>,
        events : Vec<Event>,
        children: Vec<Node>,
    },

    ComponentCall {
        name : String,
        args : Vec<Arg>,
        children : Vec<Node>,
    },
    If {
        cond: ExprNode,
        then_branch: Vec<Node>,
        else_branch : Option<Vec<Node>>,

    },
    For {
        var: String,
        var_span: Span,
        iter : ExprNode,
        body : Vec<Node>,
    },
}

#[derive(Debug,Clone,PartialEq)]
pub struct Property{
    pub name: String,
    pub value: PropertyValue
}

#[derive(Debug,Clone,PartialEq)]
pub enum PropertyValue{
    Single(ExprNode),
    Box {top: ExprNode , right: ExprNode, bottom:ExprNode,left:ExprNode},
}

#[derive(Debug,Clone,PartialEq)]
pub struct Event {
    pub name: String,
    pub handler: EventHandler,
}

#[derive(Debug,Clone,PartialEq)]
pub enum EventHandler {
    Call(ExprNode),
    Lambda {params : Vec<String>, body:LambdaBody},
}

#[derive(Debug,Clone,PartialEq)]
pub enum LambdaBody{
    Expr(ExprNode),
    Assign{target :LValue, value:ExprNode},
}

#[derive(Debug,Clone,PartialEq)]
pub struct LValue{
    pub name: String,
    pub accessors : Vec<Accessor>,
}

#[derive(Debug,Clone,PartialEq)]
pub enum Accessor{
    Field(String),
    Index(ExprNode),
}

#[derive(Debug,Clone,PartialEq)]
pub struct ExprNode{
    pub id: ExprId,
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind{
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Color(String),
    Size(f64, SizeUnit),
    Ident(String),
    List(Vec<ExprNode>),
    StructLit {type_name :String, fields : Vec<(String, ExprNode)>},
    Unary {op : UnaryOp, expr: Box<ExprNode>},
    Binary {op: BinaryOp, lhs:Box<ExprNode>, rhs: Box<ExprNode>},
    Call { callee : Box<ExprNode>, args : Vec<Arg>},
    Field {base :Box<ExprNode>, name : String},
    Index {base :Box<ExprNode>, index : Box<ExprNode>},
    AwaitFetch {type_args : TypeExpr, url : Box<ExprNode>},
    Await {expr : Box<ExprNode>}

}

#[derive(Debug,Clone,PartialEq)]
pub struct Arg {
    pub name : Option<String>,
    pub value : ExprNode,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum UnaryOp{Neg, Not}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum BinaryOp {Or, And, Eq, NotEq, Lt, Gt, LtEq, GtEq, Add, Sub, Mul, Div, Mod}

#[derive(Debug, Clone,  PartialEq)]
pub enum TypeExpr {
    Int, Float, String, Bool, Color, Size, Void,
    List(Box<TypeExpr>),
    Map(Box<TypeExpr>,Box<TypeExpr>),
    Option(Box<TypeExpr>),
    Named(String),
}

#[derive(Debug,Clone,PartialEq)]
pub enum Stmt{
    Let{name: String, name_span: Span, ty: Option<TypeExpr>, value: ExprNode},
    Assign {target: LValue, value: ExprNode},
    If {cond: ExprNode, then_branch:Vec<Stmt>, else_branch:Option<Vec<Stmt>>},
    For{var : String, var_span :Span, iter: ExprNode, body : Vec<Stmt>},
    While{ cond: ExprNode, body : Vec<Stmt>},
    Return(Option<ExprNode>),
    Assert{expr: ExprNode, span:Span},
    Expr(ExprNode),

}