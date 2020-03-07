use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::ops::Deref;
use std::cell::Cell;

pub type Object = Box<Block<dyn Allocation>>;

pub trait Allocation {}

pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

pub enum ValueType {
    Number(f64),
    String(HeapRef<String>)
}

pub struct Header {
    marked : Cell<bool>
}

pub struct HeapRef<T: 'static + ?Sized + Allocation>{
    pub ptr : NonNull<Block<T>>
}

impl <T: 'static + ?Sized + Allocation> HeapRef<T>{
    pub fn new(ptr : NonNull<Block<T>>) -> Self{
        Self{
            ptr
        }
    }
}

pub struct Heap {
    objects : Vec<Object>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects : vec![]
        }
    }

    pub fn allocate<T: 'static + Allocation>(&mut self, data : T) -> HeapRef<T> {
        let mut allocation = Box::new(
            Block{
                header : Header::default(),
                data
            }
        );
        let ptr = unsafe { NonNull::new_unchecked(&mut *allocation) };

        self.objects.push(allocation);

        HeapRef::new(ptr)
    }

    pub fn collect(&mut self){
        self.objects.retain(|val| val.header.marked.get())
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            marked : Cell::new(false)
        }
    }
}

impl<T: 'static + Allocation + ?Sized> Deref for HeapRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(self.ptr.as_ref()).data
        }
    }
}

impl Allocation for String {}
