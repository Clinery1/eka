use anyhow::{
    Result,
    bail,
    anyhow,
};
use std::fmt::Debug;
use crate::ast::{
    Interner,
    IdentMap,
    Ident,
    FnId,
};
use super::{
    data::Gc,
    Primitive,
    GcTracer,
};


/// A macro to bundle multiple types implementing the [`Object`] trait into one enum without having
/// to write a lot of code. It implements `From<ObjectType>` for each type. Example:
/// ```rust
/// bundle_object_types! {
///     bundle MyObjects {
///         // The variants can be called whatever you want, and generics can use `Self` to refer
///         // to the bundle's name. In this case it is `MyObjects`. The variant names are only
///         // required because there is no way to generate a unique, but reusable name from the
///         // `macro_rules` macro system
///         VariantName: ObjectType1<Self>,
///         OtherVariantName: ObjectType2<Self>,
///     }
/// }
/// ```
#[macro_export]
macro_rules! bundle_object_types {
    ($(bundle $name:ident { $($obj_name:ident : $obj:ty,)+})*)=>{
        $(
        #[derive(Debug)]
        pub enum $name {
            $(
                $obj_name($obj),
            )+
        }
        $(
            impl From<$obj> for $name {
                fn from(inner: $obj)->Self {
                    $name::$obj_name(inner)
                }
            }
        )+
        impl ObjectBundle for $name {}
        impl Object for $name {
            type ObjectBundle = Self;

            #[inline]
            fn get(&self,
                name: $crate::ast::Ident,
                interner: &$crate::ast::Interner,
            )->anyhow::Result<$crate::interpreter::Primitive<$name>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.get(name, interner),
                    )+
                }
            }
            #[inline]
            fn set(&mut self,
                name: $crate::ast::Ident,
                data: $crate::interpreter::Primitive<$name>,
                interner: &$crate::ast::Interner,
            )->anyhow::Result<()> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.set(name, data, interner),
                    )+
                }
            }

            #[inline]
            fn call(&mut self,
                args: Vec<$crate::interpreter::Primitive<$name>>,
                interner: &$crate::ast::Interner,
                gc: &mut $crate::interpreter::data::Gc<$name>,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.call(args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn method(&mut self,
                name: $crate::ast::Ident,
                args: Vec<$crate::interpreter::Primitive<$name>>,
                interner: &$crate::ast::Interner,
                gc: &mut $crate::interpreter::data::Gc<$name>,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.method(name, args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn trace<T: $crate::interpreter::GcTracer<$name>>(&self, tracer: &mut T) {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.trace(tracer),
                    )+
                }
            }
        }
        )*
    };
}


pub trait Object: Sized {
    type ObjectBundle: ObjectBundle + Debug;

    /// If an object can be recycled. Default is yes.
    ///
    /// It is generally better to recycle objects
    #[inline]
    fn can_recycle(&self)->bool {
        true
    }

    /// A method to allow for extra things to be implemented on object recycle. Defaults to just
    /// dropping `self` and overwriting it with `new`.
    ///
    /// This only runs if [`Object::can_recycle`] returns true. Otherwise `Self` gets dropped like
    /// normal.
    #[inline]
    fn recycle_insert(&mut self, new: Self) {
        *self = new;
    }

    /// A method that is **always** called when the object is determined to be dead.
    /// This will always be called before calling drop, but dropping may happen at any time from
    /// after this call to the end of the universe.
    ///
    /// If you can't cleanup the resources without dropping, then implement [`Object::can_recycle`]
    /// and return `false` all the time. This will ensure `Self` is dropped when it becomes dead
    /// instead of being recycled.
    #[inline]
    fn finalize(&mut self) {}

    fn get(&self, name: Ident, interner: &Interner)->Result<Primitive<Self::ObjectBundle>>;
    fn set(&mut self, name: Ident, data: Primitive<Self::ObjectBundle>, interner: &Interner)->Result<()>;

    fn call(&mut self,
        args: Vec<Primitive<Self::ObjectBundle>>,
        interner: &Interner,
        gc: &mut Gc<Self::ObjectBundle>,
    )->Result<CallReturn<Self::ObjectBundle>>;
    fn method(&mut self,
        name: Ident,
        args: Vec<Primitive<Self::ObjectBundle>>,
        interner: &Interner,
        gc: &mut Gc<Self::ObjectBundle>,
    )->Result<CallReturn<Self::ObjectBundle>>;

    fn trace<T: GcTracer<Self::ObjectBundle>>(&self, tracer: &mut T);
}

pub trait ObjectBundle: Sized + Object<ObjectBundle = Self> + Debug {
}


pub enum CallReturn<O: ObjectBundle> {
    CallFn(FnId, Vec<Primitive<O>>),
    Data(Primitive<O>),
}


#[derive(Debug)]
pub struct BaseObject<O: ObjectBundle>(IdentMap<Primitive<O>>);
impl<O: ObjectBundle> Object for BaseObject<O> {
    type ObjectBundle = O;

    fn get(&self, name: Ident, _: &Interner)->Result<Primitive<O>> {
        self.0.get(&name)
            .cloned()
            .ok_or(anyhow!("Does not contain the field"))
    }
    fn set(&mut self, name: Ident, data: Primitive<O>, _: &Interner)->Result<()> {
        self.0.insert(name, data);
        Ok(())
    }

    fn call(&mut self, _: Vec<Primitive<O>>, _: &Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
        bail!("Cannot call BaseObject");
    }
    fn method(&mut self, _: Ident, _: Vec<Primitive<O>>, _: &Interner, _: &mut Gc<O>)->Result<CallReturn<O>> {
        bail!("BaseObject has no methods");
    }

    fn trace<T: GcTracer<O>>(&self, tracer: &mut T) {
        for val in self.0.values() {
            match val {
                Primitive::Data(d)=>tracer.trace(d.clone()),
                _=>{},
            }
        }
    }
}
