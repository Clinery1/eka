use anyhow::Result;
use std::{
    fmt::Debug,
    rc::Rc,
};
use object::{
    ObjectBundle,
    CallReturn,
};
use crate::ast::{
    ExprStore,
    FunctionStore,
    Interner,
    Ident,
    FnId,
};


pub mod object;
pub mod builtins;


pub type NativeFn<Gc, O> = fn(Vec<Primitive<Gc, O>>, &mut Interner, &mut Gc)->Result<CallReturn<Gc, O>>;


pub trait GcTrait<O: ObjectBundle<Self>>: Sized + Debug {
    type DataRef: Clone + Debug;
    fn alloc<RO: Into<O>>(&mut self, data: RO)->Self::DataRef;
    fn trace(&mut self, ptr: Self::DataRef);
}

pub trait Interpreter<O: ObjectBundle<Self::Gc>> {
    type Gc: GcTrait<O>;

    fn init(&mut self, _: &mut Interner)->Result<()> {Ok(())}

    /// Compile the expressions. `compile` may be called multiple times with different amounts of
    /// expressions.
    fn compile(&mut self,
        _exprs: &ExprStore,
        _funcs: &FunctionStore,
        _interner: &mut Interner,
    )->Result<()> {Ok(())}

    /// Run the given expressions. Callers are required to ensure `compile` was called after any
    /// changes to `ExprStore` or `FunctionStore` were made and before this method is called.
    fn run(&mut self,
        exprs: &ExprStore,
        funcs: &FunctionStore,
        interner: &mut Interner,
    )->Result<Primitive<Self::Gc, O>>;
}


#[derive(Debug)]
pub enum Primitive<Gc: GcTrait<O>, O: ObjectBundle<Gc>> {
    Data(Gc::DataRef),
    String(Rc<String>),
    Number(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Keyword(Ident),
    NativeFn(NativeFn<Gc, O>),
    Fn(FnId),
    None,
}
impl<Gc: GcTrait<O>, O: ObjectBundle<Gc>> Clone for Primitive<Gc, O> {
    fn clone(&self)->Self {
        use Primitive::*;
        match self {
            Data(d)=>Data(d.clone()),
            String(s)=>String(s.clone()),
            Number(n)=>Number(*n),
            Float(f)=>Float(*f),
            Char(c)=>Char(*c),
            Bool(b)=>Bool(*b),
            Keyword(i)=>Keyword(*i),
            NativeFn(f)=>NativeFn(*f),
            Fn(i)=>Fn(*i),
            None=>None,
        }
    }
}
