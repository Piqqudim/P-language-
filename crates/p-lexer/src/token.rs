
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {

    pub start : usize,

    pub end : usize,

    pub  line : u32,

    pub column : u32,


}

