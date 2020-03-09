use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use std::cell::{Cell, RefCell};
use std::any::Any;
use std::fmt;
use std::mem;
use std::alloc::{alloc, handle_alloc_error, Layout};
use std::fmt::{Formatter, Error};


pub trait Allocation {}

pub type Array = Vec<HeapRef<dyn Allocation>>;
pub type ManagedValue = HeapRef<dyn Allocation>;
pub type Object = Box<Block<dyn Allocation>>;

enum ManagedValueType {
    Number,
    String,
    Array
}

pub struct Header {
    marked : Cell<bool>,
    value_type : ManagedValueType
}

pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

pub struct HeapRef<T: 'static + Allocation + ?Sized>(NonNull<Block<T>>);



#[derive(Copy, Clone)]
pub enum Value {
    Number(HeapRef<f64>),
    String(HeapRef<String>),
    Array(HeapRef<Array>)
}

impl Into<ManagedValueType> for Value {
    fn into(self) -> ManagedValueType{
        match self{
            Value::Array(_) => ManagedValueType::Array,
            Value::String(_) => ManagedValueType::String,
            Value::Number(_) => ManagedValueType::Number,
        }
    }
}

impl <T: 'static + ?Sized + Allocation> Copy for HeapRef<T> {}

impl <T: 'static + ?Sized + Allocation> Clone for HeapRef<T> {
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

    pub fn allocate<T: 'static + Allocation>(&mut self, data : T) -> ManagedValue {

        let mut allocation = Box::new(
            Block {
                header : Header::new(),
                data
            }
        );

        let pointer = unsafe { NonNull::new_unchecked(&mut *allocation)};

        // println!("Allocated {:?}@{:?} : ptr_size {}", &allocation.data, pointer, mem::size_of_val(&pointer));

        self.objects.push(allocation);

        HeapRef(pointer)
    }

    pub fn collect(&mut self){
        self.objects.retain(|val| val.header.marked.get())
    }
}

impl Header {
    fn new(t : ManagedValueType) -> Self {
        Self {
            value_type : t,
            marked : Cell::new(false),
        }
    }
}

impl<T: 'static + ?Sized + Allocation> Deref for HeapRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &(self.0.as_ref()).data
        }
    }
}

impl<T: 'static + ?Sized + Allocation> AsRef<T> for HeapRef<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl Allocation for String {}
impl Allocation for f64 {}
impl Allocation for Array{}

// impl fmt::Display for ManagedValue{
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_ref())
//     }
// }
//
// impl fmt::Debug for ManagedValue {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{:?}", self.as_ref())
//     }
// }
//
// impl fmt::Display for dyn Allocation {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self)
//     }
// }
//
impl fmt::Debug for HeapRef<dyn Allocation> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}


// impl fmt::Display for ValueType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ValueType::String(value) => write!(f, "{}", value.as_ref()),
//             ValueType::Number(value) => write!(f, "{}", value.as_ref()),
//             _ => unimplemented!()
//         }
//     }
// }
//
// impl fmt::Debug for ValueType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ValueType::String(value) => write!(f, "Managed Value(String): {}", value.as_ref()),
//             ValueType::Number(value) => write!(f, "Managed Value(Number): {}", value.as_ref()),
//             _ => unimplemented!()
//         }
//     }
// }

// impl Into<HeapRef<f64>> for HeapRef<dyn Allocation> {
//     fn into(self) -> HeapRef<f64>{
//         self as HeapRef<f64>
//     }
// }

// impl ValueType {
//     pub fn value<T : Allocation>(&self) -> &HeapRef<T> {
//         match self {
//             ValueType::String(x) => x,
//             ValueType::Number(x) => x,
//             ValueType::Array(x) => x
//         }
//     }
// }