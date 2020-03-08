# Garbage Day

### Hands-On Learning about Garbage Collection.

#### Chapter 1

The first question you might have is, "why in the world would I want to learn about Garbage Collection?". And I can't answer that.

I can however give you an overview of why _I_ want to learn more about GC.

It'd also be nice to skip over the obligatory Automatic Memory Management (AMM) pro/con list. If you're reading this
I'm making the assumption that you know that some languages manage memory dynamically, and why they made
that design decision.

---

Like a ton of other professional engineers out there, the majority of my career has been spent designing products
and platforms in a language that employs some version of AMM.

An interesting thing happens when you hand memory management over to the runtime of your choice: you relinquish control 
for safety and convenience. And most of the time, this trade-off turns out to be a perfectly fine -- if not optimal.
Managing Memory is a better job for robots.

However, things can and do go wrong. When you start to suspect that your application has a memory leak, you have to step away
from the wonderful land of automatic memory management and into the jungle of the runtime.

-- Ok, that's bit dramatic. But I'm sure that the majority of people who identify with the above have found themselves in
this scenario, staring at the hieroglyphics of an allocation graph.

I actually love moments like these! Getting to peak into the inner-workings of the run-time is an excellent learning experience.
It's one path into learning two things simultaniously: 

1) in-depth fundamentals of computer science and,
2) how the internals of your runtime of choice really, truly works

I've found that these two things have a compound effect, applicable in many other areas. In short, it makes you a better engineer.

That's the starting point. The thread -- if you will -- for why I want to learn more about Garbage Collection: to learn not
just about how one runtime works, but how Runtime_s_ are designed, in the general sense. Not only to become a better engineer,
but to learn some stuff about a thing that not a ton of us think about in our day-to-day.


#### Getting started

I will be using [The Garbage Collection Handbook](http://gchandbook.org/) as the guiding source of truth for these exercises.
It's widely considered the de-facto book on garbage collection, and contains pretty much everything you could ever want to know (and maybe even more)
on Modern GC design.

In addition to that, I'll be using the following texts for referencing runtime design principles:

- [Modern Compiler Design](https://www.springer.com/gp/book/9781461446989)
- [The Dragon Book](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
- [Crafting Interpreters](https://craftinginterpreters.com/)

We'll be implementing our Garbage Collectors in Rust for two reasons:

1) I like Rust a ton, and want to do some lower-level stuff with it.
2) Rust provides us with the sufficiently low-level interfaces we need to actually write a Memory Manager, while
offering wonderful zero-cost abstractions to introduce some safety around the inherently dangerous task of memory allocation.

---

In order to study different methods of garbage collection, we first need _something_ to collect. A typical scenario
that requires GC is a Runtime that allows the allocation of objects who's size cannot be known at compile-time -- in other words, 
a value with a size that cannot be known until the program is run.

To do this, we'll imagine a fake runtime and implement none of it.

All we'll care about are the values that this runtime provides, and provide and interface for managing them.

We'll allocate a chunk of memory on the heap based on the size of the value, and pass around a pointer to that region of memory.

The act of Garbage Collection is determining when that pointer can no longer be referenced by the program and yielding it back to the memory manager.

So we'll need to implement an allocator before we can do anything else.

Alright. Enough typing, let's get our hands dirty. Let's start with our imaginary Runtime values:

```rust
enum ValueType {
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

> Design note: we'll assume that our runtime allocates _every_ value in the heap.

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

That was easy! I don't know about you but I can _feel_ my Rust levelling up.

Let's create a `Heap` data structure for holding on to live objects:

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

// allocate something
let value : HeapRef<String> = heap.allocate(String::from("I'm a string"));

// use the value
assert_eq!(value.as_ref(), &*value);
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

🙌🙌🙌🙌We're almost there! There are a few missing key features for our allocator, but that's been enough code for today.

Next, we'll add two necessary concepts to our allocator, one more `ValueType` variant and implement a _ton_ of traits 
for `HeapRef` and `ValueType`.