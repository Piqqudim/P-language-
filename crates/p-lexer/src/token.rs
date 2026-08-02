use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    
    //Byte offset of the first character
    pub start : usize,
    //Byte offset one past the last character so that [span.start..span.end] is always the exact lexeme
    pub end : usize,
    //1- based line number of 'start'
    pub  line : u32,
    // 1-based column number of 'start'
    pub col : u32,


}

// This is a constructor for the span, mostly when a span object get created or initialized

impl  Span {
    pub fn new(start : usize, end : usize, line: u32, col : u32) -> Self {
        Self { start, end, line, col }

    }
}


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SizeUnit {
    //These units exist for Css literals and the Units recognized on a "Size" literal {""}SIZE
    Px,
    Rem,
    Em,
    Percent,
    Vm,
    Vh,

}
//For easy understanding for those people about to contribute
//this works like interface in C# and Java majorly the imp keyword substitute for interface here

impl fmt::Display for SizeUnit{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let object = match self {
            SizeUnit::Px => "px",
            SizeUnit::Percent => "%",
            SizeUnit::Em => "em",
            SizeUnit::Rem => "rem",
            SizeUnit::Vh => "vh",
            SizeUnit::Vm => "vm",

            
        };
        write!(f, "{object}")
        
    }

}

//This is the real work
//Every terminal symbol the lexer can produce
//Some things don't correspond to any character span the developer type , but they still carry a 'Span'(the position where the lexer decided to emit them) purely for diagnostic
