use std::{fmt};

use crate::token::{FileId, SizeUnit, Span, Token, TokenKind, keyword_lookup};

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
    IndentNotMultipleOfUnit { line : u32, col : u32, unit : usize},
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
pub fn tokenize(source : &str, file : FileId) ->Result<Vec<Token>,LexError> {
    let mut lexer = Lexer::new(source, file);
    lexer.run()?;
    Ok(lexer.tokens)

}

struct Lexer<'a> {
    #[allow(dead_code)]
    source: &'a str,
    chars: Vec<(usize,char)>,
    idx : usize,
    line:u32,
    col: u32,
    paren_depth : i32,
    indent_stack : Vec<usize>,
    indent_unit : Option<usize>,
    tokens : Vec<Token>,
    file : FileId,
}

impl <'a> Lexer<'a> {
    fn new (source: &'a str, file : FileId)-> Self {
        Self { source, chars: source.char_indices().collect(), idx: 0, line: 1, col: 1, paren_depth: 0, indent_stack: vec![0], indent_unit: None, tokens: Vec::new(), file }
    }

   
    


    fn peek(&self)->Option<char>{
        self.chars.get(self.idx).map(|&(_,c)| c)
        
    }
    fn peek_at(&self, offset:usize)->Option<char>{
        self.chars.get(self.idx + offset).map(|&(_,c)| c)

    }

    fn byte_pos(&self)-> usize {
        self.chars.get(self.idx).map(|&(b,_)|b).unwrap_or(self.source.len())
    }

    fn here(&self) -> Span {
        let b = self.byte_pos();
        Span { start: b, end: b , line: self.line, col: self.col, file: self.file }

    // Advances one character, updating line/col,  Returns the consumed char
    }

    fn bump(&mut self)-> Option<char> {
        let c = self.peek()?;
        self.idx += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col +=1;
        }
        Some(c)
    }

    fn run(&mut self) -> Result<(), LexError>{
        let spaces = self.skip_blank_and_comment_lines()?;
        if self.peek().is_some(){
            self.apply_identation(spaces)?;
        }
        let mut had_content = false;
        loop {
            match self.peek(){
                None => break,
                Some('\n') => {
                    self.bump();
                    if had_content{
                        self.tokens.push(Token::new(TokenKind::Newline, self.here()));
                    }
                    had_content = false;
                    if self.paren_depth == 0 {
                        let spaces = self.skip_blank_and_comment_lines()?;
                        if self.peek().is_some(){
                            self.apply_identation(spaces)?;
                        }
                    }
                }
                Some('\t') => {return  Err(LexError::TabCharacter { line: self.line, col: self.col });}
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    self.skip_block_comment()?;
                }
                Some('"') => {
                    self.scan_string_or_color()?;
                    had_content = true;
                }
                Some(c) if c.is_ascii_digit() => {
                    self.scan_number()?;
                    had_content = true;
                    
                }
                Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                    self.scan_indent_or_keyword();
                    had_content = true;
                }
                Some(_) => {
                    self.scan_operator_or_punct()?;
                    had_content = true
                }
            }
        }
        if had_content {
            self.tokens.push(Token::new(TokenKind::Newline, self.here()));
        }
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.tokens.push(Token::new(TokenKind::Dedent,self.here()));
        }
        self.tokens.push(Token::new(TokenKind::Eof, self.here()));
        Ok(())
    
    }

   //Consumes any run of fully-blank lines and comment-only lines,
      //returning the leading space count of the first line that has 
       //real content (or 0 if EOF is reached first)
    fn skip_blank_and_comment_lines(&mut self) -> Result<usize, LexError>{
        loop
        {
            let mut spaces = 0usize;
            loop {
                match self.peek(){
                    Some(' ') => {
                        spaces += 1;
                         self.bump();
                    }
                    Some('\t') => {

                        return Err(LexError::TabCharacter { line: self.line, col: self.col });
                    }
                    _ => break,
                }
            }
                match self.peek() {
                    None => return Ok(0),

                    Some('\n') => {
                        self.bump();
                        continue;
                    }
                    Some('/') if self.peek_at(1) == Some('/') => {
                        self.skip_line_comment();
                        continue;
                        
                    }
                    Some('/') if self.peek_at(1) == Some('/') =>{
                        self.skip_line_comment();
                        continue;
                        
                    }
                    Some('/') if self.peek_at(1) == Some('*') => {
                        self.skip_block_comment()?;
                        continue;
                    }
                    _ => return Ok(spaces),
                }
            }
        }
    
    fn skip_line_comment(&mut self){
        while let Some(c) = self.peek(){
            if c == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let (start_line, start_col) = (self.line, self.col);
        self.bump();  // '/'
        self.bump(); // '*'
        loop {
            match self.peek() {
                None => {
                    return Err(LexError::UnterminatedBlockComment { line: start_line, col: start_col })
                }
                Some('*') if self.peek_at(1) == Some('/') => {
                    self.bump();
                    self.bump();
                    return Ok(());
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    // Compares 'spaces' to the current indent stack and emits
    // Indent / Dedent tokens
    fn apply_identation(&mut self, spaces : usize) -> Result<(),LexError>{
        
        let current = *self.indent_stack.last().unwrap();
        let delta = spaces.saturating_sub(current);
        println!("INDENT: line={} col = {} spaces = {} current {} delta {} unit = {:?}", self.line, self.col, spaces, current, delta, self.indent_unit);


        if spaces > current {
            
            match self.indent_unit{
                Some(unit) => {
                    if delta % unit != 0 {
                        return Err(LexError::IndentNotMultipleOfUnit { line: self.line, col: self.col , unit});
                    }
                }
                None => { 
                    println!("SETTING INDENT UNIT = {}", delta);
                    self.indent_unit = Some(delta);}
            }
            self.indent_stack.push(spaces);
            self.tokens.push(Token::new(TokenKind::Indent, self.here()));
        }       else if spaces < current {
                     while *self.indent_stack.last().unwrap() > spaces {
                    self.indent_stack.pop();
                    self.tokens.push(Token::new(crate::TokenKind::Dedent, self.here()));
            }
                if *self.indent_stack.last().unwrap() != spaces {
                return Err(LexError::InconsistentDedent { line: self.line , col: self.col  })
            }
        }
        Ok(())
        
    }

    fn scan_string_or_color(&mut self)-> Result<(), LexError>{
        let start = self.here();
        self.bump(); // opening quote

        let mut content = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(LexError::UnterminatedString { line: start.line, col: start.col })
                }
                Some('"') => {
                    self.bump();
                    break;
                }
                Some('\\') => {
                    self.bump();
                    let (l,c) = (self.line,self.col);
                    match self.bump() {
                        Some('"') => content.push('"'),
                        Some('\\') => content.push('\\'),
                        Some('\n') => content.push('\n'),
                        Some('\t') => content.push('\t'),
                        Some(other) => {
                          return  Err(LexError::InvalidEscape { line: l, col: c, ch: other })
                        }
                        None => {
                            return Err(LexError::UnterminatedString { line: start.line, col: start.col })
                        }
                    }
                }
                 Some(c) => {
                 content.push(c);
                  self.bump();
            }
            }
        }
        let kind = if is_hex_color(&content){
            TokenKind::Color(content.to_lowercase())
        } else {
            TokenKind::Str(content)
        };
        self.tokens.push(Token::new(kind, start));
        Ok(())
    }
    fn scan_number(&mut self) -> Result<(), LexError> {
    let start = self.here();
    let mut text = String::new();

    // Consume the integer portion first.
    while let Some(c) = self.peek() {
        if c.is_ascii_digit() {
            text.push(c);
            self.bump();
        } else {
            break;
        }
    }

    // Optional fractional portion.
    let mut is_float = false;

    if self.peek() == Some('_')
        && self.peek_at(1).map_or(false, |c| c.is_ascii_digit())
    {
        is_float = true;
        text.push('.');
        self.bump();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
    }

    // Check for an immediately-following size unit,
    // longest match first.
    const UNITS: &[(&str, SizeUnit)] = &[
        ("px", SizeUnit::Px),
        ("rem", SizeUnit::Rem),
        ("em", SizeUnit::Em),
        ("vw", SizeUnit::Vm),
        ("vh", SizeUnit::Vh),
    ];

    for (suffix, unit) in UNITS {
        if self.matches_ahead(suffix) {
            for _ in 0..suffix.len() {
                self.bump();
            }

            let value: f64 = text.parse().map_err(|_| {
                LexError::InvalidNumber {
                    line: start.line,
                    col: start.col,
                }
            })?;

            self.tokens
                .push(Token::new(TokenKind::Size(value, *unit), start));

            return Ok(());
        }
    }

    // Percentage.
    if self.peek() == Some('%') {
        self.bump();

        let value: f64 = text.parse().map_err(|_| {
            LexError::InvalidNumber {
                line: start.line,
                col: start.col,
            }
        })?;

        self.tokens.push(Token::new(
            TokenKind::Size(value, SizeUnit::Percent),
            start,
        ));

        return Ok(());
    }

    // Plain number.
    if is_float {
        let value: f64 = text.parse().map_err(|_| {
            LexError::InvalidNumber {
                line: start.line,
                col: start.col,
            }
        })?;

        self.tokens
            .push(Token::new(TokenKind::Float(value), start));
    } else {
        let value: i64 = text.parse().map_err(|_| {
            LexError::InvalidNumber {
                line: start.line,
                col: start.col,
            }
        })?;

        self.tokens
            .push(Token::new(TokenKind::Int(value), start));
    }

    Ok(())
}
    
    
    fn matches_ahead(&self, s: &str)->bool {
        let mut offset = 0;
        for expected in s.chars(){
            match self.peek_at(offset) {
                Some(c) if c == expected => offset +=1,
                _=> return false ,
            }
        }

        //Dont match a prefic of a longer identifier , e.g "px" inside "pixels".
        !matches!(self.peek_at(offset), Some(c) if c.is_ascii_alphanumeric())
    }
    // it scans 
    fn scan_indent_or_keyword(&mut self){
        let start = self.here();
        let mut text = String::new();
        while  let  Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                text.push(c);
                self.bump();
            }
            else {
                break;
            }
        }
        let kind = match text.as_str(){
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _=> keyword_lookup(&text).unwrap_or(TokenKind::Ident(text)),
        };
        self.tokens.push(Token::new(kind,start));
            
    }
    

    fn scan_operator_or_punct(&mut self) -> Result<(), LexError>{
        let start = self.here();
        let c = self.bump().unwrap();
        let kind = match c {
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '(' => {
                self.paren_depth +=1;
                TokenKind::LParen
            }
            ')' => {
                self.paren_depth -=1;
                TokenKind::RParen
            }
            '[' => {
                self.paren_depth += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.paren_depth -= 1 ;
                TokenKind::RBracket
            }
            '{' => {
                self.paren_depth += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.paren_depth -= 1;
                TokenKind::RBrace
            },

            '+' => TokenKind::Plus,
            
            '*' => TokenKind::Star,
            
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '-' => {
                if self.peek() == Some('>'){
                    self.bump();
                    TokenKind::Arrow
                }else {
                    TokenKind::Minus
                }
            }
            '=' => {
                if self.peek() == Some('>') {
                    self.bump();
                    TokenKind::FatArrow
                } else if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::EqEq
                }
                else {
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::NotEq
                }
                else {
                    return Err(LexError::UnknownCharacter { line: start.line, col: start.col, ch: '!' });
                }
            }

            '<' => {
                if self.peek() == Some('=') {
                    self.bump();
                    TokenKind::LtEq
                }
                else {
                    TokenKind::Lt
                }
            }

            '>' => { 
                if self.peek() == Some('='){
                    self.bump();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }

            }
            other => {
                return Err(LexError::UnknownCharacter { line: start.line, col: start.col, ch: other, })
            }
        };
       
        self.tokens.push(Token::new(kind, start));
        
        Ok(())
    }

}
    


fn is_hex_color(s : &str) -> bool {
    let Some(rest) = s.strip_prefix('#') else { return false ;};
    matches!(rest.len(),3|4|6|8) && rest.chars().all(|c| c.is_ascii_hexdigit())
}

    


    

