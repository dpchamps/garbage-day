# Garbage Day

### Hands-On Learning about Garbage Collection.

#### Chapter 1

The first question you might have is, "why in the world would I want to learn about Garbage Collection?". 

While, I can't personally answer why you'd be interested in anything really, I can talk briefly about why I want to learn more about GC.

---

Like a lot of other engineers out there, I've spent my career designing products in a language that employs some version of Automatic Memory Management.

When we hand the responsibility of memory management over to the runtime, we enter into a contract: trading control for safety and convenience. 
Most of the time, this trade-off works perfectly fine, if not optimally. Managing memory is a better job for the robots. A perfect candidate for automation.

However, things can and do go wrong. And when they do, it's up to the developer who has been tasked with investigating a potential memory leak
to dive into the internals and start pulling on threads, poking edges on a graph and comparing heap snapshots. Having encountered several moments like these throughout my career,
I've found that understanding _what_ is going on behind the scenes is at least moderately important to be able to efficiently fix and diagnose these issues. Otherwise,
it leads to a lot of guesswork, staring at the hieroglyphics of an allocation graph -- wondering what all of it could mean.

Learning about and poking at the internals of your runtime is kind of paradoxical. In one light, we choose these higher-level languages in order to get away from the
low-level inconveniences. To be a little less imperative, so that we might solve our problems more efficiently. 

But at the same time knowing all well that our declarative wonderland is really just propped up by bits. And we do occasionally need to know about them, what they're up to
and why they might be acting up. 


My goal is to take this descent into _a_ runtime even further -- into the abstract: to learn not just about how one runtime works, 
but how Runtime_s_ are designed, in the general sense. In the process I hope to learn a lot about a thing not a ton of people talk about in their day-to-day and to share my findings 
with you.

Who knows, maybe the next time you're looking at a graph of your process in prod and start wondering what could be leaking memory, you'll be more equipped to deal with it. Ready even.

#### Getting started

I will be using [The Garbage Collection Handbook](http://gchandbook.org/) as the guiding source of truth for these exercises.
It's widely considered the de-facto book on garbage collection, and contains pretty much everything you could ever want to know (and maybe even more)
on Modern GC design.

In addition to that, I'll be using the following texts for referencing runtime design principles:

- [Modern Compiler Design](https://www.springer.com/gp/book/9781461446989)
- [The Dragon Book](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
- [Crafting Interpreters](https://craftinginterpreters.com/)

I've chosen to implement our Garbage Collectors in Rust for two reasons:

1) I like Rust a ton, and want to use it more.
2) Rust provides us with the sufficiently low-level interfaces we need to actually write a Memory Manager, while
offering wonderful zero-cost abstractions to introduce some safety around the inherently dangerous task of memory allocation.

---

#### Step 0 - The Allocator

In order to study different methods of garbage collection, we first need _something_ to collect. A scenario
that requires GC is a Runtime that allows the allocation of objects who's size cannot be known at compile-time.

For our GC implementations, we'll imagine a fake runtime and implement none of it, only worrying about are the underlying values 
and interface that exists for managing them.

> Note, this isn't something we'll just create once and move on. Garbage Collection is intimately tied to the administration 
> and management of objects on the heap. As we move through GC strategies, we'll expand-upon and tweak this
> implementation, noting the differences and the trade-offs that we're making as we go.
> For now however, we'll be creating something with the barest of bare-bones. 

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

It's not too hard to derive similar structure from the above two programs: we create a pointer, 
allocate some memory on the heap and then place a single char into that space. 
We can then print out the address in memory the pointer is pointing to along with the contents of that memory.

We don't have to free the memory in the `Rust` program, because we're leveraging a built-in Smart Pointer and `Rust`'s ownership model:
the compiler is able to determine at compile-time when the value is about to drop and call the corresponding free call.

Now let's compare that to something in Javascript:

```javascript
const char = 'a';

console.log(char);
```

A similar program is slightly out our reach in JS-Land, because all memory management is handled by the JS-Runtime. 
The best we can do is output the value of `char`. Our program is blissfully unaware of the where and how of the memory we're using. 
Although, it is possible to see what that underlying data structure looks like if we _really_ want to.
 
With the help of lldb and the llnode plugin, it's possible to drill down to something like this:

```bash
# ... find string in heap
(lldb) v8 inspect -s 0x77b167c7969
0x77b167c7969:<String: "a">
```

This turns out to be a helpful, because _this_ is the process we want to emulate through the above low-level techniques:

We'll allocate a chunk of memory on the heap based on the size of the value, and pass around a pointer to that region of memory.

Our Garbage Collection algorithms will determine when that pointer can no longer be referenced and yield it back to the memory manager.

Alright. Enough typing! Let's get our hands dirty. Let's start with our imaginary Runtime values, something similar to whatever, `<String: "a">` is above.

> Design note: we'll assume that our runtime allocates _every_ value in the heap.

```rust
enum Value {
  Number(HeapRef<f64>),
  String(HeapRef<String>)    
}
```

Gotta start somewhere.

Ok, so -- let's consider what the above code tells us:

We provide the definition of two separate values to our runtime. In user-land, a programmer might write something like this:

``` 
var stringVar = "Hello"
var numberVar = 10.8
```

The runtime will parse the above code, and then allocate memory for these variables.

In order to provide a generic interface for allocating and managing memory, we'll need to allocate _more_ than
the requested data for administrative purposes:

```rust
pub trait Allocation {}

pub struct Header {}
```

We start with the `Allocation` trait. This will provide a way for us to specify an interface for all of the values that
can be managed. It's empty now, but we'll expand it as we go.

Next, we create the `Header` struct. Again, empty for now. But this data will live alongside whatever other data has been requested 
for allocation. The header will be used for keeping tabs on our memory.

Finally, we need a definition for the actual things we'll be allocating as well as a data structure for referencing them:

```rust
pub struct Block<T: 'static + ?Sized + Allocation>{
    pub header : Header,
    pub data : T
}

pub type Object = Box<Block<dyn Allocation>>;

pub struct HeapRef<T: 'static + ?Sized + Allocation>(NonNull<Block<T>>);
```

_whew_ Let's break down the above code, starting with the elephant in the room

`T: 'static + ?Sized + Allocation`

Here, we indicate that our heap objects may be a generic type with the following bounds:

| bound      | meaning                                                          |
|------------|------------------------------------------------------------------|
| 'static    | That our value and it's references may outlive every lifetime. They don't have to, but they must be *able* to.   |
| ?Sized     | That our value may or may not have a known size at compile time. |
| Allocation | That our value implements our Allocation trait                   |

`Block` is the actual data we'll be allocating, whereas the `Object` type specifies the smart pointer for
our block allocation.

Finally, `HeapRef` is the SmartPointer our Memory Manager will pass back to the Runtime for referencing. It's 
a tuple struct that contains one element: a raw pointer to an allocated block.

If we do our jobs right, we provide the guarantee that as long as a `HeapRef` pointer can be referenced, it will point to the correct data.

We currently support allocations for `f64` and `String` types, so we'll have to implement the `Allocation` traits for those values:

```rust 
impl Allocation for String {}
impl Allocation for f64 {}
```

I don't know about you but I can _feel_ my Rust improving.

Let's create a `Heap` data structure for holding-on to live objects:

```rust
pub struct Heap {
    objects : Vec<Object>,
}
```

Now, the fun part. Let's make all this stuff_do_ something!

We know that at some point, the runtime will need to request an allocation from the Memory Manager. So what needs to happen
in order to provide that `HeapRef` back to the runtime?

Let's imagine driving the memory manager like this:

```rust
// create a heap instance
let mut heap = mem_manager::Heap::new();

// allocate something, note the question marks (indicating we don't know what to do here just yet
let value : <???> = heap.allocate(String::from("I'm a string"));

// use the value
assert_eq!(String::from("I'm a string"), *value);
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

Let's review the `DeRef` and `AsRef` traits we've implemented for our `HeapRef` smart pointer:

Memory location and administrative overhead should be opaque to the runtime. It doesn't need to know about any `Header` information we
store. All anything in the Runtime should care about is the initial data it requested to allocate. 

`HeapRef` should point to the block entire block, because that meta-data will be important should the runtime need to clone or copy a value. 
However, by implementing the above traits, we effectively provide the runtime access only to the data it requested.

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

This implementation isn't bad by any measure. But it's not exactly sufficient for our needs. Recall the question marks in
our specification above:

```rust 
let value : <???> = heap.allocate(String::from("I'm a string"));
```

As it stands, our implementation will return a `HeapRef<String>`. And while that may seem fine, remember that we want to be able to 
manage these values in a way that is agnostic of the underlying memory manager -- at least at some level.

Consider the following user-land code:

```
var x = "I'm a string";
var y = 10;
var z = [x, y];
```

Our current implementation does not allow for this representation easily.

Ultimately, there needs to be a way to represent an allocation as a generic managed value. To do this, 
we'll need to tweak the existing definition of the `Allocation` trait:

```rust
pub trait Allocation: UpcastValue +  std::fmt::Debug {}
```

The UpcastValue trait is straightforward, it leverages the [Any](https://doc.rust-lang.org/std/any/trait.Any.html) trait.

```rust
pub trait UpcastValue {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> UpcastValue for T {
    fn as_any(&self) -> &dyn Any { self }
}

pub type ManagedValue = HeapRef<dyn Allocation>;
```

Finally, we represent our generic value as an aliased type of `HeapRef<dyn Allocation>` indicating that
when we're dealing with a `ManagedValue`, we're passing around _something_ that our allocator is managing.

The signature for the `allocate` method will then change to:

`pub fn allocate<T: 'static + Allocation>(&mut self, data : T) -> ManagedValue`

which will give us the answer we need to the gap in our specification:

```rust
let value : ManagedValue = heap.allocate(String::from("I'm a string"));
```

However, this presents a new problem: we can't actually _use_ this allocation anywhere:

```rust
let number = heap.allocate(10.0);
assert_eq!(10.0, *number);
```

Will result in a compiler error:

> can't compare `{float}` with `dyn mem_manager::Allocation`

Which is reasonable! The compiler doesn't know anything about the underlying data. In order to be able reason about it,
we'll need to provide a way to `downcast` our managed values. This is tricky, and our implementation is bound to change.
But to get things up and running, let's implement something simple and to the point:

```rust
impl ManagedValue {
    pub fn downcast<T : Any + Allocation>(self) -> Option<HeapRef<T>>{
        if let Some(downcast_value) =  (*self).as_any().downcast_ref::<T>() {
            Some(HeapRef(self.0.cast::<Block<T>>()))
        }else{
            None
        }
    }
}
``` 

Perhaps not the _greatest_ implementation, but it does the job for now. Let's break it down:

##### `(*self).as_any()`

* First, we get the underlying data our HeapRef is pointing to -- which we know at least is encapsulated within `Block<dyn Allocation>. 
* The `data` field for `Block` is inevitably bound by the `UpcastValue` trait, which implements the `as_any` method to upcast it to an `Any` trait.

##### `if let Some(downcast_value) = <snip>.downcast_ref::<T>() {`

* After upcasting to any, we can now perform a check: does downcasting our `ManagedValue` match on the given type?
* If so, we can safely cast our pointer to a block of type `T`,
* Otherwise, we return `None` indicating that the corresponding type is incompatible.

*note* it would've been possible to recklessly cast our pointer to `Block<T>`, but that would be a terrible idea!

Consumers of our allocator implicitly sign a contract that states "we assume you know what you're doing". If we we're to simply cast our
pointer without checking the underlying data in this downcast step, we'd be handing it off to our consumers in an _invalid_ state. Imagine the following scenario:

```rust
impl ManagedValue {
    pub fn downcast<T : Any + Allocation>(self) -> HeapRef<T>{
       HeapRef(self.0.cast::<Block<T>>())
    }
}

let number = heap.allocate(10.0);

println!("{}", number.downcast<String>()); // SEG FAULT :O

```

_whew_ that was a ton of stuff!

But we've got a functioning allocator.

🙌🙌🙌🙌We're almost there! There are a few missing key features, but that's been enough code for today.

Next, we'll add two necessary concepts to our allocator, one more `Value` variant and implement a _ton_ of traits 
for `HeapRef` and `ValueType`.