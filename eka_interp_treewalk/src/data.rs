use rustc_hash::FxHashSet;
use anyhow::{
    Result,
    bail,
};
use std::{
    alloc::{
        Layout,
        alloc as global_alloc,
        dealloc as global_dealloc,
    },
    hash::{
        Hash,
        Hasher,
    },
    fmt::{
        Debug,
        Formatter,
        Result as FmtResult,
    },
    ops::{
        Deref,
        DerefMut,
    },
    marker::PhantomData,
    cell::Cell,
    ptr::NonNull,
    mem,
    thread_local,
};
use eka_core::{
    interpreter::{
        object::*,
        GcTrait,
        Primitive,
    },
    ast::{
        Ident,
        Interner,
    },
    misc::FxIndexSet,
};


thread_local! {
    static GC_WORKLOAD: Cell<GcWorkload> = Cell::new(GcWorkload {
        traces: 100,
        mark_dead: 10,
        gc_when_no_dead: true,
    });
}


pub enum GcState {
    MarkRoots,
    Trace,
    MarkDead,
}


#[must_use]
pub struct DataRef<O: ObjectBundle<Gc<O>>>(NonNull<DataBox<O>>);
impl<O: ObjectBundle<Gc<O>>> Debug for DataRef<O> {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        unsafe {self.0.as_ref()}.data.fmt(f)
    }
}
impl<O: ObjectBundle<Gc<O>>> Clone for DataRef<O> {
    fn clone(&self)->Self {
        DataRef(self.0)
    }
}
impl<O: ObjectBundle<Gc<O>>> PartialEq for DataRef<O> {
    fn eq(&self, o: &Self)->bool {
        self.0 == o.0
    }
}
impl<O: ObjectBundle<Gc<O>>> Eq for DataRef<O> {}
impl<O: ObjectBundle<Gc<O>>> Hash for DataRef<O> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}
impl<O: ObjectBundle<Gc<O>>> Deref for DataRef<O> {
    type Target = O;
    fn deref(&self)->&O {
        &self.get_box().data
    }
}
impl<O: ObjectBundle<Gc<O>>> DerefMut for DataRef<O> {
    fn deref_mut(&mut self)->&mut O {
        &mut self.get_box_mut().data
    }
}
impl<O: ObjectBundle<Gc<O>>> DataRef<O> {
    #[inline]
    pub fn root(self, gc: &mut Gc<O>)->Option<RootDataRef<O>> {
        gc.root(self.clone())
    }
}
impl<O: ObjectBundle<Gc<O>>> DataRef<O> {
    fn get_box_mut(&mut self)->&mut DataBox<O> {
        unsafe {self.0.as_mut()}
    }
    fn get_box(&self)->&DataBox<O> {
        unsafe {self.0.as_ref()}
    }
}

#[must_use]
pub struct RootDataRef<O: ObjectBundle<Gc<O>>>(DataRef<O>);
impl<O: ObjectBundle<Gc<O>>> Deref for RootDataRef<O> {
    type Target = O;
    fn deref(&self)->&O {
        &self.0
    }
}
impl<O: ObjectBundle<Gc<O>>> DerefMut for RootDataRef<O> {
    fn deref_mut(&mut self)->&mut O {
        &mut self.0
    }
}

struct DataBox<O: ObjectBundle<Gc<O>>> {
    data: O,
}

#[derive(Copy, Clone)]
pub struct GcWorkload {
    traces: usize,
    mark_dead: usize,
    gc_when_no_dead: bool,
}

/// An object form of GcWorkload that can be used in the interpreter.
#[derive(Debug)]
pub struct GcWorkloadObject<O: ObjectBundle<Gc<O>>> {
    mark_dead: Ident,
    traces: Ident,
    gc_when_no_dead: Ident,
    _phantom: PhantomData<fn()->O>,
}
impl<O: ObjectBundle<Gc<O>>> GcWorkloadObject<O> {
    pub fn new(interner: &mut Interner)->Self {
        GcWorkloadObject {
            mark_dead: interner.intern("markDead"),
            traces: interner.intern("traces"),
            gc_when_no_dead: interner.intern("gcWhenNoDead"),
            _phantom: PhantomData,
        }
    }
}
impl<O: ObjectBundle<Gc<O>>> Object<Gc<O>> for GcWorkloadObject<O> {
    type ObjectBundle = O;

    fn get(&self, name: Ident, _: &Interner)->Result<Primitive<Gc<O>, O>> {
        let wl = GC_WORKLOAD.get();

        if name == self.mark_dead {
            return Ok(Primitive::Number(wl.mark_dead as i64));
        }
        if name == self.traces {
            return Ok(Primitive::Number(wl.traces as i64));
        }
        if name == self.gc_when_no_dead {
            return Ok(Primitive::Bool(wl.gc_when_no_dead));
        }

        bail!("No field with the given name on GcWorkload");
    }

    fn set(&mut self, name: Ident, data: Primitive<Gc<O>, O>, _: &Interner)->Result<()> {
        let mut wl = GC_WORKLOAD.get();

        if name == self.mark_dead {
            match data {
                Primitive::Number(n)=>wl.mark_dead = n as usize,
                _=>bail!("GcWorkload.mark_dead is a Number"),
            }
            GC_WORKLOAD.set(wl);
        }
        if name == self.traces {
            match data {
                Primitive::Number(n)=>wl.traces = n as usize,
                _=>bail!("GcWorkload.traces is a Number"),
            }
            GC_WORKLOAD.set(wl);
        }
        if name == self.gc_when_no_dead {
            match data {
                Primitive::Bool(b)=>wl.gc_when_no_dead = b,
                _=>bail!("GcWorkload.gc_when_no_dead is a Bool"),
            }
            GC_WORKLOAD.set(wl);
        }

        bail!("No field with the given name on GcWorkload");
    }

    fn call(&mut self, _: Vec<Primitive<Gc<O>, O>>, _: &Interner, _: &mut Gc<O>)->Result<CallReturn<Gc<O>, O>> {
        bail!("Cannot call GcWorkload");
    }

    fn method(&mut self, _: Ident, _: Vec<Primitive<Gc<O>, O>>, _: &Interner, _: &mut Gc<O>)->Result<CallReturn<Gc<O>, O>> {
        bail!("GcWorkload has no methods");
    }

    fn trace(&self, _: &mut Gc<O>) {}
}

pub struct Gc<O: ObjectBundle<Gc<O>>> {
    white: FxIndexSet<DataRef<O>>,
    grey: FxIndexSet<DataRef<O>>,
    black: FxIndexSet<DataRef<O>>,
    roots: FxHashSet<DataRef<O>>,
    dead: FxIndexSet<DataRef<O>>,
    state: GcState,
}
impl<O: ObjectBundle<Self>> Debug for Gc<O> {
    fn fmt(&self, f: &mut Formatter)->FmtResult {
        write!(f, "<GC>")
    }
}
impl<O: ObjectBundle<Gc<O>>> Gc<O> {
    pub fn new()->Self {
        Gc {
            white: FxIndexSet::default(),
            grey: FxIndexSet::default(),
            black: FxIndexSet::default(),
            roots: FxHashSet::default(),
            dead: FxIndexSet::default(),
            state: GcState::MarkRoots,
        }
    }

    pub fn root(&mut self, dr: DataRef<O>)->Option<RootDataRef<O>> {
        if self.roots.contains(&dr) {
            return None;
        }

        self.roots.insert(dr.clone());
        return Some(RootDataRef(dr));
    }

    pub fn unroot(&mut self, dr: RootDataRef<O>) {
        self.roots.remove(&dr.0);
        drop(dr);
    }

    pub fn alloc(&mut self, data: O)->DataRef<O> {
        // short-circuit if there is already a dead object.
        if let Some(mut dr) = self.dead.swap_remove_index(0) {
            dr.get_box_mut()
                .data
                .recycle_insert(data);

            return dr;
        }

        let db = DataBox {data};

        let layout = Layout::new::<DataBox<O>>();

        // SAFETY: We are allocating via the global allocator...
        let raw_ptr = unsafe {global_alloc(layout)};
        // ... then we assert the allocation is not null
        let nn_ptr = NonNull::new(raw_ptr)
            .expect("Could not allocate")
            .cast::<DataBox<O>>();

        // SAFETY: We are initializing the memory
        unsafe {
            nn_ptr.write(db);
        }

        let dr = DataRef(nn_ptr);
        self.grey.insert(dr.clone());

        if self.dead.len() == 0 && GC_WORKLOAD.get().gc_when_no_dead {
            self.gc_inc();
        }

        return dr;
    }

    pub fn gc_inc(&mut self) {
        use GcState::*;
        match self.state {
            MarkRoots=>{
                self.mark_roots();
                self.state = Trace;
            },
            Trace=>{
                self.trace();
                if self.grey.is_empty() {
                    self.state = MarkDead;
                }
            },
            MarkDead=>{
                self.mark_dead();
                if self.white.is_empty() {
                    mem::swap(&mut self.white, &mut self.black);
                    self.state = MarkRoots;
                }
            },
        }
    }

    fn mark_roots(&mut self) {
        for root in self.roots.iter() {
            self.grey.insert(root.clone());
            self.white.swap_remove(root);
        }
    }

    fn trace(&mut self) {
        let wl = GC_WORKLOAD.get();
        let mut count = 0;

        // for every item in the grey list, trace it and mark it black
        while let Some(dr) = self.grey.shift_remove_index(0) {
            self.black.insert(dr.clone());
            dr.get_box().data.trace(self);
            
            count += 1;
            if count > wl.traces {
                break;
            }
        }
    }

    fn mark_dead(&mut self) {
        let wl = GC_WORKLOAD.get();
        let mut count = 0;

        // for every white item left (now dead) check if it needs immediate dropping and do so.
        // Otherwise put it in the dead list for recycling later.
        while let Some(mut dr) = self.white.swap_remove_index(0) {
            dr.finalize();
            if dr.can_recycle() {
                self.dead.insert(dr);
            } else {
                unsafe {
                    self.cleanup_single_dead(dr);
                }
            }

            count += 1;
            if count > wl.mark_dead {
                break;
            }
        }
    }

    unsafe fn cleanup_single_dead(&self, dr: DataRef<O>) {
        // Assert we are not included in any of the lists
        debug_assert!(!self.white.contains(&dr));
        debug_assert!(!self.grey.contains(&dr));
        debug_assert!(!self.black.contains(&dr));
        debug_assert!(!self.roots.contains(&dr));
        debug_assert!(!self.dead.contains(&dr));

        // SAFETY: We are cleaning up initialized memory
        unsafe {dr.0.drop_in_place()};

        let layout = Layout::new::<DataBox<O>>();
        let raw_ptr = dr.0.as_ptr().cast::<u8>();
        // SAFETY: We are deallocating memory that we are reasonably sure has no pointers outside
        // of here. Of course, we can't be sure of that, so this function is marked unsafe.
        unsafe {global_dealloc(raw_ptr, layout)};
    }
}
impl<O: ObjectBundle<Gc<O>>> GcTrait<O> for Gc<O> {
    type DataRef = DataRef<O>;
    
    fn alloc<RO: Into<O>>(&mut self, data: RO)->DataRef<O> {
        self.alloc(data.into())
    }

    fn trace(&mut self, ptr: DataRef<O>) {
        // don't insert if its in the black list already
        if self.black.contains(&ptr) {
            // assert the ptr is NOT in the white list or grey list
            debug_assert!(!self.white.contains(&ptr));
            debug_assert!(!self.grey.contains(&ptr));
            return;
        }

        self.white.swap_remove(&ptr);
        self.grey.insert(ptr);
    }
}
