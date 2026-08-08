use std::fmt;


//Node Declaratives
#[derive(Debug,Clone,Copy,PartialEq,Hash,Eq,PartialOrd, Ord)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {

    pub file : FileId,
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
    pub fn new(file: FileId,start : usize, end : usize, line: u32, col : u32) -> Self {
        Self { file ,start, end, line, col }

    }

    pub fn to(&self, other:Span) ->Span {
        Span{file:self.file,start:self.start,end:other.end,line:self.line,col:self.col}
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
    Struct,
    Await,
    Fetch,
    Route,
    Body,
    Get,
    Post,
    Put,
    Delete,
    Patch,

    //For persistency
    Store,

    // Added for Javascript interoperability
    Extern,
    Client,
    Server,
    Global,
    Module,
    Npm,
    As,

    //Added for testing 
    Test,
    Assert,


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
    RBracket,
    LBrace,
    RBrace, // }
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
            Struct => write!(f,"struct"),
            Await =>write!(f,"await"),
            Fetch => write!(f, "fetch"),
            Route => write!(f,"route"),
            Body => write!(f, "body"),
            Get => write!(f, "GET"),
            Post => write!(f, "POST"),
            Put => write!(f, "PUT"),
            Delete => write!(f, "DELETE"),
            Patch => write!(f, "PATCH"),
            Store => write!(f, "store"),
            Extern => write!(f, "extern"),
            Client => write!(f, "client"),
            Server => write!(f, "server"),
            Global => write!(f, "global"),
            Module => write!(f, "module"),
            Npm => write!(f, "npm"),
            As => write!(f, "as"),
            Test => write!(f, "test"),
            Assert => write!(f, "assert"),


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
            LBrace => write!(f,"{{"),
            RBrace => write!(f,"}}"),
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
#[derive(Debug, Clone, PartialEq)]
pub struct  Spanned<E>{
    pub span: Option<Span>,
    pub error :  E,
}
impl<E> Spanned<E> {
    pub fn at(span : Span, error: E) -> Self {
        Self{span : Some(span), error}
    }
    pub fn unspanned(error : E) ->Self {
        Self {span : None, error}
    }
}


//We need a lookup table and we need a id-> identifier to a particular keywords or keyword table
//Return "None" for anything thst should lex as a plain 'Ident

//'true' and 'false' are intentionally handled by the lexer directly as
// 'TokenKind::Bool' not through this table, since they are literals, not keywords in the token-kind sense

//'parseInt' and 'awaitAll' are not here - they are ordinary builtin FUNCTION names recognized later in p-sema(semantics) "is_builtin_fn"
//not reserved keywords.
//In the p sema {the p-sema's name resolution precedence - user declarations checked before builtins- is what actually protects against confusion there}
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
        "struct" => Struct,
        "await" => Await,
        "fetch" => Fetch,
        "route" => Route,
        "body" => Body,
        "GET" => Get,
        "POST" => Post,
        "PUT" => Put,
        "DELETE" => Delete,
        "PATCH" => Patch,
        "store" => Store,
        "extern" => Extern,
        "client" => Client,
        "server" => Server,
        "global" => Global,
        "module" => Module,
        "npm" => Npm,
        "as" => As,
        "test" => Test,
        "assert" => Assert,
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
        Span::new(FileId(0), 0, 0, 1,1)
    }

    #[test]
    fn keyword_lookup_covers_every_grammar_9_keyword(){



        let  keyword  = ["page", "component","layout","state", "fn", "let","return",
                        "if" , "else", "for", "while","in", "import", "enum", "uses",
                        "slot", "on","struct","await","fetch","route","body",
                        "GET","POST","PUT","DELETE","PATCH",
                        "row", "column", "stack", "container", "card",
                        "grid",  "text", "image", "icon", "input","textarea",
                        "button","checkbox","switch","radio","dropdown","table",
                        "navigation","tabs","dialog","modal","menu","and","or",
                        "not", "Int", "Float","String", "Bool", "Color","Size",
                        "Void","List","Map","Option", ];
        
        
       
        for kw in keyword {
            assert!(keyword_lookup(kw).is_some(), "expected {kw:?} to be a recognized keyword")
        }
    }
    #[test]
    fn builtin_function_names_are_not_reserved_keywords(){

            //We want to confirm if parseInt/awaitAll must stay ordinary identifiers at the
            //lexer level  - they are resolved as builtins later, in p-sema
            // specifically to user code COULD shadow them with a real
            // declaration without a lex-time collision
            assert_eq!(keyword_lookup("parseInt"), None);
            assert_eq!(keyword_lookup("awaitAll"), None);
    }

    #[test]
    fn non_keyword_identifier_is_not_reclassified(){
        assert_eq!(keyword_lookup("username"),None);
        assert_eq!(keyword_lookup("LoginForm"), None);
    }

    #[test]
    fn keyword_lookup_is_case_sensitive_for_type_vs_node_names(){
        assert_eq!(keyword_lookup("List"),Some(TokenKind::ListType));
        assert_eq!(keyword_lookup("list"),Some(TokenKind::List));
    }

    #[test]
    fn token_kind_display_matches_source_spelling(){
        assert_eq!(TokenKind::Page.to_string(),"page");
        assert_eq!(TokenKind::Arrow.to_string(),"->");
        assert_eq!(TokenKind::Store.to_string(), "store");
        assert_eq!(TokenKind::Extern.to_string(),"extern");
        assert_eq!(TokenKind::Size(16.0,SizeUnit::Px).to_string(),"16px");
    }

    #[test]
    fn token_carries_its_span(){
        let tok = Token::new(TokenKind::Ident("count".to_string()),dummy_span());
        assert_eq!(tok.span.line,1);
    }

    #[test]
    fn span_to_merges_correctly(){
        let a = Span::new(FileId(0),0,3,1,1);
        let b = Span::new(FileId(0), 10, 15, 1,11);
        let merged = a.to(b);
        assert_eq!(merged.start,0);
        assert_eq!(merged.end,15);
    }

    #[test]
    fn phase2_tier1_and_tier2_keywords_are_all_present(){

        let keywords = ["store", "extern", "client","server","global","module","npm", "as","test","assert"];

        for kw in keywords{
            assert!(keyword_lookup(kw).is_some(), "expected {kw:?} to be a keyword");
        }
    }
}