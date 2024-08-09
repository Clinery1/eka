# About
This is Project Eka, my (hopefully) final programming language.

Everything is currently under construction, so anything can and will change at any moment (when I am
working on it).


# Why make Eka?
I wanted a simple, extensible language that I can use for other projects I may have.

It should have a simple syntax to allow for minimal thinking, but still be expressive enough for
general use.

The language should allow for different kinds of objects.


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
