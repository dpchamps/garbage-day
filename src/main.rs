use crate::mem_manager::ValueType;

mod mem_manager;

fn main(){
    let mut heap = mem_manager::Heap::new();

    let value = heap.allocate(String::from("Testy"));

    println!("{:?} : {}", value.ptr, *value );

    heap.collect();

    println!("{:?} : {}", value.ptr, *value );
}