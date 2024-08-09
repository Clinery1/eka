use parser_helper::Token as TokenTrait;
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    error::Error,
};
use logos::{
    Lexer,
    Logos,
};

pub use StartOrEnd::*;


#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip "[ \t\r\n]")]
#[logos(error = LexerError)]
pub enum Token<'a> {
    #[regex("[^\"';#0-9\\\\()\\[\\]{} \t\r\n][^\"';\\\\()\\[\\]{} \t\r\n]*")]
    Ident(&'a str),

    #[regex("[^\"';#0-9\\\\()\\[\\]{} \t\r\n][^\"';\\\\()\\[\\]{} \t\r\n]*/", parse_path)]
    Path(Vec<&'a str>),

    #[regex("[0-9][0-9_]*", parse_num)]
    #[regex("-[0-9_]+", parse_num_neg)]
    Number(i64),

    #[regex("[0-9]+\\.[0-9]+", parse_float)]
    #[regex("-[0-9]+\\.[0-9]+", parse_float_neg)]
    #[regex("[0-9]+\\.", invalid_float)] 
    Float(f64),

    #[regex("#[a-zA-Z]+")]
    HashLiteral(&'a str),

    #[token("\\space", |_|' ')]
    #[token("\\newline", |_|'\n')]
    #[token("\\tab", |_|'\n')]
    #[token("\\", parse_char)]
    Char(char),

    #[token("(", |_|Start)]
    #[token(")", |_|End)]
    Paren(StartOrEnd),

    #[token("[", |_|Start)]
    #[token("]", |_|End)]
    Square(StartOrEnd),
    
    #[token("{", |_|Start)]
    #[token("}", |_|End)]
    Curly(StartOrEnd),

    #[token("'")]
    Quote,

    /// TODO: implement a proper string parser
    #[regex("\"[^\"]*\"", |l|l.slice().to_string())]
    String(String),

    #[regex(";[^\n]*")]
    Comment(&'a str),

    EOF,
}
impl<'a> TokenTrait for Token<'a> {
    fn eof()->Self {Self::EOF}
}

#[derive(Debug, PartialEq, Clone)]
pub enum StartOrEnd {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    EmptyPathSegment,
    IntegerOverflow,
    FloatOverflow,
    UnexpectedEof,
    InvalidFloat,
    InvalidToken,
}
impl Default for LexerError {
    fn default()->Self {
        LexerError::InvalidToken
    }
}
impl Error for LexerError {}
impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        use LexerError::*;

        match self {
            EmptyPathSegment=>write!(f, "Empty segment in path"),
            IntegerOverflow=>write!(f, "Integer Overflow"),
            FloatOverflow=>write!(f, "Float overflow"),
            UnexpectedEof=>write!(f, "Unexpected EOF"),
            InvalidFloat=>write!(f, "Invalid Float"),
            InvalidToken=>write!(f, "Invalid Token"),
        }
    }
}


fn parse_path<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<Vec<&'a str>, LexerError> {
    let mut out = Vec::new();
    let len = lex.slice().len();
    out.push(&lex.slice()[..(len - 1)]);

    let mut err = false;

    let mut slice_start = 0;
    let mut count = 0;
    for c in lex.remainder().chars() {
        if slice_start == count {   // first character of the ident
            match c {   // [^';#0-9\\\\()\\[\\]{} \t\r\n]
                '/'=>err = true,
                ';'|
                    '\\'|
                    ' '|
                    '\t'|
                    '\r'|
                    '\n'|
                    '('|
                    ')'|
                    '['|
                    ']'|
                    '{'|
                    '}'|
                    '"'|
                    '\''|
                    '#'|
                    '0'..='9'=>break,
                _=>{},
            }
        } else {
            match c {   // [^';\\\\()\\[\\]{} \t\r\n]
                '/'=>{
                    out.push(&lex.remainder()[slice_start..count]);
                    slice_start = count + 1;
                },
                ';'|
                    '\\'|
                    ' '|
                    '\t'|
                    '\r'|
                    '\n'|
                    '('|
                    ')'|
                    '['|
                    ']'|
                    '{'|
                    '}'|
                    '"'|
                    '\''=>break,
                _=>{},
            }
        }
        count += c.len_utf8()
    }

    // check if the current path slice is empty. Will only eval to true when there are 2 slices and
    // the second is empty.
    err |= slice_start == count;

    out.push(&lex.remainder()[slice_start..count]);

    lex.bump(count);

    if err {
        return Err(LexerError::EmptyPathSegment);
    } else {
        return Ok(out);
    }
}

#[inline]
fn invalid_float<'a>(_: &mut Lexer<'a, Token<'a>>)->Result<f64, LexerError> {
    Err(LexerError::InvalidFloat)
}

#[inline]
fn parse_num<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<i64, LexerError> {
    parse_num_inner(lex.slice())
}

#[inline]
fn parse_num_neg<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<i64, LexerError> {
    parse_num_inner(&lex.slice()[1..]).map(|o|o * -1)
}

#[inline]
fn parse_num_inner(s: &str)->Result<i64, LexerError> {
    let mut acc = 0i64;

    for c in s.chars() {
        match c {
            '0'..='9'=>{
                if let Some(shifted) = acc.checked_mul(10) {
                    acc = shifted;
                } else {
                    return Err(LexerError::IntegerOverflow);
                }
                acc += ((c as u8) - b'0') as i64;
            },
            '_'=>{},
            _=>unreachable!(),
        }
    }

    return Ok(acc);
}

#[inline]
fn parse_float<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<f64, LexerError> {
    parse_float_inner(lex.slice())
}

#[inline]
fn parse_float_neg<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<f64, LexerError> {
    parse_float_inner(&lex.slice()[1..]).map(|o|o * -1.0)
}

#[inline]
fn parse_float_inner(s: &str)->Result<f64, LexerError> {
    lexical::parse(s).map_err(|_|LexerError::FloatOverflow)
}

fn parse_char<'a>(lex: &mut Lexer<'a, Token<'a>>)->Result<char, LexerError> {
    let c = lex.remainder().chars().next().ok_or(LexerError::UnexpectedEof)?;
    lex.bump(c.len_utf8());
    return Ok(c);
}
