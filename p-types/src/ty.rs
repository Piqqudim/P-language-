use p_ast::BinaryOp::Or;
use p_ast::TypeExpr;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug,Clone,PartialEq)]
pub enum Ty {
    Int,
    Float,
    String,
    Bool,
    Color,
    Size,
    Void,
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    Option(Box<Ty>),
    Enum(String),

    //If a following type is unknown ,  we can actually declared it with Ty::Unknown
    Unknown
}

impl Ty {
    pub fn compatible(&self, other: &Ty) -> bool {
        if *self == Ty::Unknown || *other == Ty::Unknown {
            return  true;
        } 
        self == other
    }
}
impl fmt::Display for Ty {
    fn fmt(&self, f:&mut fmt::Formatter<'_>)-> fmt::Result{
        match self {
            Ty::Int => write!(f,"Int"),
            Ty::Float => write!(f,"Float"),
            Ty::String => write!(f,"String"),
            Ty::Bool => write!(f,"Bool"),
            Ty::Color => write!(f,"Color"),
            Ty::Size => write!(f, "Size"),
            Ty::Void => write!(f, "Void"),
            Ty::List(t) => write!(f,"List<{t}>"),
            Ty::Map(k, v) => write!(f, "Map<{k},{v}>"),
            Ty::Option(t) => write!(f, "Option<{t}>"),
            Ty::Enum(n) => write!(f,"{n}"),
            Ty::Unknown => write!(f,"?"),
        }
    }
}

pub fn lower_type_expr(t: &TypeExpr, enum_names: &HashSet<String>) ->(Ty,Option<String>){
    match t {
        TypeExpr::Int =>(Ty::Int, None),
        TypeExpr::Float => (Ty::Float,None),
        TypeExpr::String => (Ty::String,None),
        TypeExpr::Bool => (Ty::Bool, None),
        TypeExpr::Color => (Ty::Color,None),
        TypeExpr::Size => (Ty::Size,None),
        TypeExpr::Void => (Ty::Void, None),
        TypeExpr::List (inner) => {
            let(t, unknown) =lower_type_expr(inner, enum_names);
            (Ty::List(Box::new(t)),unknown)
        }
        TypeExpr::Map(k,v )=> {
            let (kt, ku) = lower_type_expr(k, enum_names);
            let (vt, vu) = lower_type_expr(v, enum_names);
            (Ty::Map(Box::new(kt), Box::new(vt)), ku)
            
        }
    
    }
}