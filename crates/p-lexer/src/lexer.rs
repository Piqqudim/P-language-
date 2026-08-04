use std::fmt;

// This is P scanner : source text file contains a list of tokens which will be Vec<Token>
//
//Implements the indentation-to-INDENT/DEDENT/NEWLINE translation described in the construct of P language
//just like page Home
//               Column
//                  and so on
//use crate::token::{keyword_lookup,SizeUnit,Span,Token,TokenKind};

#[derive(Debug,Clone,PartialEq)]
pub enum LexError{
    // tabs are never valid indentation ( or, in this implementation, anywhere outside a string literal)
    TabCharacter {line : u32, col : u32},
    //A dedent landed on a column that does not match any enclosing indentation level
    InconsistentDedent {line : u32, col : u32},
    //An indent,s width wasn't a multiple of the file's first established indent unit
    IndentNotMultipleOfUnit { line : u32, col : u32, unit : u32},
    UnterminatedString {line : u32 , col : u32},
    UnterminatedBlockComment { line : u32, col : u32},
    InvalidEscape {line : u32, col : u32, ch : char},
    InvalidNumber { line : u32, col: u32},
    UnknownCharacter {line : u32, col : u32, ch : char},
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::TabCharacter { line, col } => write!(f, "Tab character found at line {line}, column {col}"),
            LexError::InconsistentDedent { line, col } => write!(f, "Inconsistent dedent at line {line}, column {col}"),
            LexError::IndentNotMultipleOfUnit { line, col, unit } => write!(f, "Indentation at line {line}, column {col} is not a multiple of the unit size {unit}"),
            LexError::UnterminatedString { line, col } => write!(f, "Unterminated string literal starting at line {line}, column {col}"),
            LexError::UnterminatedBlockComment { line, col } => write!(f, "Unterminated block comment starting at line {line}, column {col}"),
            LexError::InvalidEscape { line, col, ch } => write!(f, "Invalid escape sequence '\\{ch}' at line {line}, column {col}"),
            LexError::InvalidNumber { line, col } => write!(f, "Invalid number format/literal at line {line}, column {col}"),
            LexError::UnknownCharacter { line, col, ch } => write!(f, "Unknown character '{ch}' at line {line}, column {col}"),
        }   
    }
}
impl  std::error::Error for LexError{
    
}

