# About
This is Project Eka, my (hopefully) final programming language.

Everything is currently under construction, so anything can and will change at any moment (when I am
working on it).


# Why make Eka?
I wanted a simple, extensible language that I can use for other projects I may have.

It should have a simple syntax to allow for minimal thinking, but still be expressive enough for
general use.

The language should allow for different kinds of objects.

## Why start this when SimpleLisp has not been completed?
I was trying to reuse code as much as possible with SimpleLisp interpreter V2, but was ignoring the
tech debt from the old parser and stuff. Its best if I just rewrite it all so I can optimize it a
bit more for the current use-case instead of doing incremental rewrites and hacks to get it working.
Also, the entire object system was kind of flawed in how it worked. I was doing too many levels of
indirection to get to the right location, and its best if I rewrite a lot of it. See `src/object.rs`
for the new object system. It is basically the same, but it instead has a header type that acts as a
Vtable where I don't need to use as much indirection. Also, the data is allocated right after the
header, barring any padding, so it probably has better cache locality being a single large chunk.


# The object system
There technically isn't one, but there is an object protocol and that is used to interface the
language with Rust types.


# Continuations
There is "native" support for continuations via the Object protocol and the builtin yield keyword.


# Typing
Currently, the language is using dynamic typing, but I plan to add support for gradual typing later,
for some static typing if the object supports it. This is implemented on a per-object basis, and
will allow for a dynamic escape hatch.


# Logo
The logo will be a simple vector art of my rabbit laying down when I actually make it.
