use indexmap::IndexSet;
use rustc_hash::FxHashMap;
use misc_utils::{
    Key,
    define_keys,
};
use std::rc::Rc;
use crate::misc::*;


pub type FunctionStore = IndexedItemStore<FnId, Function>;
pub type ExprStore = IndexedItemStore<ExprId, Expr>;
pub type IdentMap<T> = FxHashMap<Ident, T>;


#[derive(Debug)]
pub enum Expr {
    Begin(Vec<ExprId>),

    /// Define a var in the given scope
    DefVar(Ident, ExprId),
    /// Set a variable with the given expr's data
    SetVar(Ident, ExprId),
    /// Get the data in a variable
    GetVar(Ident),

    Cond {
        branches: Vec<CondBranch>,
        default: Option<ExprId>,
    },

    /// Get the function with the given id
    Function(FnId),

    /// Create a closure with the given id
    Closure(FnId),

    /// Call the data with the args
    Call(ExprId, Vec<ExprId>),

    /// Call a method with the args. This is always generated on the first pass over the AST so
    /// we don't have special-cases all over the parser.
    Method(ExprId, Ident, Vec<ExprId>),

    /// Get the field through the path
    GetPath(Vec<Ident>),
    /// Set the field through the path
    SetPath {
        path: Vec<Ident>,
        data: ExprId,
    },

    // Primitives
    String(Rc<String>),
    Number(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Keyword(Ident),
    None,
}


define_keys!(FnId, ExprId, Ident);

#[derive(Debug, Default)]
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
    pub name: Ident,
    pub captures: Vec<Ident>,
    pub params: Vec<Ident>,
    pub block: ExprId,
}
