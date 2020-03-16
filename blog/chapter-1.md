# Garbage Day

### Hands-On Learning about Garbage Collection.

#### Chapter 1

Like a lot of other engineers out there, I've spent most of my career designing products in a language that offers some form of Automatic Memory Management.

With AMM, we enter into a blood-pact with our runtime: trading control for safety and convenience. Most of the time, this oath works in our favor, if not optimally. Managing memory is a better job for the robots.

However, things can and do go wrong. When they do, it's a developer who has to jump head-first into runtime internals. 
Having encountered several moments like these throughout my career, I've found that understanding _what_ is going on behind the scenes is at least moderately important to be able to efficiently fix and diagnose these issues. Otherwise, it leads to a lot of guesswork, staring at the hieroglyphics of an allocation graph, poking at the edges, dissecting heap snapshots -- wondering what all of it could mean.

Getting intimate with the internals of your runtime might feel counter-intuitive. We choose these higher-level languages in order to get away from such low-level inconveniences. To be less imperative, so that we might solve our problems more efficiently. 

At the same we must be mindful of this predicament: that our wonderland of higher-level abstraction is really just propped up by bits. Occasionally we need to know about them, what they're up to and why they might be acting up. 


My goal is to take this descent into _a_ runtime even further -- into the abstract: to learn not just about how one runtime works, but how Runtime _s_ are designed, in the general sense. In the process I hope to learn a lot about a thing not a ton of people talk about in their day-to-day and to share my findings with you.

Who knows, maybe the next time you're looking at the ballooning memory of a process in prod and start wondering what could be leaking memory, you'll be more equipped to deal with it. Ready even.

#### Getting started

I will be using [The Garbage Collection Handbook](http://gchandbook.org/) as the guiding source of truth for these exercises. It's widely considered to be the de-facto book on garbage collection. It contains pretty much everything you could ever want to know  on Modern GC design. Maybe even more!

Additionally, I'll be using the following texts for referencing runtime design principles:

- [Modern Compiler Design](https://www.springer.com/gp/book/9781461446989)
- [The Dragon Book](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
- [Crafting Interpreters](https://craftinginterpreters.com/)

I've chosen to implement the Garbage Collectors in Rust for two reasons:

1) I like Rust a ton, and want to use it more.
2) Rust provides us with the sufficiently low-level interfaces we need to actually write a Memory Manager, while
offering wonderful zero-cost abstractions to introduce some safety around the inherently dangerous task of memory allocation.

---

#### The Allocator

In order to study different methods of garbage collection, we first need _something_ to collect.

For our GC implementations, we'll imagine a fake runtime and implement none of it. All we care about are the underlying values and interface that exists for managing them.

> Note Garbage Collection is intimately tied to the administration and management of objects on the heap. As we move through GC strategies, we'll expand-upon and tweak this implementation, noting the differences and the trade-offs that we're making as we go. For now however, we'll be creating something with the barest of bare-bones. 

If we need to create an Allocator first, let's think about allocation through the lens of some basic examples in other languages.

Let's start with something like `C`. 

```c
#include <stdio.h>
#include <stdlib.h>
int main() {
    char *ptr = (char*)malloc(1 * sizeof(char)); 
    
    *ptr = 'a';
    
    printf("%p -> %c\n", (void *)&ptr , *ptr);
    
    free(ptr);
	return 0;
}
```

and compare it to something similar in `Rust`

```rust
fn main() {
    let ptr = Box::new('a');
    
    println!("{:p} -> {}", ptr, *ptr);
}
```

Though different, It's not too hard to find similar structure from the above two programs: 

* Create a pointer
* Allocate some memory 
* Place a single char into that space
* Print the address the pointer is pointing to and the contents of that memory.

Now let's compare that to something in Javascript:

```javascript
const char = 'a';

console.log(char);
```

A similar program is out our reach in JS-Land, because all memory management is handled by the JS-Runtime. The best we can do is output the value of `char`. Our program is blissfully unaware of the where and how of the memory we're using. Although, it is possible to see what that underlying data structure looks like if we _really_ want to:

```bash
# ... find string in heap
(lldb) v8 inspect -s 0x77b167c7969
0x77b167c7969:<String: "a">
```

We don't even have control over the data-type. There's no way to allocate memory for a single char. The best we can get is a String datatype as outlined by whatever javascript engine is driving -- v8 in the case.

This is a helpful exercise at least, because this is the process we'll need to emulate.

The idea will be to allocate a chunk of memory based on the size of the value, and pass around a pointer to that region of memory. The various GC algorithms will determine when that pointer can no longer be referenced by the (nonexistent) runtime and yield it back to the memory manager.

Enough theory and planning, let's get our hands dirty! A good place to start is with the imaginary Runtime values, something similar to whatever, `<String: "a">` is above.

> Design note: we'll assume that our runtime allocates _every_ value in the heap.

```rust
enum Value {
  Number(HeapRef<f64>),
  String(HeapRef<String>)    
}
```

Starting with something straightforward: the definition for two separate values to our runtime. In user-land, a programmer might write something like this:

``` 
var stringVar = "Hello"
var numberVar = 10.8
```

The runtime will parse the above code, and then allocate memory for these variables.

We'll need to allocate _more_ than just the requested data for administrative purposes, though. So let's create some data structures for that now.

```rust
pub trait Allocation {}

pub struct Header {}
```

The `Allocation` trait will provide a way for us to specify an interface for all of the values that can be managed. It's empty now, but we'll expand it as we go.

Next, the `Header` struct  will live alongside whatever actual data has been requested for allocation. The header will be used for store meta-data for the allocated objects. For now, it will live alongside the data -- we'll see later some GC optimizations that call for storing certain fields elsewhere.

Finally, we need a definition for the actual things we'll be allocating as well as a data structure for referencing them:

```rust
pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

pub type Object = Box<Block<dyn Allocation>>;

pub struct HeapRef<T: 'static + ?Sized + Allocation>(NonNull<Block<T>>);
```

Don't worry! It looks worst than it is. Let's break down 

`T: 'static + ?Sized + Allocation`

This indicates that our heap objects may be a generic type with the following bounds:

| bound      | meaning                                                          |
|------------|------------------------------------------------------------------|
| 'static    | That our value and it's references may outlive every lifetime. They don't have to, but they must be *able* to.   |
| ?Sized     | That our value may or may not have a known size at compile time. |
| Allocation | That our value implements our Allocation trait                   |

The Data structures are all related 
| name       | purpose                                                          |
|------------|------------------------------------------------------------------|
| `Block`    | The data structure being allocated.                              |
| `Object`   | The `Box`ed Block, a way to refer to active data that has been allocated.|
| `HeapRef`  | A Smart Pointer that points to the location of the `Object` in memory. |

> We allocate an `Object` on the heap and point to the location in memory with a `HeapRef`

If we do our jobs right, the Allocator provides the guarantee that as long as a `HeapRef` pointer can be referenced, it will point to the correct data.

Currently `f64` and `String` are the only supported managed types, so we'll have to implement the `Allocation` traits for those values:

```rust
impl Allocation for String {}
impl Allocation for f64 {}
```

Let's create a `Heap` data structure for holding-on to live objects:

```rust
pub struct Heap {
    objects : Vec<Object>,
}
```

Let's _do something_ with all this stuff!

We know that at some point, the runtime will need to request an allocation from the Memory Manager. So what needs to happen in order to provide that `HeapRef` back to the runtime?

Let's sketch a specification:

```rust
#[cfg(test)]
mod tests {
    use crate::mem_manager::Heap;

    #[test]
    fn it_allocates(){
        // create a heap instance
        let mut heap = Heap::new();
        
        // allocate something
        let value = heap.allocate(String::from("I'm a string"));

        // use the value
        assert_eq!(String::from("I'm a string"), *value);
    }
}
```

1) The allocator will need to accept some data `T`, create a new `Block` and hold on to it. 
2) Create a `HeapRef` around a raw pointer to the newly created `Block` and return it to the Runtime.

I'll omit some uninteresting code in this next section,

```rust
impl Heap {
    pub fn new() -> Self {
        Heap {
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
```

Memory location and administrative overhead should be opaque to the runtime. It doesn't need to know about any `Header` information we store. All anything in the Runtime should care about is the initial data it requested to allocate. 

`HeapRef` should point to the block entire block, because that meta-data will be important should the runtime need to clone or copy a value. However, by implementing the above `AsRef` and `DeRef`, we effectively provide the runtime access only to the data it requested.

This implementation allows our specification to pass, and it isn't bad by any measure. But it's not exactly sufficient for our needs. 

As it stands, our implementation will return a `HeapRef<String>`. And while that may seem fine, remember that we want to be able to manage these values in a way that is agnostic of the underlying memory manager -- at least at some level.

Consider the following user-land code:

```
var x = "I'm a string";
var y = 10;
var z = [x, y];
```

Our current implementation has left a crucial requirement out.

Ultimately, there needs to be a way to represent an allocation as a generic managed value. To do this, we'll need to tweak the existing definition of the `Allocation` trait:

```rust
pub trait Allocation: UpcastValue +  std::fmt::Debug {}
```

`UpcastValue` is straightforward, it leverages the [Any](https://doc.rust-lang.org/std/any/trait.Any.html) trait, to specify a method
that upcasts any value to an `Any` bound trait.

```rust
pub trait UpcastValue {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> UpcastValue for T {
    fn as_any(&self) -> &dyn Any { self }
}

pub type ManagedValue = HeapRef<dyn Allocation>;
```

Finally, the generic value is just an alias for `HeapRef<dyn Allocation>` indicating that when we're dealing with a `ManagedValue`, we're passing around _something_ that our allocator is managing.

The signature for the `allocate` method will then change to:

`pub fn allocate<T: 'static + Allocation>(&mut self, data : T) -> ManagedValue`


However, this presents a new problem: we can't actually _use_ this allocation anywhere. Our tests are failing again! Let's inspect the compiler error:

> can't compare `{float}` with `dyn mem_manager::Allocation`

The compiler doesn't know anything about the underlying data. In order to be able reason about it, we'll need to provide a way to `downcast` our managed values. This is tricky, and our implementation is bound to change. But to get things up and running, let's implement something simple and to the point:

```rust
impl ManagedValue {
    pub fn downcast<T : Any + Allocation>(self) -> Option<HeapRef<T>>{
        if let Some(_) =  (*self).as_any().downcast_ref::<T>() {
            Some(HeapRef(self.0.cast::<Block<T>>()))
        }else{
            None
        }
    }
}
``` 

Perhaps not the _greatest_ implementation, but it does the job for now. Let's break it down:

##### `(*self).as_any()`

1) We get the underlying data `HeapRef` is pointing to -- which we know at least is encapsulated within `Block<dyn Allocation>`. 
2) The `data` field for `Block` is  bound by the `UpcastValue` trait. Since `UpcastValue` implements the `as_any` method for ant `T : Any`, `data` may be upcast to `Any`.

##### `if let Some(downcast_value) = <snip>.downcast_ref::<T>() {`

3) After upcasting, we can now perform a check: is downcasting `ManagedValue` to the given type valid?
    - If so, we can safely cast our pointer to a block of type `T`,
    - Otherwise, we return `None` indicating that the corresponding type is incompatible.



🙌  Alright! We've got a functioning allocator and one test. 🙌 

Celebrations are in order. There are a few missing key features, but that's enough code and theory for one day.

Next, we'll start exploring the most straight forward Garbage Collection strategy: Mark-Sweep. 