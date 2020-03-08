use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use std::cell::Cell;
use std::any::Any;
use std::fmt;
use std::mem;
use std::alloc::{alloc, handle_alloc_error, Layout};

pub trait Allocation {}

pub type Object = Box<Block<dyn Allocation>>;

pub struct Header {
    marked : Cell<bool>
}

pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

pub struct HeapRef<T: 'static + Allocation>(NonNull<Block<T>>);

#[derive(Copy, Clone)]
pub enum ValueType {
    Number(HeapRef<f64>),
    String(HeapRef<String>),
    Array(HeapRef<Vec<ValueType>>)
}

impl <T: 'static + Allocation> HeapRef<T>{
    pub fn new(ptr : NonNull<Block<T>>) -> Self{
        HeapRef(ptr)
    }
}

impl <T: 'static + Allocation> Copy for HeapRef<T> {}
impl <T: 'static + Allocation> Clone for HeapRef<T> {
    fn clone(&self) -> Self {
        unimplemented!()
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

    pub fn allocate_bytes(&mut self, size : usize) -> *mut u8{
        let layout = Layout::from_size_align(size, 8).expect("Error");

        let pointer = unsafe { alloc(layout) };

        if pointer.is_null(){
            handle_alloc_error(layout);
        }

        pointer
    }

    pub fn allocate<T: 'static + Allocation>(&mut self, data : T) -> HeapRef<T> {

        let mut allocation = Box::new(
            Block {
                header : Header::default(),
                data
            }
        );

        let pointer = unsafe { NonNull::new_unchecked(&mut *allocation)};

        self.objects.push(allocation);

        HeapRef::new(pointer)
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

impl<T: 'static + Allocation> Deref for HeapRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &(self.0.as_ref()).data
        }
    }
}

impl<T: 'static + Allocation> AsRef<T> for HeapRef<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl Allocation for String {}
impl Allocation for &str {}
impl Allocation for f64 {}
impl Allocation for Vec<ValueType>{}


impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::String(value) => write!(f, "{}", value.as_ref()),
            ValueType::Number(value) => write!(f, "{}", value.as_ref()),
            _ => unimplemented!()
        }
    }
}

impl fmt::Debug for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::String(value) => write!(f, "Managed Value(String): {}", value.as_ref()),
            ValueType::Number(value) => write!(f, "Managed Value(Number): {}", value.as_ref()),
            _ => unimplemented!()
        }
    }
}

impl Into<Option<HeapRef<String>>> for ValueType {
    fn into(self) -> Option<HeapRef<String>>{
        match self {
            ValueType::String(x) => Some(x),
            _ => None
        }
    }
}

// impl ValueType {
//     pub fn value<T : Allocation>(&self) -> &HeapRef<T> {
//         match self {
//             ValueType::String(x) => x,
//             ValueType::Number(x) => x,
//             ValueType::Array(x) => x
//         }
//     }
// }