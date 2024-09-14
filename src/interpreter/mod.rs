use anyhow::{
    Result,
    bail,
};
use std::{
    rc::Rc,
    fmt::Write,
};
use data::{
    DataRef,
    Gc,
};
use object::{
    ObjectBundle,
    CallReturn,
};
use crate::ast::{
    Interner,
    Ident,
    FnId,
};


pub mod object;
pub mod data;
pub mod tree_walk;


pub type NativeFn<O> = fn(Vec<Primitive<O>>, &mut Interner, &mut Gc<O>)->Result<CallReturn<O>>;


pub trait GcTracer<O: ObjectBundle> {
    fn trace(&mut self, ptr: DataRef<O>);
}


#[derive(Debug)]
pub enum Primitive<O: ObjectBundle> {
    Data(DataRef<O>),
    String(Rc<String>),
    Number(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Keyword(Ident),
    NativeFn(NativeFn<O>),
    Fn(FnId),
    None,
}
impl<O: ObjectBundle> Clone for Primitive<O> {
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

fn add<O: ObjectBundle>(args: Vec<Primitive<O>>, _: &mut Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
    if args.len() == 0 {return Ok(CallReturn::Data(Primitive::None))}
    let mut args_iter = args.into_iter();
    let mut first = args_iter.next().unwrap();
    for arg in args_iter {
        use Primitive::*;
        match (&mut first, arg) {
            (Number(n), Number(n2))=>*n += n2,
            (Float(f), Float(f2))=>*f += f2,
            _=>bail!("Only numbers and floats can be added"),
        }
    }

    return Ok(CallReturn::Data(first));
}

fn sub<O: ObjectBundle>(args: Vec<Primitive<O>>, _: &mut Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
    if args.len() == 0 {return Ok(CallReturn::Data(Primitive::None))}
    let mut args_iter = args.into_iter();
    let mut first = args_iter.next().unwrap();
    for arg in args_iter {
        use Primitive::*;
        match (&mut first, arg) {
            (Number(n), Number(n2))=>*n -= n2,
            (Float(f), Float(f2))=>*f -= f2,
            _=>bail!("Only numbers and floats can be subtracted"),
        }
    }

    return Ok(CallReturn::Data(first));
}

fn mul<O: ObjectBundle>(args: Vec<Primitive<O>>, _: &mut Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
    if args.len() == 0 {return Ok(CallReturn::Data(Primitive::None))}
    let mut args_iter = args.into_iter();
    let mut first = args_iter.next().unwrap();
    for arg in args_iter {
        use Primitive::*;
        match (&mut first, arg) {
            (Number(n), Number(n2))=>*n *= n2,
            (Float(f), Float(f2))=>*f *= f2,
            _=>bail!("Only numbers and floats can be multiplied"),
        }
    }

    return Ok(CallReturn::Data(first));
}

fn div<O: ObjectBundle>(args: Vec<Primitive<O>>, _: &mut Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
    if args.len() == 0 {return Ok(CallReturn::Data(Primitive::None))}
    let mut args_iter = args.into_iter();
    let mut first = args_iter.next().unwrap();
    for arg in args_iter {
        use Primitive::*;
        match (&mut first, arg) {
            (Number(n), Number(n2))=>*n /= n2,
            (Float(f), Float(f2))=>*f /= f2,
            _=>bail!("Only numbers and floats can be divided"),
        }
    }

    return Ok(CallReturn::Data(first));
}

fn format<O: ObjectBundle>(args: Vec<Primitive<O>>, interner: &mut Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
    let mut out = String::new();
    for arg in args {
        use Primitive::*;
        match arg {
            Data(d)=>write!(out, "{d:?}")?,
            String(s)=>out.push_str(&s),
            Char(c)=>out.push(c),
            Number(n)=>write!(out, "{n}")?,
            Float(f)=>write!(out, "{f}")?,
            Bool(b)=>write!(out, "{b}")?,
            Keyword(k)=>write!(out, "{}", interner.get(k))?,
            NativeFn(_)=>out.push_str("<NativeFn>"),
            Fn(id)=>write!(out, "<Fn#{id:?}>")?,
            None=>out.push_str("None"),
        }
    }

    return Ok(CallReturn::Data(Primitive::String(Rc::new(out))));
}
