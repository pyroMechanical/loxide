use std::cell::{Cell, RefCell, UnsafeCell};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

struct GcState {
    allocations: Option<NonNull<GcBox<dyn Trace>>>,
}

impl GcState {
    fn len(&self) -> usize {
        let mut len = 0;
        let mut next = self.allocations;
        while let Some(allocation) = next {
            len += 1;
            next = unsafe{&*allocation.as_ptr()}.next.get();
        };
        len
    }

    fn collect_garbage(&mut self) {
        //println!("-- gc begin");
        let mut current = self.allocations;
        while let Some(allocation) = current {
            unsafe {
                let gc_box = allocation.as_ref();
                if gc_box.roots.get() > 0 {
                    //println!("mark value");
                    gc_box.trace_inner();
                }
                current = gc_box.next.get();
            }
        }
        let mut previous: Option<NonNull<GcBox<dyn Trace>>> = None;
        current = self.allocations;
        while let Some(allocation) = current {
            unsafe {
                let gc_box = allocation.as_ref();
                let next = gc_box.next.get();
                if !gc_box.is_marked.get() {
                    println!("freed allocation!");
                    match previous {
                        None => self.allocations = next,
                        Some(previous) => (&*previous.as_ptr()).next.set(next),
                    }
                    std::mem::drop(Box::from_raw(allocation.as_ptr()));
                }
                else {
                    previous = current;
                }
                current = next;
            }
        }
        //println!("-- gc end")
    }
}

pub fn allocations() -> usize{
    GC_STATE.with(|state|
        state.borrow().len()
    )
}

thread_local! {
    static GC_STATE: RefCell<GcState> = RefCell::new(GcState{allocations: None});
}

pub unsafe trait Trace {
    fn trace(&self) {}

    fn root(&self) {}

    fn unroot(&self) {}
}
#[repr(C)]
struct GcBox<T: ?Sized> {
    is_marked: Cell<bool>,
    roots: Cell<usize>,
    next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
    value: T,
}

impl<T: ?Sized> GcBox<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn root_inner(&self) {
        self.roots.set(self.roots.get() + 1);
    }

    fn unroot_inner(&self) {
        self.roots.set(self.roots.get() - 1);
    }
}

impl<T: Trace> GcBox<T> {
    fn new(value: T) -> NonNull<GcBox<T>> {
        let boxed = Box::new(GcBox {
            is_marked: Cell::new(false),
            roots: Cell::new(1),
            next: Cell::new(None),
            value,
        });
        NonNull::from(Box::leak(boxed))
    }
}

impl<T: ?Sized + Trace> GcBox<T> {
    fn add_next(&self, next: NonNull<GcBox<dyn Trace>>) {
        self.next.set(Some(next));
    }

    fn trace_inner(&self) {
        if !self.is_marked.get() { 
            self.is_marked.set(true);
            self.value.trace()
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
enum BorrowState {
    Reading,
    Writing,
    Unused,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct BorrowFlag(usize);

impl BorrowFlag {
    const ROOT: usize = 1 << std::mem::size_of::<usize>() - 1;
    const WRITING: usize = !BorrowFlag::ROOT;
    const UNUSED: usize = 0;

    const NEW: BorrowFlag = BorrowFlag(Self::ROOT);

    fn borrowed(self) -> BorrowState {
        match self.0 & !Self::ROOT {
            Self::WRITING => BorrowState::Writing,
            Self::UNUSED => BorrowState::Unused,
            _ => BorrowState::Reading,
        }
    }

    fn rooted(self) -> bool {
        self.0 & BorrowFlag::ROOT != 0
    }

    fn set_writing(self) -> Self {
        BorrowFlag(self.0 | Self::WRITING)
    }

    fn set_unused(self) -> Self {
        BorrowFlag(self.0 & Self::ROOT)
    }

    fn add_reading(self) -> Self {
        assert!(self.borrowed() != BorrowState::Writing);
        BorrowFlag(self.0 + 1)
    }

    fn sub_reading(self) -> Self {
        assert!(self.borrowed() == BorrowState::Reading);
        BorrowFlag(self.0 - 1)
    }

    fn set_rooted(self, rooted: bool) -> Self {
        BorrowFlag(self.0 & !Self::ROOT | if rooted { Self::ROOT } else { 0 })
    }
}
#[repr(C)]
struct GcCell<T> {
    flags: Cell<BorrowFlag>,
    value: UnsafeCell<T>,
}

impl<T> GcCell<T> {
    pub fn new(value: T) -> GcCell<T> {
        GcCell {
            flags: Cell::new(BorrowFlag::NEW),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: Trace> GcCell<T> {
    pub fn try_borrow(&self) -> Option<GcCellRef<'_, T>> {
        if self.flags.get().borrowed() == BorrowState::Writing {
            return None;
        }

        self.flags.set(self.flags.get().add_reading());

        assert!(self.flags.get().borrowed() == BorrowState::Reading);
        unsafe {
            Some(GcCellRef {
                flags: &self.flags,
                value: &*self.value.get(),
            })
        }
    }

    pub fn borrow(&self) -> GcCellRef<'_, T> {
        match self.try_borrow() {
            None => panic!("BorrowError"),
            Some(value) => value,
        }
    }

    pub fn try_borrow_mut(&self) -> Option<GcCellRefMut<'_, T>> {
        if self.flags.get().borrowed() != BorrowState::Unused {
            return None;
        }

        self.flags.set(self.flags.get().set_writing());
        unsafe {
            if !self.flags.get().rooted() {
                (*self.value.get()).root();
            }

            Some(GcCellRefMut {
                gc_cell: self,
                value: &mut *self.value.get(),
            })
        }
    }

    pub fn borrow_mut(&self) -> GcCellRefMut<'_, T> {
        match self.try_borrow_mut() {
            None => panic!("BorrowMutError"),
            Some(value) => value,
        }
    }
}

unsafe impl<T: Trace> Trace for GcCell<T> {
    fn trace(&self) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (), //GcCellRefMut roots when writing, so there's no need to mark
            _ => unsafe { &*self.value.get() }.trace(),
        }
    }
}

pub struct GcCellRef<'a, T: Trace> {
    flags: &'a Cell<BorrowFlag>,
    value: &'a T,
}

impl<'a, T: Trace> Deref for GcCellRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Trace> Drop for GcCellRef<'a, T> {
    fn drop(&mut self) {
        debug_assert!(self.flags.get().borrowed() == BorrowState::Reading);
        self.flags.set(self.flags.get().sub_reading());
    }
}

pub struct GcCellRefMut<'a, T: Trace> {
    gc_cell: &'a GcCell<T>,
    value: &'a mut T,
}

impl<'a, T: Trace> Deref for GcCellRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Trace> DerefMut for GcCellRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T: Trace> Drop for GcCellRefMut<'a, T> {
    fn drop(&mut self) {
        debug_assert!(self.gc_cell.flags.get().borrowed() == BorrowState::Writing);

        if !self.gc_cell.flags.get().rooted() {
            unsafe {
                (*self.gc_cell.value.get()).unroot();
            }
        }

        self.gc_cell
            .flags
            .set(self.gc_cell.flags.get().set_unused());
    }
}

#[repr(C)]
pub struct Gc<T: Trace + 'static> {
    ptr: Cell<NonNull<GcBox<GcCell<T>>>>,
}

unsafe fn clear_root_bit<T>(ptr: NonNull<GcBox<GcCell<T>>>) -> NonNull<GcBox<GcCell<T>>> {
    let addr = ptr.as_ptr() as isize;
    let new_addr = addr & !1;
    let new_ptr = new_addr as *mut GcBox<GcCell<T>>;
    NonNull::new_unchecked(new_ptr)
}

impl<T: Trace> Gc<T> {
    pub fn new(value: T) -> Gc<T> {
        let gc_box = Cell::new(GcBox::new(GcCell::new(value)));
        GC_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.collect_garbage();
            let next = state.allocations.replace(gc_box.get());
            if let Some(next) = next {
                unsafe{&*state.allocations.unwrap().as_ptr()}.add_next(next);
            }
        });
        unsafe {
            (*(gc_box.get().as_ptr())).value().unroot();
        }
        let result = Gc { ptr: gc_box };
        unsafe { result.set_root() };
        result
    }

    unsafe fn set_root(&self) {
        let addr = self.ptr.get().as_ptr() as isize;
        let new_addr = addr | 1;
        let new_ptr = new_addr as *mut GcBox<GcCell<T>>;
        self.ptr.set(NonNull::new_unchecked(new_ptr))
    }

    unsafe fn clear_root(&self) {
        self.ptr.set(clear_root_bit(self.ptr.get()))
    }

    fn rooted(&self) -> bool {
        self.ptr.get().as_ptr().cast::<u8>() as usize & 1 != 0
    }

    fn inner(&self) -> &GcBox<GcCell<T>> {
        unsafe { &*clear_root_bit(self.ptr.get()).as_ptr() }
    }

    pub fn borrow(&self) -> GcCellRef<T> {
        self.inner().value().borrow()
    }

    pub fn borrow_mut(&self) -> GcCellRefMut<T> {
        let gc_ref = self.inner().value().borrow_mut();
        if !self.rooted() {
            self.root();
        }
        gc_ref
    }

    pub fn root_count(&self) -> usize {
        self.inner().roots.get()
    }
}

impl<T: Trace> Clone for Gc<T> {
    fn clone(&self) -> Gc<T> {
        self.inner().root_inner();
        let gc = Gc {
            ptr: Cell::new(self.ptr.get()),
        };
        unsafe {
            gc.set_root();
        }
        gc
    }
}

impl<T: Trace + PartialEq> PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        *self.inner().value().borrow() == *other.inner().value().borrow()
    }
}

impl<T: Trace + Eq> Eq for Gc<T> {}

impl<T: Trace> Drop for Gc<T> {
    fn drop(&mut self) {
        if self.rooted() {
            self.inner().unroot_inner();
        }
    }
}

impl<T: Trace + Display> Display for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}

unsafe impl<T: Trace> Trace for Gc<T> {
    fn trace(&self) {
        let inner = self.inner().trace_inner();
    }

    fn root(&self) {
        assert!(!self.rooted(), "Can't double root a Gc<T>");
        self.inner().root_inner();
        unsafe {
            self.set_root();
        }
    }

    fn unroot(&self) {
        assert!(self.rooted(), "Can't double unroot a Gc<T>");
        self.inner().unroot_inner();
        unsafe {
            self.clear_root();
        }
    }
}

impl<T: Trace + Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner().value().borrow().fmt(f)
    }
}

impl<T: Trace + Hash> Hash for Gc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner().value().borrow().hash(state);
    }
}

type GcRef<'a, T> = std::cell::Ref<'a, T>;

pub struct GcRefMut<'a, T: Trace> {
    gc_box: &'a GcBox<RefCell<T>>,
    gc_ref: std::cell::RefMut<'a, T>,
}

impl<'a, T: Trace> Deref for GcRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.gc_ref
    }
}

impl<'a, T: Trace> DerefMut for GcRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.gc_ref
    }
}

impl<'a, T: Trace> Drop for GcRefMut<'a, T> {
    fn drop(&mut self) {
        self.gc_box.unroot_inner();
        self.gc_ref.unroot();
    }
}

unsafe impl<K: Trace, V: Trace> Trace for std::collections::HashMap<K, V> {
    fn trace(&self) {
        for (k, v) in self.iter() {
            k.trace();
            v.trace();
        }
    }

    fn root(&self) {
        for (k, v) in self.iter() {
            k.root();
            v.root();
        }
    }

    fn unroot(&self) {
        for (k, v) in self.iter() {
            k.unroot();
            v.unroot();
        }
    }
}

unsafe impl<K: Trace> Trace for std::collections::HashSet<K> {
    fn trace(&self) {
        for k in self.iter() {
            k.trace();
        }
    }

    fn root(&self) {
        for k in self.iter() {
            k.root();
        }
    }

    fn unroot(&self) {
        for k in self.iter() {
            k.unroot();
        }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    fn trace(&self) {
        for v in self.iter() {
            v.trace();
        }
    }

    fn root(&self) {
        for v in self.iter() {
            v.root();
        }
    }

    fn unroot(&self) {
        for v in self.iter() {
            v.unroot();
        }
    }
}

unsafe impl<T: Trace> Trace for Box<T> {
    fn trace(&self) {
        self.as_ref().trace()
    }
    fn root(&self) {
        self.as_ref().root()
    }
    fn unroot(&self) {
        self.as_ref().unroot()
    }
}

macro_rules! impl_empty_trace {
    ($($Type: ty), *) => {
        $(
            unsafe impl Trace for $Type {
                fn trace(&self) {}
                fn root(&self) {}
                fn unroot(&self) {}
            }
        )*
    };
}

impl_empty_trace!(
    bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, str, char
);
