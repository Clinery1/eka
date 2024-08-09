use indexmap::IndexSet;
use misc_utils::{
    Key,
    define_keys,
};
use std::rc::Rc;
use crate::misc::*;


pub type FunctionStore = IndexedItemStore<FnId, Function>;
pub type ExprStore = IndexedItemStore<ExprId, Expr>;


#[derive(Debug)]
pub enum Expr {
    Begin(Vec<ExprId>),
    /// An optimized version of `Begin` where all vars are stored in a linear array that is
    /// directly indexed.
    BeginOpt {
        start_slot: usize,
        slots: IndexSet<Ident>,
        exprs: Vec<ExprId>,
    },

    /// Define a var in the given scope
    DefVar(Ident, ExprId),
    /// Set a variable with the given expr's data
    SetVar(Ident, ExprId),
    /// Get the data in a variable
    GetVar(Ident),

    DefVarOpt(VarSlot, ExprId),
    SetVarOpt(VarSlot, ExprId),
    GetVarOpt(VarSlot),

    Cond {
        branches: Vec<CondBranch>,
        default: Option<ExprId>,
    },

    /// Get the function with the given id
    FunctionOpt(FnId),
    /// Get the function with the given id and bind the given captures to a closure object
    Closure {
        id: FnId,
        caps: Vec<VarSlot>,
    },

    /// The full function. Removed on the first pass and replaced with `FunctionOpt(FnId)` or
    /// `Closure(..)`
    Function(Function),

    /// Used to return a value from the current function before the function is over. When the
    /// function is called again, it will start at the expression after this one.
    /// NOTE: This DOES NOT mutate the global function pointer, but only the copy of the pointer
    /// that was called.
    Yield(ExprId),

    /// Call the data with the args
    Call(ExprId, Vec<ExprId>),

    /// Call a method with the args. This is always generated on the first pass over the AST so
    /// we don't have special-cases all over the parser.
    Method(ExprId, Ident, Vec<ExprId>),

    /// Get the field on a given object
    GetField {
        name: Ident,
        data: ExprId,
    },
    /// Set the field on a given object
    SetField {
        obj: ExprId,
        name: Ident,
        data: ExprId,
    },

    String(Rc<String>),
    Number(i64),
    Float(f64),
    Char(char),
    Bool(bool),
}


define_keys!(FnId, ExprId, Ident);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarSlot(pub usize);

#[derive(Debug)]
pub struct Interner(IndexSet<String>);
impl Interner {
    pub fn intern(&mut self, s: &str)->Ident {
        if let Some(idx) = self.0.get_index_of(s) {
            return Ident::from_id(idx);
        }

        let (idx, _) = self.0.insert_full(s.to_string());
        return Ident::from_id(idx);
    }

    #[inline]
    pub fn get(&self, id: Ident)->&str {
        self.0.get_index(id.0).unwrap()
    }
}

#[derive(Debug)]
pub struct CondBranch {
    pub condition: ExprId,
    pub body: ExprId,
}

#[derive(Debug)]
pub struct Function {
    /// Populated in the first pass
    yields: Vec<ExprId>,

    captures: Vec<Ident>,

    params: Vec<Ident>,

    exprs: Vec<ExprId>,
}
