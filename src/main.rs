use logos::Logos;
use std::{
    fs::read_to_string,
};


mod misc;
mod lexer;
mod parser;
mod ast;
mod interpreter;


fn main() {
    let source = read_to_string("example.eka").unwrap();

    for tok in lexer::Token::lexer(&source) {
        dbg!(tok).ok();
    }
}
