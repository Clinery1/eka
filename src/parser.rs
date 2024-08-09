use parser_helper::{
    LogosTokenStream,
    LookaheadLexer,
    SimpleError,
    new_parser,
};
use anyhow::{
    Context,
    Result,
    bail,
};
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    ops::{
        Fn as FnTrait,
        Deref,
    },
    error::Error,
};
use crate::{
    lexer::*,
    ast::*,
};


new_parser!(pub struct MyParser<'a, 1, Token<'a>, LogosTokenStream<'a, Token<'a>>, ()>);
impl<'a> MyParser<'a> {
}
