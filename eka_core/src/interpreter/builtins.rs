use anyhow::{
    Result,
    bail,
};
use std::{
    fmt::Write,
    rc::Rc,
};
use super::{
    object::{
        ObjectBundle,
        CallReturn,
    },
    Primitive,
    GcTrait,
};
use crate::ast::Interner;


pub fn add<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(args: Vec<Primitive<Gc, O>>, _: &mut Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
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

pub fn sub<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(args: Vec<Primitive<Gc, O>>, _: &mut Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
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

pub fn mul<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(args: Vec<Primitive<Gc, O>>, _: &mut Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
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

pub fn div<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(args: Vec<Primitive<Gc, O>>, _: &mut Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
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

pub fn format<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(args: Vec<Primitive<Gc, O>>, interner: &mut Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
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
