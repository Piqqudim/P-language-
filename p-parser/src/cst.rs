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
    pub value : Expr,

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
    pub root: Vec<UiNode>,

}

#[derive(Debug,Clone,PartialEq)]
pub struct  LayoutDecl{
    pub name: String,
    pub name_span: Span,
    pub root: UiNode,

}

#[derive(Debug,Clone,PartialEq)]
pub struct PageDecl{
    pub name : String,
    pub name_span : Span,
    pub uses: Option<String>,
    pub state_decls: Vec<StateDecl>,
    pub fns: Vec<FnDecl>,
    pub root: Vec<UiNode>

}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum HttpMethod {Get, Post, Put, Delete, Patch }

#[derive(Debug,Clone,PartialEq)]
pub struct RouteDecl {
    pub method : HttpMethod,
    pub method_span: Span,
    pub path: String,
    pub body_ty : Option<TypeExpr>,
    pub ret : TypeExpr,
    pub body :Vec<Stmt>,

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
    ClientModule { url : String, export : Option<String>},
    ServerNpm {package: String, export : Option<String> },
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
pub enum NodeKind{
    Row, Column,Stack,Container,Card,Grid,List,Text,Image,Icon,
    Input,Textarea,Button,Checkbox,Switch,Radio,Dropdown,Table,
    Navigation,Tabs,Dialog,Modal,Menu,Slot,
}

#[derive(Debug, Clone,PartialEq)]
pub struct Node{
    pub id: NodeId,
    pub span: Span,
    pub kind: NodeKind,

}
#[derive(Debug,Clone,PartialEq)]
pub enum UiNode{
    Kind {kind: NodeKind, inline_arg : Option<Expr>, body: Vec<NodeBodyItem>,span:Span},
    Call {name: String, args:Vec<Arg>, body:Vec<NodeBodyItem>, span:Span},
}
 impl UiNode{
    pub fn span(&self)->Span{
        match self {
            UiNode::Kind{span, .. } => *span,
            UiNode::Call {span, ..} =>*span,
        }
    }
 }

 #[derive(Debug,Clone,PartialEq)]
 pub enum NodeBodyItem {
    Property(PropertyStmt),
    Event(EventStmt),
    Node(UiNode),
    If(IfNode),
    For(ForNode)
 }
#[derive(Debug,Clone,PartialEq)]
pub struct PropertyStmt{
    pub name: String,
    pub values: Vec<Expr>,
    pub span : Span,
}


#[derive(Debug,Clone,PartialEq)]
pub struct EventStmt{
    pub name: String,
    pub handler: EventHandler,
}

#[derive(Debug,Clone,PartialEq)]
pub enum EventHandler {
    Call(Expr),
    Lambda {params : Vec<String>, body:LambdaBody},
}

#[derive(Debug,Clone,PartialEq)]
pub enum LambdaBody{
    Expr(Expr),
    Assign{target :LValue, value:Expr},
}
#[derive(Debug,Clone,PartialEq)]
pub struct IfNode{
    pub cond: Expr,
    pub then_branch: Vec<UiNode>,
    pub else_branch: Option<Vec<UiNode>>,
    pub span: Span,
}

#[derive(Debug,Clone,PartialEq)]
pub struct ForNode{
    pub var : String,
    pub var_span: Span,
    pub iter: Expr,
    pub body: Vec<UiNode>,
    pub span : Span,
}
#[derive(Debug,Clone,PartialEq)]
pub struct Expr{
    pub kind: ExprKind,
    pub span: Span,
}
#[derive(Debug,Clone,PartialEq)]
pub struct LValue{
    pub name: String,
    pub accessors : Vec<Accessor>,
}

#[derive(Debug,Clone,PartialEq)]
pub enum Accessor{
    Field(String),
    Index(Expr),
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
    List(Vec<Expr>),
    StructLit {type_name :String, fields : Vec<(String, Expr)>},
    Unary {op : UnaryOp, expr: Box<Expr>},
    Binary {op: BinaryOp, lhs:Box<Expr>, rhs: Box<Expr>},
    Call { callee : Box<Expr>, args : Vec<Arg>},
    Field {base :Box<Expr>, name : String},
    Index {base :Box<Expr>, index : Box<Expr>},
    AwaitFetch {type_args : TypeExpr, url : Box<Expr>},
    Await {expr : Box<Expr>}

}

#[derive(Debug,Clone,PartialEq)]
pub struct Arg {
    pub name : Option<String>,
    pub value : Expr,
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
    Let{name: String, name_span: Span, ty: Option<TypeExpr>, value: Expr},
    Assign {target: LValue, value: Expr},
    If {cond: Expr, then_branch:Vec<Stmt>, else_branch:Option<Vec<Stmt>>},
    For{var : String, var_span :Span, iter: Expr, body : Vec<Stmt>},
    While{ cond: Expr, body : Vec<Stmt>},
    Return(Option<Expr>),
    Assert{expr: Expr, span:Span},
    Expr(Expr),

}