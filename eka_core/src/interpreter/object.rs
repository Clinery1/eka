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
    Primitive,
    GcTrait,
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
    ($(bundle $name:ident where GC = $gc_ty:ty { $($obj_name:ident : $obj:ty,)+})*)=>{
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
        impl ObjectBundle<$gc_ty> for $name {}
        impl Object<$gc_ty> for $name {
            type ObjectBundle = Self;

            #[inline]
            fn get(&self,
                name: $crate::ast::Ident,
                interner: &$crate::ast::Interner,
            )->anyhow::Result<$crate::interpreter::Primitive<$gc_ty, $name>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.get(name, interner),
                    )+
                }
            }
            #[inline]
            fn set(&mut self,
                name: $crate::ast::Ident,
                data: $crate::interpreter::Primitive<$gc_ty, $name>,
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
                args: Vec<$crate::interpreter::Primitive<$gc_ty, $name>>,
                interner: &$crate::ast::Interner,
                gc: &mut $gc_ty,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<$gc_ty, Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.call(args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn method(&mut self,
                name: $crate::ast::Ident,
                args: Vec<$crate::interpreter::Primitive<$gc_ty, $name>>,
                interner: &$crate::ast::Interner,
                gc: &mut $gc_ty,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<$gc_ty, Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.method(name, args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn trace(&self, tracer: &mut $gc_ty) {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.trace(tracer),
                    )+
                }
            }
        }
        )*
    };
    ($(bundle $name:ident<$gc_name:ident> { $($obj_name:ident : $obj:ty,)+})*)=>{
        $(
        #[derive(Debug)]
        pub enum $name<$gc_name: $crate::interpreter::GcTrait<Self>> {
            $(
                $obj_name($obj),
            )+
        }
        $(
            impl<$gc_name: $crate::interpreter::GcTrait<Self>> From<$obj> for $name<$gc_name> {
                fn from(inner: $obj)->Self {
                    $name::$obj_name(inner)
                }
            }
        )+
        impl<$gc_name: $crate::interpreter::GcTrait<Self>> ObjectBundle<$gc_name> for $name<$gc_name> {}
        impl<$gc_name: $crate::interpreter::GcTrait<Self>> Object<$gc_name> for $name<$gc_name> {
            type ObjectBundle = Self;

            #[inline]
            fn get(&self,
                name: $crate::ast::Ident,
                interner: &$crate::ast::Interner,
            )->anyhow::Result<$crate::interpreter::Primitive<$gc_name, $name<$gc_name>>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.get(name, interner),
                    )+
                }
            }
            #[inline]
            fn set(&mut self,
                name: $crate::ast::Ident,
                data: $crate::interpreter::Primitive<$gc_name, $name<$gc_name>>,
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
                args: Vec<$crate::interpreter::Primitive<$gc_name, $name<$gc_name>>>,
                interner: &$crate::ast::Interner,
                gc: &mut $gc_name,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<$gc_name, Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.call(args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn method(&mut self,
                name: $crate::ast::Ident,
                args: Vec<$crate::interpreter::Primitive<$gc_name, $name<$gc_name>>>,
                interner: &$crate::ast::Interner,
                gc: &mut $gc_name,
            )->anyhow::Result<$crate::interpreter::object::CallReturn<$gc_name, Self>> {
                match self {
                    $(
                        $name::$obj_name(variant)=>variant.method(name, args, interner, gc),
                    )+
                }
            }

            #[inline]
            fn trace(&self, tracer: &mut $gc_name) {
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


bundle_object_types! {
    bundle CoreObjectBundle<Gc> {
        Base: BaseObject<Gc, CoreObjectBundle<Gc>>,
    }
}



pub trait Object<Gc: GcTrait<Self::ObjectBundle>>: Sized {
    type ObjectBundle: ObjectBundle<Gc> + Debug;

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

    fn get(&self, name: Ident, interner: &Interner)->Result<Primitive<Gc, Self::ObjectBundle>>;
    fn set(&mut self, name: Ident, data: Primitive<Gc, Self::ObjectBundle>, interner: &Interner)->Result<()>;

    fn call(&mut self,
        args: Vec<Primitive<Gc, Self::ObjectBundle>>,
        interner: &Interner,
        gc: &mut Gc,
    )->Result<CallReturn<Gc, Self::ObjectBundle>>;
    fn method(&mut self,
        name: Ident,
        args: Vec<Primitive<Gc, Self::ObjectBundle>>,
        interner: &Interner,
        gc: &mut Gc,
    )->Result<CallReturn<Gc, Self::ObjectBundle>>;

    fn trace(&self, tracer: &mut Gc);
}

pub trait ObjectBundle<Gc: GcTrait<Self::ObjectBundle>>: Sized + Object<Gc, ObjectBundle = Self> + Debug {
}


pub enum CallReturn<Gc: GcTrait<O>, O: ObjectBundle<Gc>> {
    CallFn(FnId, Vec<Primitive<Gc, O>>),
    Data(Primitive<Gc, O>),
}


#[derive(Debug)]
pub struct BaseObject<Gc: GcTrait<O>, O: ObjectBundle<Gc>>(IdentMap<Primitive<Gc, O>>);
impl<Gc: GcTrait<O>, O: ObjectBundle<Gc>> Object<Gc> for BaseObject<Gc, O> {
    type ObjectBundle = O;

    fn get(&self, name: Ident, _: &Interner)->Result<Primitive<Gc, O>> {
        self.0.get(&name)
            .cloned()
            .ok_or(anyhow!("Does not contain the field"))
    }
    fn set(&mut self, name: Ident, data: Primitive<Gc, O>, _: &Interner)->Result<()> {
        self.0.insert(name, data);
        Ok(())
    }

    fn call(&mut self, _: Vec<Primitive<Gc, O>>, _: &Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
        bail!("Cannot call BaseObject");
    }
    fn method(&mut self, _: Ident, _: Vec<Primitive<Gc, O>>, _: &Interner, _: &mut Gc)->Result<CallReturn<Gc, O>> {
        bail!("BaseObject has no methods");
    }

    fn trace(&self, tracer: &mut Gc) {
        for val in self.0.values() {
            match val {
                Primitive::Data(d)=>tracer.trace(d.clone()),
                _=>{},
            }
        }
    }
}
