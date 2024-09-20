use anyhow::{
    Result,
    anyhow,
    bail,
};
use misc_utils::Stack;
use std::mem;
use eka_core::{
    interpreter::{
        object::*,
        Primitive,
    },
    ast::*,
};
use data::*;


pub mod data;


pub struct Interpreter<O: ObjectBundle<Gc<O>>> {
    pub interner: Interner,
    gc: Gc<O>,
    global_scope: IdentMap<Primitive<Gc<O>, O>>,
    vars: Stack<IdentMap<Primitive<Gc<O>, O>>>,
}
impl<O: ObjectBundle<Gc<O>>> Interpreter<O> {
    pub fn new(interner: Interner)->Self {
        let mut i = Interpreter {
            interner,
            gc: Gc::new(),
            global_scope: IdentMap::default(),
            vars: Stack::new(),
        };

        use eka_core::interpreter::builtins;

        i.def_global_str("+", Primitive::NativeFn(builtins::add));
        i.def_global_str("-", Primitive::NativeFn(builtins::sub));
        i.def_global_str("*", Primitive::NativeFn(builtins::mul));
        i.def_global_str("/", Primitive::NativeFn(builtins::div));
        i.def_global_str("format", Primitive::NativeFn(builtins::format));

        return i;
    }

    pub fn run(&mut self, store: &ExprStore, funcs: &FunctionStore)->Result<Primitive<Gc<O>, O>> {
        let mut last = Primitive::None;
        for root in store.iter_roots() {
            last = self.run_expr(*root, store, funcs)?;
        }

        return Ok(last);
    }

    pub fn run_expr(&mut self, id: ExprId, store: &ExprStore, funcs: &FunctionStore)->Result<Primitive<Gc<O>, O>> {
        use Expr::*;
        match &store[id] {
            Begin(block)=>{
                let mut last = Primitive::None;
                self.vars.push(IdentMap::default());
                for id in block.iter() {
                    last = self.run_expr(*id, store, funcs)?;
                }
                self.vars.pop();

                Ok(last)
            },

            DefVar(name, expr)=>{
                let val = self.run_expr(*expr, store, funcs)?;
                self.def_var(*name, val);

                Ok(Primitive::None)
            },
            SetVar(name, expr)=>{
                let val = self.run_expr(*expr, store, funcs)?;
                self.set_var(*name, val)?;
                Ok(Primitive::None)
            },
            GetVar(name)=>return self.get_var(*name),

            Cond{branches,default}=>{
                todo!();
            },

            Function(id)=>Ok(Primitive::Fn(*id)),

            Closure(id)=>{
                todo!();
            },

            Call(lhs, raw_args)=>{
                match &store[*lhs] {
                    Expr::GetPath(p)=>return self.call_path(p, raw_args, store, funcs),
                    _=>{},
                }

                let lhs = self.run_expr(*lhs, store, funcs)?;
                let mut args = Vec::new();
                for arg in raw_args {
                    args.push(self.run_expr(*arg, store, funcs)?);
                }

                match lhs {
                    Primitive::Fn(id)=>return self.call_function(id, args, store, funcs),
                    Primitive::Data(mut d)=>{
                        let ret = d.call(args, &self.interner, &mut self.gc)?;
                        return self.object_return_thing(ret, store, funcs);
                    },
                    Primitive::NativeFn(f)=>{
                        let ret = f(args, &mut self.interner, &mut self.gc)?;
                        return self.object_return_thing(ret, store, funcs);
                    },
                    _=>bail!("Cannot call primitive type"),
                }
            },

            Method(_lhs, _name, _args)=>todo!(),

            GetPath(path)=>{
                let (lhs, name) = self.resolve_path(path)?;
                match lhs {
                    Primitive::Data(d)=>return d.get(name, &self.interner),
                    _=>bail!("Cannot get a field on primitive type"),
                }
            },
            SetPath{path,data}=>{
                let (lhs, name) = self.resolve_path(path)?;
                let val = self.run_expr(*data, store, funcs)?;
                match lhs {
                    Primitive::Data(mut d)=>return d.set(name, val, &self.interner).map(|_|Primitive::None),
                    _=>bail!("Cannot set a field on primitive type"),
                }
            },

            // Primitives
            String(s)=>Ok(Primitive::String(s.clone())),
            Number(i)=>Ok(Primitive::Number(*i)),
            Float(f)=>Ok(Primitive::Float(*f)),
            Char(c)=>Ok(Primitive::Char(*c)),
            Bool(b)=>Ok(Primitive::Bool(*b)),
            Keyword(i)=>Ok(Primitive::Keyword(*i)),
            None=>Ok(Primitive::None),
        }
    }

    pub fn alloc(&mut self, obj: O)->DataRef<O> {
        self.gc.alloc(obj)
    }

    #[inline(always)]
    fn call_path(&mut self, path: &[Ident], raw_args: &[ExprId], store: &ExprStore, funcs: &FunctionStore)->Result<Primitive<Gc<O>, O>> {
        let (lhs, name) = self.resolve_path(path)?;

        let mut args = Vec::new();
        for arg in raw_args {
            args.push(self.run_expr(*arg, store, funcs)?);
        }

        match lhs {
            Primitive::Data(mut d)=>{
                let ret = d.method(name, args, &self.interner, &mut self.gc)?;
                return self.object_return_thing(ret, store, funcs);
            },
            _=>bail!("Cannot call a method on primitive type"),
        }
    }

    fn object_return_thing(&mut self, ret: CallReturn<Gc<O>, O>, store: &ExprStore, funcs: &FunctionStore)->Result<Primitive<Gc<O>, O>> {
        match ret {
            CallReturn::CallFn(id, args)=>return self.call_function(id, args, store, funcs),
            CallReturn::Data(val)=>return Ok(val),
        }
    }

    fn call_function(&mut self, id: FnId, args: Vec<Primitive<Gc<O>, O>>, store: &ExprStore, funcs: &FunctionStore)->Result<Primitive<Gc<O>, O>> {
        let old_vars = mem::replace(&mut self.vars, Stack::new());
        self.vars.push(IdentMap::default());

        let function = &funcs[id];

        if function.params.len() != args.len() {
            bail!("Expected {} args, but got {}", function.params.len(), args.len());
        }

        for (param, data) in function.params.iter().zip(args) {
            self.def_var(*param, data);
        }

        self.vars.push(IdentMap::default());
        let ret = self.run_expr(function.block, store, funcs)?;

        self.vars = old_vars;

        return Ok(ret);
    }

    /// Resolves a path EXCEPT the last item, which it returns.
    fn resolve_path(&self, path: &[Ident])->Result<(Primitive<Gc<O>, O>, Ident)> {
        let mut path_iter = path.iter();
        let mut next_name = *path_iter.next().unwrap();
        let mut data = self.get_var(next_name)?;
        next_name = *path_iter.next().unwrap();

        loop {
            if let Some(name) = path_iter.next() {
                match data {
                    Primitive::Data(d)=>data = d.get(next_name, &self.interner)?,
                    _=>bail!("Cannot get field on primitive type"),
                }
                next_name = *name;
            } else {break}

        }

        return Ok((data, next_name));
    }

    pub fn get_var(&self, name: Ident)->Result<Primitive<Gc<O>, O>> {
        for scope in self.vars.iter() {
            if let Some(var) = scope.get(&name) {
                return Ok(var.clone());
            }
        }

        return self.global_scope.get(&name)
            .cloned()
            .ok_or(anyhow!("Var `{}` is undefined", self.interner.get(name)));
    }

    #[inline]
    pub fn def_global(&mut self, name: Ident, data: Primitive<Gc<O>, O>) {
        self.global_scope.insert(name, data);
    }

    #[inline]
    pub fn def_global_str(&mut self, name: &str, data: Primitive<Gc<O>, O>) {
        let name = self.interner.intern(name);
        self.global_scope.insert(name, data);
    }

    pub fn def_var(&mut self, name: Ident, data: Primitive<Gc<O>, O>) {
        // global scope
        if self.vars.len() == 0 {
            self.global_scope.insert(name, data);
            return;
        }

        self.vars.last_mut().unwrap().insert(name, data);
    }

    pub fn set_var(&mut self, name: Ident, data: Primitive<Gc<O>, O>)->Result<()> {
        // global scope
        if self.vars.len() == 0 {
            let var = self.global_scope.get_mut(&name)
                .ok_or(anyhow!("Cannot set undefined var `{}`", self.interner.get(name)))?;
            *var = data;

            return Ok(());
        }

        if let Some(var) = self.vars.last_mut().unwrap().get_mut(&name) {
            *var = data;
            return Ok(());
        } else {
            bail!("Cannot set undefined var `{}`", self.interner.get(name));
        }
    }
}

#[derive(Debug)]
pub struct Closure<O: ObjectBundle<Gc<O>>> {
    id: FnId,
    items: IdentMap<Primitive<Gc<O>, O>>,
}
