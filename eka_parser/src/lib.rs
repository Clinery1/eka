use parser_helper::{
    LogosTokenStream,
    LookaheadLexer,
    SimpleError,
    Span,
    new_parser,
};
use logos::Logos;
use anyhow::{
    Context,
    Result,
    bail,
};
use std::rc::Rc;
use eka_core::ast::*;
use lexer::*;


pub mod lexer;


new_parser!(pub struct Parser<'a, 2, Token<'a>, LogosTokenStream<'a, Token<'a>>, ParserData>);
// public methods
impl<'a> Parser<'a> {
    pub fn new_from_source(source: &'a str)->Parser<'a> {
        Parser::new(
            Token::lexer(source),
            ParserData {
                interner: Interner::default(),
                exprs: ExprStore::default(),
                funcs: FunctionStore::default(),
            },
        )
    }

    pub fn finish(self)->ParserData {
        self.0.finish()
    }

    pub fn parse(&mut self)->Result<()> {
        while self.peek() != &Token::EOF {
            let id = self.parse_expr()?;
            self.user_data.exprs.add_root(id);
        }

        return Ok(());
    }

    pub fn parse_expr(&mut self)->Result<ExprId> {
        match self.peek() {
            Token::Paren(Start)=>match self.peek1() {
                Token::Ident(kw)=>match *kw {
                    "def"=>self.parse_def(),
                    "set"=>self.parse_set(),
                    "defn"=>self.parse_func(),
                    "begin"=>self.parse_begin(),
                    "cond"=>self.parse_cond(),
                    _=>self.parse_call(),
                },
                _=>self.parse_call(),
            },
            _=>self.parse_primitive(),
        }
    }

    pub fn parse_cond(&mut self)->Result<ExprId> {
        self.paren_start()?;
        self.match_ident("cond")?;

        let mut branches = Vec::new();
        let mut default = None;

        while !self.try_paren_end() {
            match self.peek() {
                Token::Keyword("default")=>{
                    self.take_token();
                    if default.is_some() {
                        bail!("Cannot have multiple default branches in a cond expression");
                    }
                    default = Some(self.parse_expr().context("Cond default branch")?);
                },
                _=>branches.push(self.parse_cond_branch()?),
            }
        }

        return Ok(self.expr(Expr::Cond {branches, default}));
    }

    fn parse_cond_branch(&mut self)->Result<CondBranch> {
        let condition = self.parse_expr().context("In cond branch (condition)")?;
        let branch = self.parse_expr().context("In cond branch (body)")?;

        return Ok(CondBranch {
            condition,
            body: branch,
        });
    }

    pub fn parse_begin(&mut self)->Result<ExprId> {
        self.paren_start()?;
        self.match_ident("begin")?;

        let mut body = Vec::new();

        while !self.try_paren_end() {
            body.push(self.parse_expr().context("In begin expression")?);
        }

        return Ok(self.expr(Expr::Begin(body)));
    }

    pub fn parse_def(&mut self)->Result<ExprId> {
        self.paren_start()?;
        self.match_ident("def")?;

        let name = self.ident()?;

        let expr = self.parse_expr()?;

        self.paren_end()?;

        return Ok(self.expr(Expr::DefVar(name, expr)));
    }

    pub fn parse_set(&mut self)->Result<ExprId> {
        self.paren_start()?;
        self.match_ident("set")?;

        // check if there is a path. If so, then branch to the helper function
        match self.peek() {
            Token::Path(_)=>return self.parse_set_path_branch(),
            _=>{},
        }

        let name = self.ident()?;

        let data = self.parse_expr()?;

        self.paren_end()?;

        return Ok(self.expr(Expr::SetVar(name, data)));
    }

    fn parse_set_path_branch(&mut self)->Result<ExprId> {
        let path = self.path()?;

        let data = self.parse_expr()?;

        self.paren_end()?;

        return Ok(self.expr(Expr::SetPath{path, data}));
    }

    pub fn parse_func(&mut self)->Result<ExprId> {
        self.paren_start()?;
        self.match_ident("defn")?;

        let name = self.ident().context("In function definition")?;
        let caps = self.parse_func_caps().context("In function definition")?;
        let params = self.parse_func_params().context("In function definition")?;
        
        let mut body = Vec::new();

        while !self.try_paren_end() {
            body.push(self.parse_expr().context("In function body")?);
        }

        let block = if body.len() == 1 {
            body[0]
        } else {
            self.expr(Expr::Begin(body))
        };

        let is_closure = !caps.is_empty();

        let function = self.func(Function {
            name,
            captures: caps,
            params,
            block,
        });
        let expr_func = if is_closure {
            self.expr(Expr::Closure(function))
        } else {
            self.expr(Expr::Function(function))
        };

        return Ok(self.expr(Expr::DefVar(name, expr_func)));
    }

    fn parse_func_caps(&mut self)->Result<Vec<Ident>> {
        let mut caps = Vec::new();
        if !self.try_match_token(Token::Squiggle(Start)) {
            return Ok(caps);
        }

        while !self.try_match_token(Token::Squiggle(End)) {
            caps.push(self.ident()?);
        }

        return Ok(caps);
    }

    fn parse_func_params(&mut self)->Result<Vec<Ident>> {
        let mut params = Vec::new();
        self.match_token(Token::Vector(Start), "Expected function params")?;

        while !self.try_match_token(Token::Vector(End)) {
            params.push(self.ident()?);
        }

        return Ok(params);
    }

    pub fn parse_call(&mut self)->Result<ExprId> {
        self.paren_start()?;
        let first = self.parse_expr()?;

        let mut others = Vec::new();

        while !self.try_paren_end() {
            others.push(self.parse_expr()?);
        }

        return Ok(self.expr(Expr::Call(first, others)));
    }

    pub fn parse_primitive(&mut self)->Result<ExprId> {
        match self.next() {
            Token::Ident(name)=>{
                let ident = self.intern(name);
                Ok(self.expr(Expr::GetVar(ident)))
            },
            Token::Keyword(name)=>{
                let ident = self.intern(name);
                Ok(self.expr(Expr::Keyword(ident)))
            },
            Token::Path(p)=>{
                let expr = Expr::GetPath(p.into_iter().map(|s|self.intern(s)).collect());
                Ok(self.expr(expr))
            },
            Token::Number(n)=>Ok(self.expr(Expr::Number(n))),
            Token::Float(n)=>Ok(self.expr(Expr::Float(n))),
            Token::String(s)=>Ok(self.expr(Expr::String(Rc::new(s)))),
            Token::Char(c)=>Ok(self.expr(Expr::Char(c))),
            Token::HashLiteral("#t")=>Ok(self.expr(Expr::Bool(true))),
            Token::HashLiteral("#f")=>Ok(self.expr(Expr::Bool(false))),
            Token::HashLiteral("#N")=>Ok(self.expr(Expr::None)),
            t=>bail!(self.error(format!("Expected primitive expression, but got `{:?}`", t))),
        }
    }
}
// private helpers
#[allow(unused)]
impl<'a> Parser<'a> {
    #[inline]
    fn func(&mut self, func: Function)->FnId {
        self.user_data.funcs.insert(func)
    }

    #[inline]
    fn expr(&mut self, expr: Expr)->ExprId {
        self.user_data.exprs.insert(expr)
    }

    #[inline]
    fn update_expr(&mut self, id: ExprId, expr: Expr) {
        self.user_data.exprs[id] = expr;
    }

    #[inline]
    fn match_token<M: Into<String>>(&mut self, tok: Token<'a>, msg: M)->Result<()> {
        self.0.match_token(tok, msg)?;
        return Ok(())
    }

    #[inline]
    fn try_match_token(&mut self, tok: Token<'a>)->bool {
        if self.peek() == &tok {
            self.take_token();
            return true;
        }

        return false;
    }

    #[inline]
    fn peek(&mut self)->&Token<'a> {
        self.lookahead(0)
    }

    fn peek1(&mut self)->&Token<'a> {
        self.lookahead(1)
    }
    
    #[inline]
    fn peek_span(&mut self)->Span {
        self.lookahead_span(0)
    }

    #[inline]
    fn next(&mut self)->Token<'a> {
        self.take_token()
    }

    #[inline]
    fn error(&mut self, msg: impl Into<String>)->SimpleError<String> {
        self.0.error(msg)
    }

    #[inline]
    fn intern(&mut self, s: &str)->Ident {
        self.user_data.interner.intern(s)
    }

    fn ident(&mut self)->Result<Ident> {
        match self.take_token() {
            Token::Ident(s)=>Ok(self.intern(s)),
            _=>bail!(self.error("Expected identifier")),
        }
    }

    fn path(&mut self)->Result<Vec<Ident>> {
        match self.take_token() {
            Token::Path(p)=>Ok(p.into_iter().map(|s|self.intern(s)).collect()),
            _=>bail!(self.error("Expected identifier")),
        }
    }

    fn match_ident(&mut self, to_match: &str)->Result<()> {
        match self.take_token() {
            Token::Ident(s)=>if s != to_match {
                bail!(self.error(format!("Expected identifier `{}`, but got `{}`", to_match, s)));
            } else {
                Ok(())
            },
            _=>bail!(self.error("Expected identifier")),
        }
    }

    fn paren_start(&mut self)->Result<()> {
        match self.take_token() {
            Token::Paren(Start)=>Ok(()),
            _=>bail!(self.error("Expected `(`")),
        }
    }

    fn paren_end(&mut self)->Result<()> {
        match self.take_token() {
            Token::Paren(End)=>Ok(()),
            _=>bail!(self.error("Expected `)`")),
        }
    }

    fn try_paren_end(&mut self)->bool {
        match self.peek() {
            Token::Paren(End)=>{
                self.take_token();
                true
            },
            _=>false,
        }
    }
}

#[derive(Debug)]
pub struct ParserData {
    pub interner: Interner,
    pub exprs: ExprStore,
    pub funcs: FunctionStore,
}
