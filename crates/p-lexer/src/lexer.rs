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
        write!(f, "")
    }
}
