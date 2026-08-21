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
    StructFieldTypeMismatch { struct_field: String, field : String, expected: Ty},
    FieldAccessUnKnownField { struct_field: String, field: String},
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