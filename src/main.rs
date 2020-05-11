#![allow(dead_code)]
#![allow(unused_imports)]

// use crate::mem_manager::{HeapRef};
mod mem_manager;

use crate::mem_manager::{Heap, Value, Array, Allocation, ManagedValue, HeapRef};


fn main(){
    // let mut heap = Heap::new();
    //
    // let number = heap.allocate(10.0);
    // let string = heap.allocate(String::from("Hello"));
    //
    // // assert_eq!(10.0, *number);
    // println!("{}", *number.downcast::<String>().unwrap());
    // let array : HeapRef<Array> = heap.allocate(
    //     vec![number, string]
    // ).downcast().unwrap();
    //
    // println!("{:?}", *array);
    // println!("{:?}", *array[0].downcast::<f64>().unwrap());
    //
    // println!("[{:?}, {:?}]", *number.downcast::<f64>().unwrap(), *string.downcast::<String>().unwrap());
    // println!("[{}, {}]", *number.downcast::<f64>().unwrap(), *string.downcast::<String>().unwrap());
    // println!("[{:?}, {:?}]", number, string);
    //
    // heap.collect();
    //
    // // println!("{:?}", *array);
    // // println!("{:?}", *array[0].downcast::<f64>().unwrap());
    // // println!("[{:?}, {:?}]", *number.downcast::<f64>().unwrap(), *string.downcast::<String>().unwrap());
    // println!("[{}, {}]", *number.downcast::<f64>().unwrap(), *string.downcast::<String>().unwrap());
    // println!("[{:?}, {:?}]", number, string);


    // let d_cast_num = number.downcast::<f64>().unwrap();
    // let d_cast_string = string.downcast::<String>().unwrap();
    //
    // println!("{:?} -> {}", d_cast_num, *d_cast_num);
    // println!("{:?} -> {}", d_cast_string, *d_cast_string);
    //
    // println!("{:?}", number);
    // assert_eq!(10.0, *d_cast_num);

    // let array  = heap.allocate(vec![number, string]);

    // let test : HeapRef<f64> = number.into();

    // let test : &Array = array.as_ref();
    // println!("{:?}", *test);
    // let array = vec![string];

    // let string_val : Option<String> = (*string).into();

    // println!("{:?}", *number);

    // let mut heap = mem_manager::Heap::new();
    //
    // let string = heap.allocate(String::from("Hello"));
    //
    // println!("{:?} : {:?}", string.as_ref(), *string );
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
    // assert_eq!(10.0, number.as_ref());

    // println!("{:?} : {:?}", number.as_ref(), *number );
    // assert_eq!()
    // *value = String::from("Shit");
    // println!("{} : {}", value.as_ref(), *value );
}