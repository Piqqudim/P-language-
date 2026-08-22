//Let start with this note 1 
// Type checking : expression inference, statement/ property validation
//Ty and TypeExpr -> Ty lowering come fromp-types (Shared with p-ir)
//Current through 
//.some/.value (Options) synthetic fields (the first real runtime semantics ever)

use p_ast::*;
use p_sema::{Resolution,SemaResult};
use p_types::{NamedKind,lower_type_expr,Ty};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError{
    LetAnnotationMismatch {name: String, declared: Ty, inferred: Ty},
    AssignTypeMisMatch {name: String, target: Ty, value: Ty},
    ConditionNotBool {found: Ty},
    ForIterNotList { found: Ty},
    ReturnTypeMismatch {expected:Ty, found: Ty},
    MissingReturnValue { expected : Ty},
    UnexpectedReturnValue {found: Ty},
    BinaryOpTypeMismatch { op: &'static str, lhs: Ty, rhs: Ty},
    UnaryOpTypeMismatch { op: &'static str, operand: Ty},
    EmptyListLiteral,
    ListElementTypeMismatch {first : Ty, found: Ty},
    IndexNotIndexable {found: Ty},
    IndexNotInt {found: Ty},
    CallArgCountMismatch {name: String, expected: usize, found: usize},
    CallArgTypeMismatch { name:String, param_index: usize, expected: Ty, found: Ty},
    InvalidColorLiteral {text: String},
    PropertyTypeMismatch { property: String, expected: &'static str, found: Ty},
    PropertyInvalidKeyword { property : String, allowed: &'static [&'static str], found: String},
    UnKnownEnumType {name: String},
    StructFieldTypeMismatch { struct_name: String, field : String, expected: Ty},
    FieldAccessUnKnownField { struct_name: String, field: String},
    NoFieldAccessSupport {on : Ty},
    FetchTypeArgNotStruct { found: Ty},
    FetchUrlNotString { found: Ty},
    RouteReturnMustBeStruct { found: Ty},
    RouteBodyMustBeStruct { found: Ty},

    StoreElementMustBeStruct { store: String, found: Ty},
    StoreMissingIdField { store: String ,struct_name: String},
    StoreFieldTypeUnsupported { store: String, field: String, found: Ty},

    ExternTypeNotAllowed { context: & 'static str, found: Ty},

    AwaitAllArgNotList { found: Ty},

    AssertConditionNotBool { found: Ty},
}
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::LetAnnotationMismatch { name, declared, inferred } => {
                write!(f, " '{name}': declared type {declared} does not match inferred type {inferred}")
            }
            TypeError::AssignTypeMisMatch { name, target, value } => {
                write!(f, "cannot assign value of type {value} to '{name}' of type {target}")
            }
            TypeError::ConditionNotBool { found } => write!(f, "condition must be Bool, found {found}"),
            TypeError::ForIterNotList { found } => write!(f,"'for ... in' requires a List, found {found}"),
            TypeError::ReturnTypeMismatch { expected, found } => {
                write!(f, "return type mismatch: expected {expected}, found {found} ")
            }
            TypeError::MissingReturnValue { expected } => write!(f, "missing return value, expected {expected}"),
            TypeError::UnexpectedReturnValue { found } => write!(f, "function returns Void but a value of type {found} was returned"),
            TypeError::BinaryOpTypeMismatch { op, lhs, rhs } => {
                write!(f, "operator '{op}' cannot be applied to {lhs} and {rhs}")
            }
            TypeError::UnaryOpTypeMismatch { op, operand } => {
                write!(f, "operator '{op}' cannot be applied to {operand}")
            }
            TypeError::EmptyListLiteral => write!(f, "cannot infer the element type of an empty list literal"),
            TypeError::ListElementTypeMismatch { first, found } => {
                write!(f, "list elements must share one type: first element has {first}, found{found}")
            }
            TypeError::IndexNotIndexable { found } => write!(f, "cannot index into {found}"),
            TypeError::IndexNotInt { found } => write!(f, "list index must be Int, found{found}"),
            TypeError::CallArgCountMismatch { name, expected, found } => {
                write!(f, "'{name}' expects {expected} argument(s), found {found}")
            }
            TypeError::CallArgTypeMismatch { name, param_index, expected, found } => {
                write!(f, " '{name}' argument {param_index}: expected {expected}, found {found}")
            }
            TypeError::InvalidColorLiteral { text } => {
                write!(f, "'{text}' is not a valid Color (expected #hex or a known name)")
            }
            TypeError::PropertyTypeMismatch { property, expected, found } => {
                write!(f, "property '{property}' expects {expected}, found {found}")
            }
            TypeError::PropertyInvalidKeyword { property, allowed, found } => {
                write!(f, "property '{property}' must be one of {allowed:?}, found '{found}'")
            }
            TypeError::UnKnownEnumType { name } => write!(f, "unknown type '{name}'"),
            TypeError::StructFieldTypeMismatch { struct_name, field, expected } => {
                write!(f, "struct '{struct_name}' field '{field}': expected {expected}")
            }
            TypeError::FieldAccessUnKnownField { struct_name, field } => {
                write!(f, "struct '{struct_name}' has no field named '{field}'")
            }
            TypeError::NoFieldAccessSupport { on } => {
                write!(f, "field access ('.') is not supported on type {on}; P has no general record type")
            }
            TypeError::FetchTypeArgNotStruct { found } => {
                write!(f, "fetch<T> requires T to be a declared struct, found {found}")
            }
            TypeError::FetchUrlNotString { found } =>  write!(f, "fetch url must be String, found {found}"),
            TypeError::RouteReturnMustBeStruct { found } => {
                write!(f, "route return type must be a  declared struct, found {found}")
            }
            TypeError::RouteBodyMustBeStruct { found } => {
                write!(f, "route body type must be a declared struct, found {found}")
            }
            TypeError::StoreElementMustBeStruct { store, found } => {
                write!(f, "store '{store}' must hold List<T> where T is a declared struct, found List<{found}>")
            }
            TypeError::StoreMissingIdField { store, struct_name } => {
                write!(f, "store '{store}' holds '{struct_name}', which has no 'id: Int' or 'id:String' field")
            }
            TypeError::StoreFieldTypeUnsupported { store, field, found } => {
                write!(f,"store '{store}' field '{field}' has unsupported type {found}(only Int/Float/String/Bool/Color are allowed in a store's element struct)")
            }
            TypeError::ExternTypeNotAllowed { context, found } => {
                write!(f, "extern {context} type {found} is not allowed - only primitives and List<primitive>")
            }
            TypeError::AwaitAllArgNotList { found } =>{
                write!(f, "awaitAll requires a List argument, found {found}")
            }
            TypeError::AssertConditionNotBool { found } => {
                write!(f, "assert requires a Bool expression, found {found}")
            }

              

        }
    }
}