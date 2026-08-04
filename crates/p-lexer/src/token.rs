use std::fmt;

//Node Declaratives

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
#[derive(Debug,Clone,PartialEq)]
pub enum TokenKind {
    // --- Structural (synthetic) in P language majorly for files structure
    Indent,
    Dedent,
    Newline,
    Eof, // End-of-file


    // -- Literals that exist in P language
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),

    //Normalized to lower case hex for css without the leading '""' / trailing '""',
    //e.g the source "#FFF" becomes "Color("#fff"._to_string())
    Color(String),
    Size(f64, SizeUnit),


    //Identifier
    Ident(String),

    //Declaration keywords for P language
    Page,
    Component,
    Layout,
    State,
    Fn,
    Let,
    Return,
    If,
    For,
    Else,
    While,
    In,
    Import,
    Enum,
    Uses,
    Slot,
    On,
    And,
    Or,
    Not,


    //-- Node-Kind keywords for P language
    Row,
    Column,
    Stack,
    Container,
    Card,
    Grid,
    List,
    Text,
    Image,
    Icon,
    Input,
    Textarea,
    Button,
    Checkbox,
    Switch,
    Radio,
    Dropdown,
    Table,
    Navigation,
    Tabs,
    Dialog,
    Modal,
    Menu,




    // --- Type Keywords for P language
    IntType,
    FloatType,
    StringType,
    BoolType,
    ColorType,
    SizeType,
    VoidType,
    ListType,
    MapType,
    OptionType,


    // Punctuation / operators that exist in P language
    Colon, // :
    Comma, // ,
    Dot, // .
    LParen, // (
    RParen, // )
    LBracket, // {
    RBracket, // }
    Arrow, // ->
    FatArrow, // =>
    Assign, // = 
    Plus , // +
    Minus, // -
    Star, // *
    Slash, // /
    Percent, // %
    EqEq, // ==
    NotEq, // !=
    Lt, // <  (we can use this also for opens generics : List<Int>)
    Gt, // > (also similar to the Lt)
    LtEq,  // <= 
    GtEq, // >=

}

impl fmt::Display for TokenKind{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        match self {
            Indent => write!(f, "<INDENT>"),
            Dedent => write!(f, "<DEDENT>"),
            Newline => write!(f, "NEWLINE>"),
            Eof => write!(f, "<EOF>"),
            Int(n) => write!(f, "{n}"),
            Float(n) => write!(f, "{n}"),
            Str(s) => write!(f, "{s :?}"),
            Bool(b) => write!(f, "{b}"),
            Color(c) => write!(f, "\"{c}\""),
            Size(n,u ) => write!(f, "{n}{u}"),
            Ident(s) => write!(f, "{s}"),
            Page => write!(f, "page"),
            Component => write!(f, "component"),
            Layout => write!(f, "layout"),
            State => write!(f, "state"),
            Fn => write!(f,  "fn"),
            Let => write!(f, "let"),
            Return => write!(f, "return"),
            If => write!(f, "if"),
            Else => write!(f, "if"),
            For => write!(f, "for"),
            While => write!(f, "while"),
            In => write!(f, "in"),
            Import => write!(f, "import"),
            Enum => write!(f, "enum"),
            Uses => write!(f, "uses"),
            Slot => write!(f, "slot"),
            On => write!(f, "on"),
            And => write!(f, "and"),
            Or => write!(f, "or"),
            Not => write!(f,"not"),
            Row => write!(f,"row"),
            Column => write!(f, "column"),
            Stack => write!(f, "stack"),
            Container => write!(f, "container"),
            Card => write!(f, "card"),
            Grid => write!(f, "grid"),
            List => write!(f,"list"),
            Text => write!(f, "text"),
            Image => write!(f, "image"),
            Icon => write!(f, "icon"),
            Input => write!(f, "input"),
            Textarea => write!(f, "textarea"),
            Button => write!(f, "button"),
            Checkbox => write!(f,"checkbox"),
            Switch => write!(f, "switch"),
            Radio => write!(f, "radio"),
            Dropdown => write!(f, "dropdown"),
            Table => write!(f, "table"),
            Navigation => write!(f, "navigation"),
            Tabs =>  write!(f, "tabs"),
            Dialog => write!(f, "dialogs"),
            Modal => write!(f, "modal"),
            Menu => write!(f, "menu"),
            IntType => write!(f, "Int"),
            FloatType => write!(f, "Float"),
            StringType => write!(f, "String"),
            BoolType => write!(f, "Bool"),
            ColorType => write!(f, "Color"),
            SizeType => write!(f, "Size"),
            VoidType => write!(f, "Void"),
            ListType => write!(f, "List"),
            MapType => write!(f, "Map"),
            OptionType => write!(f, "Option"),
            Colon => write!(f, ":"),
            Comma => write!(f, ","),
            Dot => write!(f, "."),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            LBracket => write!(f, "["),
            RBracket => write!(f, "]"),
            Arrow => write!(f, "->"),
            FatArrow => write!(f, "=>"),
            Assign => write!(f, "="),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Star => write!(f, "*"),
            Slash => write!(f, "/"),
            Percent => write!(f, "%"),
            EqEq => write!(f, "=="),
            NotEq => write!(f, "!="),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            LtEq => write!(f, "<="),
            GtEq => write!(f, ">="),

        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token{
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind : TokenKind, span : Span) -> Self {
        Self { kind, span }
    }
}


//We need a lookup table and we need a id-> identifier to a particular keywords or keyword table
//Return "None" for anything thst should lex as a plain 'Ident

//'true' and 'false' are intentionally handled by the lexer directly as
// 'TokenKind::Bool' not through this table, since they are literals, not keywords in the token-kind sense
pub fn keyword_lookup(ident : &str) -> Option<TokenKind>{
    use TokenKind::*;
    Some(match ident {
        "page" => Page,
        "component" => Component,
        "layout" => Layout,
        "state" => State,
        "fn" => Fn,
        "let" => Let,
        "return" => Return,
        "if" => If,
        "else" => Else,
        "for" => For,
        "while" => While,
        "in" => In,
        "import" => Import,
        "enum" => Enum,
        "uses" => Uses,
        "slot" => Slot,
        "on" => On,
        "and" => And,
        "or" => Or,
        "not" => Not,
        "row" => Row,
        "column" => Column,
        "stack" => Stack,
        "container" => Container,
        "card" => Card,
        "grid" => Grid,
        "text" => Text,
        "image" => Image,
        "icon" => Icon,
        "input" => Input,
        "textarea" => Textarea,
        "button" => Button,
        "checkbox" => Checkbox,
        "switch" => Switch,
        "radio" => Radio,
        "dropdown" => Dropdown,
        "table" => Table,
        "navigation" => Navigation,
        "tabs" => Tabs,
        "dialog" => Dialog,
        "modal" => Modal,
        "menu" => Menu,
        "Int" => IntType,
        "Float" => FloatType,
        "String" => StringType,
        "Bool" => BoolType,
        "Color" => ColorType,
        "Size" => SizeType,
        "Void"=> VoidType,
        "List"=> ListType,
        "Map" => MapType,
        "Option" => OptionType,
        _ => return None,
    })

}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn keyword_lookup_covers_every_grammar_9_keyword(){



        let  keyword  = ["page", "component","layout","state", "fn", "let","return",
                        "if" , "else", "for", "while","in", "import", "enum", "uses",
                        "slot", "on","row", "column", "stack", "container", "card",
                        "grid", "list", "text", "image", "icon", "input","textarea",
                        "button","checkbox","switch","radio","dropdown","table",
                        "navigation","tabs","dialog","modal","menu","and","or",
                        "not", "Int", "Float","String", "Bool", "Color","Size",
                        "Void","List","Map","Option", ];
        
        
       
        for kw in keyword {
            assert!(keyword_lookup(kw).is_some(), "expected {kw:?} to be a recognized keyword")
        }
    }
}