use anyhow::{
    Result,
    bail,
};
use std::{
    any::{
        TypeId,
        Any,
    },
    alloc::Layout,
    ptr::NonNull,
};
use crate::{
    ast::Ident
};
use super::{
    FnId,
    Primitive,
};


pub trait Object: Any {
    fn get(&self, name: Ident)->Result<Primitive>;
    fn set(&mut self, name: Ident, data: Primitive)->Result<()>;

    fn call(&mut self, args: Vec<Primitive>)->Result<CallReturn>;
    fn method(&mut self, name: Ident, args: Vec<Primitive>)->Result<CallReturn>;

    fn compare(&self, other: &dyn Object)->bool;
}
impl PartialEq for dyn Object {
    fn eq(&self, other: &Self)->bool {
        self.compare(other)
    }
}
impl dyn Object {
    /// Implementation of [`Any::downcast_ref`] for [`dyn Object`] until dyn trait upcasts are
    /// stabilized and we can use
    /// ```rust
    /// <dyn Any>::downcast_ref::<T>(self)
    /// ```
    pub fn downcast_ref<T: Any>(&self)->Option<&T> {
        let self_id = self.type_id();
        let t_id = TypeId::of::<T>();

        if self_id != t_id {
            return None;
        }

        // SAFETY: we ensure the types are the same, and this code is copied from
        // [https://doc.rust-lang.org/1.79.0/src/core/any.rs.html#299].
        return Some(unsafe { &*(self as *const dyn Object as *const T) });
    }

    /// Implementation of [`Any::downcast_mut`] for [`dyn Object`] until dyn trait upcasts are
    /// stabilized and we can use
    /// ```rust
    /// <dyn Any>::downcast_mut::<T>(self)
    /// ```
    pub fn downcast_mut<T: Object>(&mut self)->Option<&mut T> {
        let self_id = (&*self).type_id();
        let t_id = TypeId::of::<T>();

        if self_id != t_id {
            return None;
        }

        // SAFETY: we ensure the types are the same, and this code is copied from
        // [https://doc.rust-lang.org/1.79.0/src/core/any.rs.html#329].
        return Some(unsafe { &mut *(self as *mut dyn Object as *mut T) });
    }
}

trait StaticObject {
    fn get_static(ptr: NonNull<dyn Object>, name: Ident)->Result<Primitive>;
    fn set_static(ptr: NonNull<dyn Object>, name: Ident, data: Primitive)->Result<()>;

    fn call_static(ptr: NonNull<dyn Object>, args: Vec<Primitive>)->Result<CallReturn>;
    fn method_static(ptr: NonNull<dyn Object>, name: Ident, args: Vec<Primitive>)->Result<CallReturn>;

    fn compare_static(ptr: NonNull<dyn Object>, other: &dyn Object)->bool;
}
impl<T: Object> StaticObject for T {
    #[inline]
    fn get_static(ptr: NonNull<dyn Object>, name: Ident)->Result<Primitive> {
        // SAFETY: This method is only called from `ObjectHeader::get`, and is not accessible
        // anywhere else. We know that the values passed into this function are valid.
        let ptr_ref = unsafe {ptr.cast::<T>().as_ref()};
        ptr_ref.get(name)
    }
    #[inline]
    fn set_static(ptr: NonNull<dyn Object>, name: Ident, data: Primitive)->Result<()> {
        // SAFETY: This method is only called from `ObjectHeader::set`, and is not accessible
        // anywhere else. We know that the values passed into this function are valid.
        let ptr_ref = unsafe {ptr.cast::<T>().as_mut()};
        ptr_ref.set(name, data)
    }

    #[inline]
    fn call_static(ptr: NonNull<dyn Object>, args: Vec<Primitive>)->Result<CallReturn> {
        // SAFETY: This method is only called from `ObjectHeader::call`, and is not accessible
        // anywhere else. We know that the values passed into this function are valid.
        let ptr_ref = unsafe {ptr.cast::<T>().as_mut()};
        ptr_ref.call(args)
    }
    #[inline]
    fn method_static(ptr: NonNull<dyn Object>, name: Ident, args: Vec<Primitive>)->Result<CallReturn> {
        // SAFETY: This method is only called from `ObjectHeader::method`, and is not accessible
        // anywhere else. We know that the values passed into this function are valid.
        let ptr_ref = unsafe {ptr.cast::<T>().as_mut()};
        ptr_ref.method(name, args)
    }

    #[inline]
    fn compare_static(ptr: NonNull<dyn Object>, other: &dyn Object)->bool {
        // SAFETY: This method is only called from `ObjectHeader::compare`, and is not accessible
        // anywhere else. We know that the values passed into this function are valid.
        let ptr_ref = unsafe {ptr.cast::<T>().as_ref()};
        ptr_ref.compare(other)
    }
}


type GetFn = fn(NonNull<dyn Object>, Ident)->Result<Primitive>;
type SetFn = fn(NonNull<dyn Object>, Ident, Primitive)->Result<()>;
type CallFn = fn(NonNull<dyn Object>, Vec<Primitive>)->Result<CallReturn>;
type MethodFn = fn(NonNull<dyn Object>, Ident, Vec<Primitive>)->Result<CallReturn>;
type CompareFn = fn(NonNull<dyn Object>, &dyn Object)->bool;


pub enum CallReturn {
    CallFn(FnId, Vec<Primitive>),
    Data(Primitive),
}


#[repr(C)]
pub(in super) struct ObjectHeader<T: 'static> {
    ptr: NonNull<dyn Object>,

    get: GetFn,
    set: SetFn,
    call: CallFn,
    method: MethodFn,
    compare: CompareFn,

    /// A function that returns the layout of `Self + dyn Object` and the offset from
    /// `NonNull<Self>` to get to `NonNull<dyn Object>`.
    layout: fn()->(Layout, usize),

    /// Any extra book-keeping information required by a GC or whatever.
    extra: T,
}
impl<T> Drop for ObjectHeader<T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.drop_in_place()
        }
    }
}
impl<T> ObjectHeader<T> {
    pub fn layout<O: 'static + Object>()->(Layout, usize) {
        Layout::new::<Self>()
            .extend(Layout::new::<O>())
            .expect("Could not append object layout to header layout before allocation")
    }

    /// Allocate a new object header and data then return the pointer.
    #[must_use]
    pub fn alloc<O: 'static + Object>(data: O, extra: T)->NonNull<Self> {
        let (layout, data_offset) = Self::layout::<O>();

        // SAFETY: This is an allocation. It should be safe.
        let raw_ptr = unsafe {std::alloc::alloc(layout)};

        let header_ptr = NonNull::new(raw_ptr.cast::<Self>())
            .expect("Could not allocate memory for object");

        // SAFETY: We havev already allocated the memory, and we are using the offset provided by a
        // known-good source: `std::alloc::Layout`
        let data_ptr = unsafe {header_ptr.cast::<O>().byte_offset(data_offset as isize)};

        // SAFETY: We are initializing memory
        unsafe {
            data_ptr.write(data);
        }

        // SAFETY: We are initializing memory
        unsafe {
            header_ptr.write(ObjectHeader {
                ptr: data_ptr,
                get: <O as StaticObject>::get_static,
                set: <O as StaticObject>::set_static,
                call: <O as StaticObject>::call_static,
                method: <O as StaticObject>::method_static,
                compare: <O as StaticObject>::compare_static,
                layout: Self::layout::<O>,
                extra,
            });
        }

        // SAFETY: We have already initialized all memory allocated.
        return header_ptr;
    }
}
impl<T> Object for ObjectHeader<T> {
    #[inline]
    fn get(&self, name: Ident)->Result<Primitive> {
        (self.get)(self.ptr, name)
    }

    #[inline]
    fn set(&mut self, name: Ident, data: Primitive)->Result<()> {
        (self.set)(self.ptr, name, data)
    }

    #[inline]
    fn call(&mut self, args: Vec<Primitive>)->Result<CallReturn> {
        (self.call)(self.ptr, args)
    }

    #[inline]
    fn method(&mut self, name: Ident, args: Vec<Primitive>)->Result<CallReturn> {
        (self.method)(self.ptr, name, args)
    }

    #[inline]
    fn compare(&self, other: &dyn Object)->bool {
        (self.compare)(self.ptr, other)
    }
}
