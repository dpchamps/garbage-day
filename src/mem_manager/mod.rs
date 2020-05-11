use std::ptr::NonNull;
use std::ops::{Deref, DerefMut};
use std::cell::{Cell};
use std::any::Any;
use std::cmp::Ordering;

pub trait UpcastValue {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> UpcastValue for T {
    fn as_any(&self) -> &dyn Any { self }
}

pub trait Allocation: UpcastValue +  std::fmt::Debug {}

pub type Array = Vec<HeapRef<dyn Allocation>>;
pub type ManagedValue = HeapRef<dyn Allocation>;

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
    objects : Vec<Box<Block<dyn Allocation>>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects : vec![]
        }
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

        HeapRef(pointer)
    }

    pub fn collect(&mut self){
        println!("Collecting: {}", self.objects.len());
        self.objects.retain(|val| val.header.marked.get());
        println!("Done Collecting: {}", self.objects.len());
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

impl<T: 'static + ?Sized + Allocation> DerefMut for HeapRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut (self.0.as_mut()).data
        }
    }
}

impl<T: 'static + ?Sized + Allocation> AsRef<T> for HeapRef<T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<T: 'static + ?Sized + Allocation + PartialEq> PartialEq for HeapRef<T>{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_ref(),  other.as_ref())
    }

    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(self.as_ref(), other.as_ref())
    }
}

impl Allocation for String {}
impl Allocation for f64 {}
impl Allocation for Array{}

impl ManagedValue {
    pub fn downcast<T : Any + Allocation>(self) -> Result<HeapRef<T>, ManagedValue>{
        if let Some(_) =  (*self).as_any().downcast_ref::<T>() {
            Ok(HeapRef(self.0.cast::<Block<T>>()))
        }else{
            Err(self)
        }
    }
}

impl <T: 'static + Allocation> From<HeapRef<T>> for ManagedValue {
    fn from(item: HeapRef<T>) -> Self {
        HeapRef(item.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::mem_manager::{Heap, HeapRef, ManagedValue};

    #[test]
    fn it_allocates(){
        // create a heap instance
        let mut heap = Heap::new();

        // allocate something
        let value = heap.allocate(String::from("I'm a string"));

        // use the value
        assert_eq!(String::from("I'm a string"), *value);
    }

    #[test]
    fn it_upcasts(){
        let mut heap = Heap::new();

        let value = heap.allocate(String::from("hello"));
        let upcast : ManagedValue = value.into();
    }

    #[test]
    fn it_downcasts(){
        let mut heap = Heap::new();

        let value : ManagedValue = heap.allocate(10.0).into();

        assert_eq!(10.0, *value.downcast::<f64>().unwrap())
    }

    #[test]
    fn mutable_casts(){
        let mut heap = Heap::new();

        let mut value = heap.allocate(10.0);
        let mut managed : ManagedValue = value.into();

        *value = 11.0;

        assert_eq!(11.0, *value);
        assert_eq!(11.0, *managed.downcast::<f64>().unwrap());
    }
}
