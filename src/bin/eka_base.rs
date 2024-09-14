use logos::Logos;
use anyhow::{
    Result,
    bail,
};
use std::{
    io::{
        Stdin,
        Stdout,
        Stderr,
        Write,
        stdin,
        stdout,
        stderr,
    },
    time::{
        Instant,
        Duration,
    },
    fs::read_to_string,
};
use eka::{
    interpreter::{
        object::*,
        data::*,
        tree_walk,
        Primitive,
        GcTracer,
    },
    ast::{
        Ident,
        Interner,
    },
    lexer,
};


eka::bundle_object_types! {
    bundle EkaBaseBundle {
        BaseObject: BaseObject<Self>,
        GcWorkload: GcWorkloadObject<Self>,
        Console: Console,
        Duration: DurationObject,
        Instant: InstantObject,
    }
}


#[derive(Debug)]
pub struct InstantObject(Instant);
impl Object for InstantObject {
    type ObjectBundle = EkaBaseBundle;

    fn get(&self, _: Ident, _: &Interner)->Result<Primitive<EkaBaseBundle>> {
        bail!("There are no fields on Instant");
    }
    fn set(&mut self, _: Ident, _: Primitive<EkaBaseBundle>, _: &Interner)->Result<()> {
        bail!("There are no fields on Instant");
    }

    fn call(&mut self, _: Vec<Primitive<EkaBaseBundle>>, _: &Interner, gc: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        let duration = self.0.elapsed();
        let dr = gc.alloc(DurationObject(duration).into());

        return Ok(CallReturn::Data(Primitive::Data(dr)));
    }
    fn method(&mut self, _: Ident, _: Vec<Primitive<EkaBaseBundle>>, _: &Interner, _: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        bail!("Instant has no methods");
    }

    fn trace<T: GcTracer<EkaBaseBundle>>(&self, _: &mut T) {}
}

#[derive(Debug)]
pub struct DurationObject(Duration);
impl Object for DurationObject {
    type ObjectBundle = EkaBaseBundle;

    fn get(&self, _: Ident, _: &Interner)->Result<Primitive<EkaBaseBundle>> {
        bail!("There are no fields on Duration");
    }
    fn set(&mut self, _: Ident, _: Primitive<EkaBaseBundle>, _: &Interner)->Result<()> {
        bail!("There are no fields on Duration");
    }

    fn call(&mut self, _: Vec<Primitive<EkaBaseBundle>>, _: &Interner, _: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        Ok(CallReturn::Data(Primitive::String(format!("{:?}", self.0).into())))
    }
    fn method(&mut self, _: Ident, _: Vec<Primitive<EkaBaseBundle>>, _: &Interner, _: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        bail!("Duration has no methods");
    }

    fn trace<T: GcTracer<EkaBaseBundle>>(&self, _: &mut T) {}
}

#[derive(Debug)]
pub struct Console {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,

    read_line_ident: Ident,
    print_ident: Ident,
    eprint_ident: Ident,
}
impl Console {
    pub fn new(i: &mut Interner)->Self {
        Console {
            stdin: stdin(),
            stdout: stdout(),
            stderr: stderr(),

            read_line_ident: i.intern("readLine"),
            print_ident: i.intern("print"),
            eprint_ident: i.intern("eprint"),
        }
    }
}
impl Object for Console {
    type ObjectBundle = EkaBaseBundle;

    fn get(&self, _: Ident, _: &Interner)->Result<Primitive<EkaBaseBundle>> {
        bail!("There are no fields on Console");
    }
    fn set(&mut self, _: Ident, _: Primitive<EkaBaseBundle>, _: &Interner)->Result<()> {
        bail!("There are no fields on Console");
    }

    fn call(&mut self, _: Vec<Primitive<EkaBaseBundle>>, _: &Interner, _: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        bail!("Cannot call Console");
    }
    fn method(&mut self, name: Ident, args: Vec<Primitive<EkaBaseBundle>>, _: &Interner, _: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
        if name == self.read_line_ident {
            if args.len() != 0 {
                bail!("Expected zero arguments to Console.read_line");
            }

            let mut s = String::new();
            self.stdin.read_line(&mut s)?;

            return Ok(CallReturn::Data(Primitive::String(s.into())));
        }
        if name == self.print_ident {
            if args.len() != 1 {
                bail!("Expected one argument to Console.print");
            }
            match &args[0] {
                Primitive::String(s)=>{
                    write!(&mut self.stdout, "{}", s)?;
                    return Ok(CallReturn::Data(Primitive::Number(s.len() as i64)));
                },
                _=>bail!("Can only write strings via Console.print"),
            }
        }
        if name == self.eprint_ident {
            if args.len() != 1 {
                bail!("Expected one argument to Console.eprint");
            }
            match &args[0] {
                Primitive::String(s)=>{
                    write!(&mut self.stderr, "{}", s)?;
                    return Ok(CallReturn::Data(Primitive::Number(s.len() as i64)));
                },
                _=>bail!("Can only write strings via Console.eprint"),
            }
        }

        bail!("No method with the given name");
    }

    fn trace<T: GcTracer<EkaBaseBundle>>(&self, _: &mut T) {}
}


fn main() {
    let source = read_to_string("example.eka").unwrap();

    for tok in lexer::Token::lexer(&source) {
        dbg!(tok).ok();
    }

    let mut parser = eka::parser::Parser::new_from_source(&source);
    parser.parse().unwrap();

    let mut data = parser.finish();
    dbg!(&data);
    let console = Console::new(&mut data.interner);
    let gc_workload = GcWorkloadObject::<EkaBaseBundle>::new(&mut data.interner);

    let mut interpreter = tree_walk::Interpreter::<EkaBaseBundle>::new(data.interner);

    let console_dr = interpreter.alloc(console.into());
    interpreter.def_global_str("console", Primitive::Data(console_dr));

    let gc_workload_dr = interpreter.alloc(gc_workload.into());
    interpreter.def_global_str("gcWorkload", Primitive::Data(gc_workload_dr));

    interpreter.def_global_str("instantNow", Primitive::NativeFn(instant_now));

    dbg!(interpreter.run(&data.exprs, &data.funcs).unwrap());
}

fn instant_now(args: Vec<Primitive<EkaBaseBundle>>, _: &mut Interner, gc: &mut Gc<EkaBaseBundle>)->Result<CallReturn<EkaBaseBundle>> {
    if args.len() != 0 {
        bail!("Expected zero args for instantNow");
    }

    let dr = gc.alloc(InstantObject(Instant::now()).into());

    return Ok(CallReturn::Data(Primitive::Data(dr)))
}
