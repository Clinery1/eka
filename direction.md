# Current direction
1: Written down features


# Directions I could go in
I have determined there are currently 3 directions I can go in with some minor overlap between them.
1. Implement a list of features and changes I had written down
2. Have a goal towards performance, but not too much; write a bytecode interpreter.
3. Focus on the end goal of a game-ready language with extensions for useful things like ECS.


## 1: Features I have written down (mid-term)
- Split into multiple packages (core, parser, interpreter, base)
- Make a generic interface for interpreters so we can have multiple.
- Add a bytecode interpreter.
- Start a WASM interpreter.
- Add first class continuations, and a way for a program to access its own continuation via a
    function like `call/cc`.
- Add various ways to call functions with the `call/NAME` syntax.
- Change the objects to NOT store the `Ident` of the fields, but instead store it in a
    `thread_local` `Cell` that is initialized upon creation of the first object.
- Change the `Keyword` token to strip the `:` so it has the same `Ident` as a raw name.


## 2: Performance/bytecode interpreter (short-term)
This direction will only last till I get the bytecode interpreter completed, then I will need to go
another direction, periodically revisiting this one when it gets slow again.
- Scrap the current tree-walk interpreter and write a bytecode interpreter.
- Make things fast at the expense of progress.
- Integrate the `profiling` crate into the interpreter.


## 3: Further the end goal
The end goal of this language is ideally one where I can write games. They may not be performant,
but if I need performant code, then I can write it in Rust. I just want to prototype and script with
Eka. I have a few ideas that will get me going towards that goal.

- Integrate the Rapier 2d and 3d engines into the language as `Object`s.
- Integrate a math library like Ultraviolet or NAlgebra as `Object`s.
- Integration with a graphics library (`glium`, `screen-13`, `luminance`, etc.).
