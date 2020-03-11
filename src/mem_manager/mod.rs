use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use std::cell::{Cell, RefCell};
use std::any::Any;
use std::fmt;
use std::mem;
use std::alloc::{alloc, handle_alloc_error, Layout};
use std::fmt::{Formatter, Error};

pub trait UpcastValue {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> UpcastValue for T {
    fn as_any(&self) -> &dyn Any { self }
}

pub trait Allocation: UpcastValue +  std::fmt::Debug {}

pub type Array = Vec<HeapRef<dyn Allocation>>;
pub type ManagedValue = HeapRef<dyn Allocation>;
pub type Object = Box<Block<dyn Allocation>>;

#[derive(Debug)]
pub struct Header {
    marked : Cell<bool>,
}

#[derive(Debug)]
pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

#[derive(Debug)]
pub struct HeapRef<T: 'static+ ?Sized + Allocation >(NonNull<Block<T>>);

#[derive(Copy, Clone)]
pub enum Value {
    Number(HeapRef<f64>),
    String(HeapRef<String>),
    Array(HeapRef<Array>)
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
                header : Header::default(),
                data
            }
        );

        let pointer = unsafe { NonNull::new_unchecked(&mut *allocation)};

        self.objects.push(allocation);

        HeapRef(pointer)
    }

    pub fn collect(&mut self){
        self.objects.retain(|val| val.header.marked.get())
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
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

// impl<T: 'static + ?Sized + Allocation> Into<HeapRef<T>> for HeapRef<ManagedValue>{
//     fn into(self) -> HeapRef<T> {
//         unimplemented!()
//     }
// }

impl Allocation for String {}
impl Allocation for f64 {}
impl Allocation for Array{}

impl ManagedValue {
    pub fn downcast<T : Any + Allocation>(self) -> Option<HeapRef<T>>{
        unsafe {
            if let Some(downcast_value) = self.0.as_ref().data.as_any().downcast_ref::<T>(){
                Some(HeapRef(self.0.cast::<Block<T>>()))
            }else{
                None
            }
        }
    }
}

