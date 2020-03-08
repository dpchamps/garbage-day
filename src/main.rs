#![allow(dead_code)]

use crate::mem_manager::{HeapRef};

mod mem_manager;

fn main(){
    let mut heap = mem_manager::Heap::new();

    let string = heap.allocate(String::from("Hello"));

    println!("{:?} : {:?}", string.as_ref(), *string );
    // let p = heap.allocate_bytes(16);
    //
    // unsafe {
    //     println!("{:?} -> {}", p, *p);
    // }
    // let number = heap.allocate(65.0);
    // let string = heap.allocate(10.0);

    // let value = heap.allocate(
    //     vec![
    //         ValueType::Number(number),
    //         ValueType::String(string)
    //     ]
    // );
    //
    // let x : Option<HeapRef<String>> = value[1].into();
    //
    // println!("{}", x.unwrap().as_ref());
    //
    // heap.collect();
    //
    // println!("{}", x.unwrap().as_ref());
    // assert_eq!(*x.value(), *number);

    // println!("{:?} : {:?}", number.as_ref(), *number );
    // assert_eq!()
    // *value = String::from("Shit");
    // println!("{} : {}", value.as_ref(), *value );
}