Note: this implementation effectively makes allocated inner data immutable to the runtime. Attempting to mutate a value as-is will result
in a compiler error:

```rust
let mut heap = mem_manager::Heap::new();
let mut value = heap.allocate(String::from("I'm a string"));

*value = String::from("I'm another string");
```

This is very much by design! It's part of the whole relinquishing of control bit I was talking about at the top. Imagine 
if the runtime tried to do something like the following:

```rust 
let mut value = heap.allocate(String::from("I'm a string"));

*value = 64.25 // :O
``` 

Mutating data without the allocator's consent would be a terrible idea. It needs to know about every bit of memory it doles out 
in order to be effective. If we were to allow arbitrary side-effects to data from outside the allocator, it would
render our memory management model entirely useless. It may not be obvious now, but when we get into actual garbage collection strategies,
we'll see how dangerous that would be.